use dmbc::currency::configuration::GENESIS_WALLET_PUB_KEY;
use exonum::crypto::PublicKey;
use exonum::encoding::serialize::FromHex;

pub fn default_genesis_wallet() -> PublicKey {
    PublicKey::from_hex(GENESIS_WALLET_PUB_KEY).unwrap()
}
