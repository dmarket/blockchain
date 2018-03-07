//! Transaction statuses.

use exonum::crypto::Hash;
use exonum::storage::{Fork, MapIndex, Snapshot};

use currency::SERVICE_NAME;
use currency::error::Error;

/// Schema for accessing transaction statuses.
#[derive(Clone, Debug)]
pub struct Schema<S>(pub S)
where
    S: AsRef<Snapshot>;

type ResultRepr = u8;

fn to_repr(result: Result<(), Error>) -> ResultRepr {
    match result {
        Ok(_) => 0,
        Err(err) => err as u8,
    }
}

fn from_repr(repr: ResultRepr) -> Result<(), Error> {
    match repr {
        0 => Ok(()),
        value => Err(Error::try_from(value).expect("Invalid status repr")),
    }
}

impl<S> Schema<S>
where
    S: AsRef<Snapshot>,
{
    fn index(self) -> MapIndex<S, Hash, ResultRepr> {
        let key = SERVICE_NAME.to_string() + "v1.transactions";
        MapIndex::new(key, self.0)
    }

    /// Fetch transaction status for transaction.
    pub fn fetch(self, tx_id: &Hash) -> Option<Result<(), Error>> {
        self.index().get(tx_id).map(|repr| from_repr(repr))
    }
}

impl<'a> Schema<&'a mut Fork> {
    fn index_mut(&mut self) -> MapIndex<&mut Fork, Hash, ResultRepr> {
        let key = SERVICE_NAME.to_string() + "v1.transactions";
        MapIndex::new(key, self.0)
    }

    /// Store transaction status in the database
    pub fn store(&mut self, tx_id: Hash, status: Result<(), Error>) {
        let repr = to_repr(status);
        self.index_mut().put(&tx_id, repr);
    }
}
