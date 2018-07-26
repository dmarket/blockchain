use std::cmp;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::time::{Duration, Instant};

use exonum::blockchain::{Block, StoredConfiguration};
use exonum::crypto::Hash;
use exonum::helpers::Height;
use exonum_configuration::config_api::{ApiResponseConfigHashInfo, ApiResponseProposePost};
use futures::future;
use futures::future::Loop;
use futures::stream;
use futures::{Future, IntoFuture, Stream};
use hyper::{Body, Client, Request, Response, Uri};
use serde::Deserialize;
use serde_json as json;
use tokio::timer::Delay;

use nodes;
use nodes::NodeKeys;
use log;

const PROTOCOL_PREFIX: &str = "http://";
const LATEST_BLOCK_PATH: &str = "/api/services/cryptocurrency/v1/blocks?count=1";
const ACTUAL_CONFIG_PATH: &str = "/api/services/configuration/v1/configs/actual";
const PROPOSE_CONFIG_PATH: &str = "/api/services/configuration/v1/configs/postpropose";
const VOTE_CONFIG_PATH_BEGIN: &str = "/api/services/configuration/v1/configs/";
const VOTE_CONFIG_PATH_END: &str = "/postvote";
const HEIGHT_THRESHOLD: u64 = 5;
const ACTUAL_FROM_DELAY: u64 = 7;

pub fn new() -> impl Future<Item = (), Error = Error> {
    future::loop_fn((), |_| {
        stream::futures_unordered(nodes::list().into_iter().map(future::ok))
            .and_then(|(keys, info)| {
                get_height(&info.public).map(move |height| (keys, info, height))
            })
            .fold(
                (HashMap::new(), 0),
                |(mut map, highest), (keys, info, height)| {
                    map.insert(keys, (info, height));
                    Ok((map, cmp::max(highest, height))).into_future()
                },
            )
            .and_then(|(map, highest)| {
                if map.is_empty() {
                    Err(Error::NoNodes)
                } else {
                    Ok((map, highest))
                }.into_future()
            })
            .and_then(|(mut map, highest)| {
                let highest = highest - HEIGHT_THRESHOLD;
                map.retain(|_, &mut (_, height)| height >= highest);
                let map: HashMap<_, _> = map
                    .into_iter()
                    .map(|(keys, (info, _))| (keys, info))
                    .collect();

                assert!(
                    !map.is_empty(),
                    "at least one actual node must be present in the list"
                );

                future::ok(map)
            })
            .and_then(|candidates: HashMap<_, _>| {
                let node_public = candidates.values().next().unwrap().public.clone();
                get_validators(&node_public)
                    .and_then(|mut validators| {
                        if candidates.len() < validators.len() {
                            return Err(Error::ValidatorsAbsent)
                        }
                        let mut sorted_candidates: Vec<_> = candidates.keys().cloned().collect();
                        sorted_candidates.sort();
                        validators.sort();
                        if validators != sorted_candidates {
                            Ok((candidates, validators))
                        } else {
                            Err(Error::ValidatorsActual)
                        }
                    })
                    .join(get_height(&node_public))
            })
            .and_then(|((candidates, validators), height)| {
                debug!(log::KEEPER, "Preparing to propose config";
                       "candidates" => ?&candidates,
                       "validators" => ?&validators,
                       "height" => height);

                let actual_from = height + ACTUAL_FROM_DELAY;
                let proposer = candidates[&validators[0]].clone();
                let node_public = proposer.public.clone();
                let node_private = proposer.private.clone();
                propose_config(
                    node_public,
                    node_private,
                    candidates.keys().cloned().collect(),
                    actual_from,
                ).and_then(|hash| {
                    Delay::new(Instant::now() + Duration::from_secs(3))
                        .map_err(Error::from_std)
                        .map(move |_| hash)
                })
                .and_then(move |hash| {
                    // Send votes only to actual validators.
                    let mut voters = candidates;
                    voters.retain(|vk, _| validators.iter().any(|k| k == vk));

                    stream::futures_unordered(voters.values().cloned().map(future::ok))
                        .and_then(move |info| vote_config(info.private, hash))
                        .fold((), |_, _| Ok(()))
                })
            })
            .then(|result| {
                if let Err(e) = result {
                    error!(log::KEEPER, "Keeper cycle unsuccessful"; "error" => %e);
                }
                Delay::new(Instant::now() + Duration::from_secs(30)).map_err(Error::from_std)
            })
            .map(|_| Loop::Continue(()))
    })
}

fn get_height(addr: &str) -> impl Future<Item = u64, Error = Error> {
    let uri = [PROTOCOL_PREFIX, addr, LATEST_BLOCK_PATH]
        .concat()
        .parse()
        .unwrap();
    info!(log::KEEPER, "Get height"; "address" => %uri);

    Client::new()
        .get(uri)
        .map_err(Error::from_std)
        .and_then(parse_response)
        .map(|blocks: Result<Vec<Block>, ()>|
             blocks
                .unwrap()
                .pop()
                .unwrap()
                .height().0)
}

fn get_validators(addr: &str) -> impl Future<Item = Vec<NodeKeys>, Error = Error> {
    let uri = [PROTOCOL_PREFIX, addr, ACTUAL_CONFIG_PATH]
        .concat()
        .parse()
        .unwrap();

    info!(log::KEEPER, "Get validators"; "address" => %uri);

    Client::new()
        .get(uri)
        .map_err(Error::from_std)
        .and_then(parse_response)
        .map(|response: ApiResponseConfigHashInfo| {
            response
                .config
                .validator_keys
                .into_iter()
                .map(NodeKeys::from)
                .collect()
        })
}

fn propose_config(
    public: String,
    private: String,
    keys: Vec<NodeKeys>,
    actual_from: u64,
) -> impl Future<Item = Hash, Error = Error> {
    info!(log::KEEPER, "Propose config"; "from" => &private, "keys" => ?keys, "actual_from" => actual_from);

    Client::new()
        .get(
            [PROTOCOL_PREFIX, &public, ACTUAL_CONFIG_PATH]
                .concat()
                .parse()
                .unwrap(),
        )
        .map_err(Error::from_std)
        .and_then(parse_response)
        .and_then(move |response: ApiResponseConfigHashInfo| {
            let config = StoredConfiguration {
                validator_keys: keys
                    .into_iter()
                    .map(NodeKeys::into_validator_keys)
                    .collect(),
                actual_from: Height(actual_from),
                previous_cfg_hash: response.hash,
                ..response.config
            };
            let uri: Uri = [PROTOCOL_PREFIX, &private, PROPOSE_CONFIG_PATH]
                .concat()
                .parse()
                .unwrap();
            Client::new()
                .request(
                    Request::post(uri)
                        .header("Content-Type", "application/json")
                        .body(json::to_string_pretty(&config).unwrap().into())
                        .unwrap(),
                )
                .map_err(Error::from_std)
                .and_then(parse_response)
                .map(|response: ApiResponseProposePost| response.cfg_hash)
        })
}

fn vote_config(addr: String, hash: Hash) -> impl Future<Item = (), Error = Error> {
    info!(log::KEEPER, "Vote for config"; "from" => %addr, "hash" => ?hash);
    let uri: Uri = [
        PROTOCOL_PREFIX,
        &addr,
        VOTE_CONFIG_PATH_BEGIN,
        &hash.to_string(),
        VOTE_CONFIG_PATH_END,
    ].concat()
        .parse()
        .unwrap();

    Client::new()
        .request(
            Request::post(uri)
                .header("Content-Type", "application/json")
                .body(Body::empty()).unwrap()
        )
        .map_err(Error::from_std)
        .map(|_| ())
}

fn parse_response<T>(resp: Response<Body>) -> impl Future<Item = T, Error = Error>
where
    T: for<'a> Deserialize<'a>,
{
    resp.into_body()
        .concat2()
        .map_err(Error::from_std)
        .and_then(|chunk| {
            debug!(log::KEEPER, "Got response"; "response" => ::std::str::from_utf8(&chunk).unwrap());
            json::from_slice(&chunk)
                .map_err(Error::from_std)
                .into_future()
        })
}

#[derive(Debug)]
pub enum Error {
    NoNodes,
    ValidatorsActual,
    ValidatorsAbsent,
    Std(Box<StdError + Send + 'static>),
}

impl Error {
    fn from_std<E>(err: E) -> Self
    where
        E: StdError + Send + 'static,
    {
        Error::Std(Box::new(err))
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::NoNodes => write!(fmt, "no nodes"),
            Error::ValidatorsActual => write!(fmt, "validators actual"),
            Error::ValidatorsAbsent => write!(fmt, "some validators are absent"),
            Error::Std(err) => write!(fmt, "keeper error: {}", err),
        }
    }
}
