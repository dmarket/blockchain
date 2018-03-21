extern crate curl;
extern crate env_logger;
extern crate exonum;
extern crate log;
extern crate serde_json;

use curl::easy::Easy;
use exonum::crypto::PublicKey;
use std::io::Read;
use serde_json::Value;

const HTTP_GET: u8 = 1;
const HTTP_POST: u8 = 2;

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
        let url = self.url.clone() + "/api/services/cryptocurrency/v1/wallets";
        let data:Vec<u8> = Vec::new();

        self.request(HTTP_GET, url.to_string(), &data)
    }

    pub fn tx_send(&self, tx: &[u8]) -> String {
        let url = self.url.clone() + "/api/services/cryptocurrency/v1/transactions";

        let response = self.request(HTTP_POST, url.to_string(), tx);
        let v: Value = serde_json::from_str(&response).unwrap();
        let hash = v["tx_hash"].to_string();
        hash.trim_matches('"').to_string()

    }

    pub fn tx_status(&self, tx_hash: &str) -> String {
        let url = self.url.clone() + "/api/services/cryptocurrency/v1/transactions/" + tx_hash;
        let data:Vec<u8> = Vec::new();

        let response = self.request(HTTP_GET, url.to_string(), &data);
        let v: Value = serde_json::from_str(&response).unwrap();
        let status = v["tx_status"].to_string();
        status.trim_matches('"').to_string()

    }

    pub fn wallet(&self, pk: PublicKey) -> String {
        let s = pk.to_string();
        let url = self.url.clone() + "/api/services/cryptocurrency/v1/wallets/" + &s;
        let data:Vec<u8> = Vec::new();

        self.request(HTTP_GET, url.to_string(), &data)
    }

    fn request(&self, method: u8, url: String, data: &[u8]) -> String {
        let mut response = Vec::new();
        let mut send_data = data;
        let mut request = Easy::new();

        request.url(&url).unwrap();

        if method == HTTP_POST {
            info!("Request POST: {}", url);
            request.post(true).unwrap();
            request.post_field_size(data.len() as u64).unwrap();
        } else {
            info!("Request GET: {}", url);
        }
        {
            let mut transfer = request.transfer();
            if method == HTTP_POST {
                transfer.read_function(|buf| {
                    Ok(send_data.read(buf).unwrap_or(0))
                }).unwrap();
            }
            transfer.write_function(|dt| {
                response.extend_from_slice(dt);
                Ok(dt.len())
            }).unwrap();

            transfer.perform().unwrap();
        }

        let tx_response = String::from_utf8(response).unwrap();
        info!("{}", tx_response);
        tx_response
    }
}
