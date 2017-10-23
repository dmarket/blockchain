extern crate serde;
use exonum::crypto::Hash;
use exonum::blockchain;
use exonum::storage::{Fork, MapIndex, Snapshot};

use super::SERVICE_ID;

pub struct TxStatusSchema<'a> {
    pub view: &'a mut Fork,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum TxStatus {
    Fail,
    Success,
    Pending,
}

impl<'a> TxStatusSchema<'a> {
    pub fn txs(&mut self) -> MapIndex<&mut Fork, Hash, u8> {
        let prefix = blockchain::gen_prefix(SERVICE_ID, 2, &());
        MapIndex::new(prefix, self.view)
    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn get_status(&mut self, tx_hash: &Hash) -> Option<TxStatus> {
        match self.txs().get(tx_hash) {
            Some(0u8) => Some(TxStatus::Fail),
            Some(1u8) => Some(TxStatus::Success),
            Some(2u8) => Some(TxStatus::Pending),
            Some(_) | None => None,
        }
    }

    pub fn set_status(&mut self, tx_hash: &Hash, status: TxStatus) {
        let status = match status {
            TxStatus::Fail => 0u8,
            TxStatus::Success => 1u8,
            TxStatus::Pending => 2u8
        };
        self.txs().put(tx_hash, status);
    }

}

#[derive(Debug)]
pub struct TxSchema<T> {
    view: T,
}

impl<T> TxSchema<T>
where
    T: AsRef<Snapshot>,
{
    pub fn new(snapshot: T) -> TxSchema<T> {
        TxSchema { view: snapshot }
    }

    pub fn txs(&self) -> MapIndex<&T, Hash, u8> {
        let prefix = blockchain::gen_prefix(SERVICE_ID, 2, &());
        MapIndex::new(prefix, &self.view)

    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn get_status(&self, tx_hash: &Hash) -> Option<TxStatus> {
        match self.txs().get(tx_hash) {
            Some(0u8) => Some(TxStatus::Fail),
            Some(1u8) => Some(TxStatus::Success),
            Some(2u8) => Some(TxStatus::Pending),
            Some(_) | None => None,
        }
    }

}