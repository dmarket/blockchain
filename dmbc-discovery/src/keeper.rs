use std::error::Error as StdError;
use std::fmt;
use std::time::{Duration, Instant};

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

pub fn new() -> impl Future<Item = (), Error = Error> {
    future::loop_fn((), |_| {
        stream::futures_unordered(nodes::list().into_iter().map(future::ok))
            .and_then(|(keys, info)| {
                eprintln!("Processing {:?}", &keys);
                latest_block_height(info.public).map(move |height| (keys, height))
            })
            .fold((), |_, (keys, height)| {
                eprintln!("Keys: {:?}, Height: {}", keys, height);
                future::ok(())
            })
            .then(|_| Delay::new(Instant::now() + Duration::from_secs(10)).map_err(Error::from_std))
            .map(|_| Loop::Continue(()))
    })
}

fn latest_block_height(addr: String) -> impl Future<Item = u64, Error = Error> {
    eprintln!("GET {}", addr.clone() + LATEST_BLOCK_PATH);
    Client::new()
        .get([PROTOCOL_PREFIX, &addr, LATEST_BLOCK_PATH].concat().parse().unwrap())
        .map_err(|e| Error::from_std(e))
        .and_then(|response| parse_response(response))
        .map(|block: Block| block.height().0)
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
pub struct Error(Option<Box<StdError + Send + 'static>>);

impl Error {
    fn from_std<E>(err: E) -> Self
    where
        E: StdError + Send + 'static,
    {
        Error(Some(Box::new(err)))
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match &self.0 {
            Some(err) => write!(fmt, "keeper error: {}", err),
            None => write!(fmt, "keeper error"),
        }
    }
}
