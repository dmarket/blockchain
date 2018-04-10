use exonum::encoding::serialize::FromHex;
use exonum::crypto::PublicKey;
use dmbc::currency::configuration::GENESIS_WALLET_PUB_KEY;

pub fn default_genesis_wallet() -> PublicKey {
    PublicKey::from_hex(GENESIS_WALLET_PUB_KEY).unwrap()
}
