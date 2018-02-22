use exonum::crypto::Hash;
use exonum::storage::{Fork, Snapshot};

use currency::transactions::Error;

/// Schema for accessing transaction statuses.
pub struct Schema<S>(pub S) where S: AsRef<Snapshot>;

impl<S> Schema<S>
where
    S: AsRef<Snapshot>
{
    /// Fetch transaction status for transaction.
    pub fn fetch(&self, tx_id: &Hash) -> Option<Result<(), Error>> {
        unimplemented!()
    }
}

impl<'a> Schema<&'a mut Fork> {
    /// Store transaction status in the database
    pub fn store(&mut self, tx_id: Hash, status: Result<(), Error>) {
        unimplemented!()
    }
}

