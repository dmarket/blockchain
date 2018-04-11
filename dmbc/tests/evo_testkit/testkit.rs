
use exonum::crypto::{self, PublicKey};
use exonum::storage::Error;
use exonum_testkit::{TestKit as ExonumTestKit, TestKitBuilder, ApiKind};

use dmbc::currency::{SERVICE_NAME, Service};
use dmbc::currency::assets::AssetBundle;
use dmbc::currency::api::wallet::WalletResponse;
use dmbc::currency::wallet::{self, Wallet};

pub trait EvoTestKit {
    fn create_wallet(&mut self, pub_key: &PublicKey, balance: u64) -> Result<Wallet, Error>;

    fn add_assets(&mut self, pub_key: &PublicKey, assets: Vec<AssetBundle>) -> Result<(), Error>;
}

impl EvoTestKit for ExonumTestKit {
    fn create_wallet(&mut self, pub_key: &PublicKey, balance: u64) -> Result<Wallet, Error> {
        let blockchain = self.blockchain_mut();
        let mut fork = blockchain.fork();
        let wallet = Wallet::new(balance, vec![]);
        wallet::Schema(&mut fork).store(&pub_key, wallet.clone());

        blockchain.merge(fork.into_patch())?;

        Ok(wallet)
    }

    fn add_assets(&mut self, pub_key: &PublicKey, assets: Vec<AssetBundle>) -> Result<(), Error> {
        let blockchain = self.blockchain_mut();
        let mut fork = blockchain.fork();
        let wallet = wallet::Schema(&fork).fetch(&pub_key);
        let wallet = Wallet::new(wallet.balance(), assets);
        wallet::Schema(&mut fork).store(&pub_key, wallet);

        blockchain.merge(fork.into_patch())?;

        Ok(())
    }
}

pub fn get_wallet(kit: &ExonumTestKit, pub_key: &PublicKey) -> Wallet {
    let response: WalletResponse = kit.api().get(
        ApiKind::Service(SERVICE_NAME),
        &format!("v1/wallets/{}", pub_key.to_string()),
    );

    response.unwrap()
}

#[test]
fn name() {
    let mut testkit = TestKitBuilder::validator()
        .with_validators(4)
        .with_service(Service::new())
        .create();

    let (pub_key, _) = crypto::gen_keypair();

    let wallet = testkit.create_wallet(&pub_key, 100000).unwrap();

    let equivalent = get_wallet(&testkit, &pub_key);
    assert_eq!(wallet, equivalent);
}