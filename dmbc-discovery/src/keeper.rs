use std::error::Error as StdError;
use std::fmt;
use std::time::{Duration, Instant};
use std::cmp;
use std::collections::HashMap;

use exonum::blockchain::Block;
use futures::future;
use futures::future::Loop;
use futures::stream;
use futures::{Future, IntoFuture, Stream};
use hyper::{Body, Client, Response};
use serde::Deserialize;
use serde_json as json;
use tokio::timer::Delay;

use nodes;

const PROTOCOL_PREFIX: &str = "http://";
const LATEST_BLOCK_PATH: &str = "/api/services/cryptocurrency/v1/blocks?count=1";
const HEIGHT_THRESHOLD: u64 = 5;

pub fn new() -> impl Future<Item = (), Error = Error> {
    future::loop_fn((), |_| {
        stream::futures_unordered(nodes::list().into_iter().map(future::ok))

            .and_then(|(keys, info)| get_height(&info.public)
                                         .map(move |height| (keys, info, height)))

            .fold((HashMap::new(), 0), |(mut map, highest), (keys, info, height)| {
                map.insert(keys, (info, height));
                Ok((map, cmp::max(highest, height))).into_future()
            })

            .and_then(|(map, highest)| {
                if map.is_empty() {
                    Err(Error::NoNodes)
                } else {
                    Ok((map, highest))
                }.into_future()
            })
        
            .and_then(|(map, highest)| {
                let highest = highest - HEIGHT_THRESHOLD;
                map.retain(|(_, &(_, height))| height >= highest);
                let map: HashMap<_, _> = map
                    .into_iter()
                    .map(|(keys, (info, _))| (keys, info))
                    .collect();

                assert!(!map.is_empty(), "at least one actual node must be present in the list");

                future::ok(map)
            })

            .and_then(|map| {
                let node_public = map.values().next().unwrap().public.clone();
                future::ok((get_validators(&node_public), map))
                    .and_then(|(validators, candidates)| {
                        if validators != candidates {
                            Ok(candidates)
                        } else {
                            Err(Error::ValidatorsActual)
                        }
                    })
            })

            .and_then(|map| {
                // futures intensify
                let node_private = map.values().next().unwrap().private.clone();
                propose_config(&node_private, map.keys().cloned().collect())
                    .and_then(move |hash| {
                        stream::futures_unordered(map.into_iter().map(future::ok))
                            .and_then(|| vote_config(&node_private, hash))
                            .fold(|_,_|())
                    })
            })

            .then(|_| Delay::new(Instant::now() + Duration::from_secs(10)).map_err(Error::from_std))
            .map(|_| Loop::Continue(()))
    })
}

fn get_height(addr: &str) -> impl Future<Item = u64, Error = Error> {
    eprintln!("GET {}", addr.to_strign() + LATEST_BLOCK_PATH);
    Client::new()
        .get([PROTOCOL_PREFIX, addr, LATEST_BLOCK_PATH].concat().parse().unwrap())
        .map_err(Error::from_std)
        .and_then(parse_response)
        .map(|block: Block| block.height().0)
}

fn get_validators(addr: &str) -> impl Future<Item = Vec<NodeKeys>, Error = Error> {
    Client::new()
        .get([PROTOCOL_PREFIX, addr, ACTUAL_CONFIG_PATH].concat().parse().unwrap())
        .map_err(Error::from_std)
        .and_then(parse_response)
        .map(|config: StoredConfiguration| config.validator_keys
             .into_iterator()
             .map(NodeKeys::from)
             .collect())
}

fn propose_config(addr: &str, keys: Vec<NodeKeys>) -> impl Future<Item = Hash, Error = Error> {
    Client::new()
        .get([PROTOCOL_PREFIX, addr, ACTUAL_CONFIG_PATH].concat().parse().unwrap())
        .map_err(Error::from_std)
        .and_then(parse_response)
        .map(|config| {
             Client::new()
                 .request(
                     Request::post([PROTOCOL_PREFIX, addr, PROPOSE_CONFIG_PATH].concat().parse().unwrap())
                         .body(json::to_string_pretty())
                         .map_err(Error::from_std)
                         .and_then(parse_response)
                         .map(|info| {
                         })
                 )
        })
}

fn parse_response<T>(resp: Response<Body>) -> impl Future<Item = T, Error = Error>
where
    T: for<'a> Deserialize<'a>,
{
    resp.into_body()
        .concat2()
        .map_err(Error::from_std)
        .and_then(|chunk| {
            json::from_slice(&chunk)
                .map_err(Error::from_std)
                .into_future()
        })
}

#[derive(Debug)]
pub enum Error{
    NoNodes,
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
        match &self.0 {
            Error::Std(Some(err)) => write!(fmt, "keeper error: {}", err),
        }
    }
}
