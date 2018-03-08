extern crate curl;

use curl::easy::Easy;
use std::io::Read;

pub struct EvoClient {
    url: String,
}

impl EvoClient {
    pub fn new(url: String) -> Self {
        EvoClient {
            url
        }
    }

    pub fn wallets(&self) -> String {
        let mut data = Vec::new();
        let mut handle = Easy::new();
        let url = self.url.clone() + "/api/services/cryptocurrency/v1/wallets";
        handle.url(&url).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }
        String::from_utf8(data).unwrap()
    }

    pub fn tx_send(&self, tx: &[u8]) -> String {
        let mut response = Vec::new();
        let mut data = tx;
        let mut handle = Easy::new();

        let url = self.url.clone() + "/api/services/cryptocurrency/v1/transactions";

        handle.url(&url).unwrap();
        handle.post(true).unwrap();
        handle.post_field_size(data.len() as u64).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer.read_function(|buf| {
                Ok(data.read(buf).unwrap_or(0))
            }).unwrap();

            transfer.write_function(|dt| {
                response.extend_from_slice(dt);
                Ok(dt.len())
            }).unwrap();

            transfer.perform().unwrap();
        }

        String::from_utf8(response).unwrap()
    }

    pub fn tx_status(&self, tx_hash: &str) -> String {
        let mut data = Vec::new();
        let mut handle = Easy::new();
        let url = self.url.clone() + "/api/services/cryptocurrency/v1/transactions/" + tx_hash;
        handle.url(&url).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }
        String::from_utf8(data).unwrap()

    }


}
