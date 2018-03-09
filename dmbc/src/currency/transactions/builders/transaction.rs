#![allow(missing_docs)]

use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::storage::StorageValue;

use currency;
use currency::assets::{AssetBundle, AssetId, Fees, MetaAsset, TradeAsset};
use currency::transactions::add_assets::AddAssets;
use currency::transactions::create_wallet::CreateWallet;
use currency::transactions::delete_assets::DeleteAssets;
use currency::transactions::exchange::{Exchange, ExchangeOffer};
use currency::transactions::components::{FeeStrategy, Intermediary};
use currency::transactions::exchange_intermediary::{ExchangeIntermediary,
                                                    ExchangeOfferIntermediary};
use currency::transactions::mining::Mining;
use currency::transactions::trade::{Trade, TradeOffer};
use currency::transactions::trade_intermediary::{TradeIntermediary, TradeOfferIntermediary};
use currency::transactions::transfer::Transfer;

pub struct Builder {
    public_key: Option<PublicKey>,
    secret_key: Option<SecretKey>,
    network_id: u32,
    protocol_version: u32,
    service_id: u16,
}

struct TransactionMetadata {
    public_key: PublicKey,
    secret_key: SecretKey,
    network_id: u32,
    protocol_version: u32,
    service_id: u16,
}

impl From<Builder> for TransactionMetadata {
    fn from(b: Builder) -> Self {
        TransactionMetadata {
            public_key: b.public_key.unwrap(),
            secret_key: b.secret_key.unwrap(),
            network_id: b.network_id,
            protocol_version: b.protocol_version,
            service_id: b.service_id,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            public_key: None,
            secret_key: None,
            network_id: 0,
            protocol_version: 0,
            service_id: currency::SERVICE_ID,
        }
    }

    pub fn keypair(self, public_key: PublicKey, secret_key: SecretKey) -> Self {
        Builder {
            public_key: Some(public_key),
            secret_key: Some(secret_key),
            ..self
        }
    }

    pub fn random_keypair(self) -> Self {
        let (public_key, secret_key) = crypto::gen_keypair();
        Builder {
            public_key: Some(public_key),
            secret_key: Some(secret_key),
            ..self
        }
    }

    pub fn network_id(self, network_id: u32) -> Self {
        Builder { network_id, ..self }
    }

    pub fn protocol_version(self, protocol_version: u32) -> Self {
        Builder {
            protocol_version,
            ..self
        }
    }

    pub fn service_id(self, service_id: u16) -> Self {
        Builder { service_id, ..self }
    }

    pub fn tx_add_assets(self) -> AddAssetBuilder {
        self.validate();
        AddAssetBuilder::new(self.into())
    }

    pub fn tx_create_wallet(self) -> CreateWalletBuilder {
        self.validate();
        CreateWalletBuilder::new(self.into())
    }

    pub fn tx_del_assets(self) -> DelAssetBuilder {
        self.validate();
        DelAssetBuilder::new(self.into())
    }

    pub fn tx_exchange(self) -> ExchangeBuilder {
        self.validate();
        ExchangeBuilder::new(self.into())
    }

    pub fn tx_exchange_with_intermediary(self) -> ExchangeIntermediaryBuilder {
        self.validate();
        ExchangeIntermediaryBuilder::new(self.into())
    }

    pub fn tx_mining(self) -> MiningBuilder {
        self.validate();
        MiningBuilder::new(self.into())
    }

    pub fn tx_trade_assets(self) -> TradeBuilder {
        self.validate();
        TradeBuilder::new(self.into())
    }

    pub fn tx_trade_assets_with_intermediary(self) -> TradeIntermediaryBuilder {
        self.validate();
        TradeIntermediaryBuilder::new(self.into())
    }

    pub fn tx_transfer(self) -> TransferBuilder {
        self.validate();
        TransferBuilder::new(self.into())
    }

    fn validate(&self) {
        match (&self.public_key, &self.secret_key) {
            (&Some(_), &Some(_)) => (),
            _ => panic!("Public and secret keys must be set."),
        }
    }
}

pub struct AddAssetBuilder {
    meta: TransactionMetadata,
    assets: Vec<MetaAsset>,
    seed: u64,
}

impl AddAssetBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        AddAssetBuilder {
            meta,
            assets: Vec::new(),
            seed: 0,
        }
    }

    pub fn add_asset(self, name: &str, count: u64, fees: Fees) -> Self {
        let asset = MetaAsset::new(&self.meta.public_key, name, count, fees);
        self.add_asset_value(asset)
    }

    pub fn add_asset_value(mut self, asset: MetaAsset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn add_asset_receiver(
        self,
        receiver: PublicKey,
        name: &str,
        count: u64,
        fees: Fees,
    ) -> Self {
        let asset = MetaAsset::new(&receiver, name, count, fees);
        self.add_asset_value(asset)
    }

    pub fn seed(self, seed: u64) -> Self {
        AddAssetBuilder { seed, ..self }
    }

    pub fn build(self) -> AddAssets {
        self.validate();
        AddAssets::new(
            &self.meta.public_key,
            self.assets,
            self.seed,
            &self.meta.secret_key,
        )
    }

    fn validate(&self) {}
}

pub struct CreateWalletBuilder {
    meta: TransactionMetadata,
}

impl CreateWalletBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        CreateWalletBuilder { meta }
    }

    pub fn build(self) -> CreateWallet {
        self.validate();
        CreateWallet::new(&self.meta.public_key, &self.meta.secret_key)
    }

    fn validate(&self) {}
}

pub struct DelAssetBuilder {
    meta: TransactionMetadata,
    assets: Vec<AssetBundle>,
    seed: u64,
}

impl DelAssetBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        DelAssetBuilder {
            meta,
            assets: Vec::new(),
            seed: 0,
        }
    }

    pub fn add_asset(self, name: &str, count: u64) -> Self {
        let asset = AssetBundle::from_data(name, count, &self.meta.public_key);
        self.add_asset_value(asset)
    }

    pub fn add_asset_value(mut self, asset: AssetBundle) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn seed(self, seed: u64) -> Self {
        DelAssetBuilder { seed, ..self }
    }

    pub fn build(self) -> DeleteAssets {
        self.validate();
        DeleteAssets::new(
            &self.meta.public_key,
            self.assets,
            self.seed,
            &self.meta.secret_key,
        )
    }

    fn validate(&self) {}
}

pub struct ExchangeBuilder {
    meta: TransactionMetadata,

    sender_assets: Vec<AssetBundle>,
    sender_value: u64,

    recipient: Option<PublicKey>,
    recipient_assets: Vec<AssetBundle>,

    fee_strategy: u8,

    seed: u64,

    data_info: Option<String>,
}

impl ExchangeBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        ExchangeBuilder {
            meta,

            sender_assets: Vec::new(),
            sender_value: 0,

            recipient: None,
            recipient_assets: Vec::new(),

            fee_strategy: 1,

            seed: 0,

            data_info: None,
        }
    }

    pub fn sender_add_asset(self, name: &str, count: u64) -> Self {
        let asset = AssetBundle::from_data(name, count, &self.meta.public_key);
        self.sender_add_asset_value(asset)
    }

    pub fn sender_add_asset_value(mut self, asset: AssetBundle) -> Self {
        self.sender_assets.push(asset);
        self
    }

    pub fn sender_value(self, sender_value: u64) -> Self {
        ExchangeBuilder {
            sender_value,
            ..self
        }
    }

    pub fn recipient(self, pub_key: PublicKey) -> Self {
        ExchangeBuilder {
            recipient: Some(pub_key),
            ..self
        }
    }

    pub fn recipient_add_asset(self, name: &str, count: u64) -> Self {
        let asset = AssetBundle::from_data(name, count, &self.recipient.unwrap());
        self.recipient_add_asset_value(asset)
    }

    pub fn recipient_add_asset_value(mut self, asset: AssetBundle) -> Self {
        self.recipient_assets.push(asset);
        self
    }

    pub fn fee_strategy(self, fee_strategy: u8) -> Self {
        ExchangeBuilder {
            fee_strategy,
            ..self
        }
    }

    pub fn seed(self, seed: u64) -> Self {
        ExchangeBuilder { seed, ..self }
    }

    pub fn data_info(self, data_info: &str) -> Self {
        ExchangeBuilder {
            data_info: Some(data_info.to_string()),
            ..self
        }
    }

    pub fn build(self) -> Exchange {
        self.verify();
        let offer = ExchangeOffer::new(
            &self.meta.public_key,
            self.sender_assets,
            self.sender_value,
            self.recipient.as_ref().unwrap(),
            self.recipient_assets,
            self.fee_strategy,
        );
        let signature = crypto::sign(&offer.clone().into_bytes(), &self.meta.secret_key);
        Exchange::new(
            offer,
            self.seed,
            &signature,
            &self.data_info.unwrap_or_default(),
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
        assert!(self.recipient.is_some());
        assert!(FeeStrategy::try_from(self.fee_strategy).is_some());
    }
}

pub struct ExchangeIntermediaryBuilder {
    meta: TransactionMetadata,

    intermediary_public_key: Option<PublicKey>,
    intermediary_secret_key: Option<SecretKey>,
    commision: u64,

    sender_assets: Vec<AssetBundle>,
    sender_value: u64,

    recipient: Option<PublicKey>,
    recipient_assets: Vec<AssetBundle>,

    fee_strategy: u8,

    seed: u64,

    data_info: Option<String>,
}

impl ExchangeIntermediaryBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        ExchangeIntermediaryBuilder {
            meta,

            intermediary_public_key: None,
            intermediary_secret_key: None,
            commision: 0,

            sender_assets: Vec::new(),
            sender_value: 0,

            recipient: None,
            recipient_assets: Vec::new(),

            fee_strategy: 1,

            seed: 0,

            data_info: None,
        }
    }

    pub fn sender_add_asset(self, name: &str, count: u64) -> Self {
        let asset = AssetBundle::from_data(name, count, &self.meta.public_key);
        self.sender_add_asset_value(asset)
    }

    pub fn sender_add_asset_value(mut self, asset: AssetBundle) -> Self {
        self.sender_assets.push(asset);
        self
    }

    pub fn sender_value(self, sender_value: u64) -> Self {
        ExchangeIntermediaryBuilder {
            sender_value,
            ..self
        }
    }

    pub fn intermediary_key_pair(self, public_key: PublicKey, secret_key: SecretKey) -> Self {
        ExchangeIntermediaryBuilder {
            intermediary_public_key: Some(public_key),
            intermediary_secret_key: Some(secret_key),
            ..self
        }
    }

    pub fn commision(self, commision: u64) -> Self {
        ExchangeIntermediaryBuilder {
            commision: commision,
            ..self
        }
    }

    pub fn recipient(self, pub_key: PublicKey) -> Self {
        ExchangeIntermediaryBuilder {
            recipient: Some(pub_key),
            ..self
        }
    }

    pub fn recipient_add_asset(self, name: &str, count: u64) -> Self {
        let asset = AssetBundle::from_data(name, count, &self.recipient.unwrap());
        self.recipient_add_asset_value(asset)
    }

    pub fn recipient_add_asset_value(mut self, asset: AssetBundle) -> Self {
        self.recipient_assets.push(asset);
        self
    }

    pub fn fee_strategy(self, fee_strategy: u8) -> Self {
        ExchangeIntermediaryBuilder {
            fee_strategy,
            ..self
        }
    }

    pub fn seed(self, seed: u64) -> Self {
        ExchangeIntermediaryBuilder { seed, ..self }
    }

    pub fn data_info(self, data_info: &str) -> Self {
        ExchangeIntermediaryBuilder {
            data_info: Some(data_info.to_string()),
            ..self
        }
    }

    pub fn build(self) -> ExchangeIntermediary {
        self.verify();

        let intermediary =
            Intermediary::new(&self.intermediary_public_key.unwrap(), self.commision);

        let offer = ExchangeOfferIntermediary::new(
            intermediary,
            &self.meta.public_key,
            self.sender_assets,
            self.sender_value,
            self.recipient.as_ref().unwrap(),
            self.recipient_assets,
            self.fee_strategy,
        );
        let signature = crypto::sign(&offer.clone().into_bytes(), &self.meta.secret_key);
        let intermediary_signature = crypto::sign(
            &offer.clone().into_bytes(),
            &self.intermediary_secret_key.unwrap(),
        );
        ExchangeIntermediary::new(
            offer,
            self.seed,
            &signature,
            &intermediary_signature,
            &self.data_info.unwrap_or_default(),
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
        assert!(self.recipient.is_some());
        assert!(self.intermediary_public_key.is_some());
        assert!(self.intermediary_secret_key.is_some());
    }
}

pub struct MiningBuilder {
    meta: TransactionMetadata,
    seed: u64,
}

impl MiningBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        MiningBuilder { meta, seed: 0 }
    }

    pub fn seed(self, seed: u64) -> Self {
        MiningBuilder { seed, ..self }
    }

    pub fn build(self) -> Mining {
        self.verify();
        Mining::new(&self.meta.public_key, self.seed, &self.meta.secret_key)
    }

    fn verify(&self) {}
}

pub struct TradeBuilder {
    meta: TransactionMetadata,
    buyer: Option<PublicKey>,
    assets: Vec<TradeAsset>,
    seed: u64,
}

impl TradeBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TradeBuilder {
            meta,
            buyer: None,
            assets: Vec::new(),
            seed: 0,
        }
    }

    pub fn buyer(self, pub_key: PublicKey) -> Self {
        TradeBuilder {
            buyer: Some(pub_key),
            ..self
        }
    }

    pub fn add_asset(self, name: &str, count: u64, price: u64) -> Self {
        let id = AssetId::from_data(name, &self.meta.public_key);
        let asset = TradeAsset::new(id, count, price);
        self.add_asset_value(asset)
    }

    pub fn add_asset_value(mut self, asset: TradeAsset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn seed(self, seed: u64) -> Self {
        TradeBuilder { seed, ..self }
    }

    pub fn build(self) -> Trade {
        self.verify();

        let offer = TradeOffer::new(&self.buyer.unwrap(), &self.meta.public_key, self.assets);
        let signature = crypto::sign(&offer.clone().into_bytes(), &self.meta.secret_key);
        Trade::new(offer, self.seed, &signature, &self.meta.secret_key)
    }

    fn verify(&self) {
        assert!(self.buyer.is_some());
    }
}

pub struct TradeIntermediaryBuilder {
    meta: TransactionMetadata,
    buyer: Option<PublicKey>,
    intermediary_public_key: Option<PublicKey>,
    intermediary_secret_key: Option<SecretKey>,
    commision: u64,

    assets: Vec<TradeAsset>,
    seed: u64,
    data_info: Option<String>,
}

impl TradeIntermediaryBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TradeIntermediaryBuilder {
            meta,
            buyer: None,
            intermediary_public_key: None,
            intermediary_secret_key: None,
            commision: 0,
            assets: Vec::new(),
            seed: 0,
            data_info: None,
        }
    }

    pub fn buyer(self, pub_key: PublicKey) -> Self {
        TradeIntermediaryBuilder {
            buyer: Some(pub_key),
            ..self
        }
    }

    pub fn intermediary_key_pair(self, public_key: PublicKey, secret_key: SecretKey) -> Self {
        TradeIntermediaryBuilder {
            intermediary_public_key: Some(public_key),
            intermediary_secret_key: Some(secret_key),
            ..self
        }
    }

    pub fn commision(self, commision: u64) -> Self {
        TradeIntermediaryBuilder {
            commision: commision,
            ..self
        }
    }

    pub fn add_asset(self, name: &str, count: u64, price: u64) -> Self {
        let id = AssetId::from_data(name, &self.meta.public_key);
        let asset = TradeAsset::new(id, count, price);
        self.add_asset_value(asset)
    }

    pub fn add_asset_value(mut self, asset: TradeAsset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn seed(self, seed: u64) -> Self {
        TradeIntermediaryBuilder { seed, ..self }
    }

    pub fn data_info(self, data_info: &str) -> Self {
        TradeIntermediaryBuilder {
            data_info: Some(data_info.to_string()),
            ..self
        }
    }

    pub fn build(self) -> TradeIntermediary {
        self.verify();

        let intermediary =
            Intermediary::new(&self.intermediary_public_key.unwrap(), self.commision);

        let offer = TradeOfferIntermediary::new(
            intermediary,
            &self.buyer.unwrap(),
            &self.meta.public_key,
            self.assets,
        );
        let signature = crypto::sign(&offer.clone().into_bytes(), &self.meta.secret_key);
        let intermediary_signature = crypto::sign(
            &offer.clone().into_bytes(),
            &self.intermediary_secret_key.unwrap(),
        );
        TradeIntermediary::new(
            offer,
            self.seed,
            &signature,
            &intermediary_signature,
            &self.data_info.unwrap_or_default(),
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
        assert!(self.buyer.is_some());
        assert!(self.intermediary_public_key.is_some());
        assert!(self.intermediary_secret_key.is_some());
    }
}

pub struct TransferBuilder {
    meta: TransactionMetadata,
    recipient: Option<PublicKey>,
    amount: u64,
    assets: Vec<AssetBundle>,
    seed: u64,
    data_info: Option<String>,
}

impl TransferBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TransferBuilder {
            meta,
            recipient: None,
            amount: 0,
            assets: Vec::new(),
            seed: 0,
            data_info: None,
        }
    }

    pub fn recipient(self, pub_key: PublicKey) -> Self {
        TransferBuilder {
            recipient: Some(pub_key),
            ..self
        }
    }

    pub fn amount(self, amount: u64) -> Self {
        TransferBuilder { amount, ..self }
    }

    pub fn add_asset(self, name: &str, count: u64) -> Self {
        let asset = AssetBundle::from_data(name, count, &self.meta.public_key);
        self.add_asset_value(asset)
    }

    pub fn add_asset_value(mut self, asset: AssetBundle) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn seed(self, seed: u64) -> Self {
        TransferBuilder { seed, ..self }
    }

    pub fn data_info(self, data_info: &str) -> Self {
        TransferBuilder {
            data_info: Some(data_info.to_string()),
            ..self
        }
    }

    pub fn build(self) -> Transfer {
        self.verify();

        Transfer::new(
            &self.meta.public_key,
            self.recipient.as_ref().unwrap(),
            self.amount,
            self.assets,
            self.seed,
            &self.data_info.unwrap_or_default(),
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
        assert!(self.recipient.is_some());
    }
}

#[cfg(test)]
mod test {
    use exonum::crypto;
    use exonum::storage::StorageValue;

    use dmbc::currency::asset::{AssetBundle, MetaAsset};

    use dmbc::currency::transaction::add_assets::AddAssets;
    use dmbc::currency::transaction::create_wallet::CreateWallet;
    use dmbc::currency::transaction::del_assets::DeleteAssets;
    use dmbc::currency::transaction::exchange::{Exchange, ExchangeOffer};
    use dmbc::currency::transaction::intermediary::Intermediary;
    use dmbc::currency::transaction::exchange_with_intermediary::{ExchangeIntermediary,
                                                                  ExchangeOfferIntermediary};
    use dmbc::currency::transaction::mining::Mining;
    use dmbc::currency::transaction::trade_assets::{Trade, TradeOffer};
    use dmbc::currency::transaction::trade_assets_with_intermediary::{TradeIntermediary,
                                                                      TradeOfferIntermediary};
    use dmbc::currency::transaction::trade_ask_assets::{TradeAsk, TradeAskOffer};
    use dmbc::currency::transaction::trade_ask_assets_with_intermediary::{TradeAskIntermediary,
                                                                          TradeAskOfferIntermediary};
    use dmbc::currency::transaction::transfer::Transfer;

    use dmbc::currency::builders::fee;
    use dmbc::currency::builders::transaction;

    #[test]
    #[should_panic]
    fn meta_underspecified() {
        transaction::Builder::new().tx_add_assets();
    }

    #[test]
    fn not_equal() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_mining()
            .seed(9)
            .build();

        let equivalent = Mining::new(&public_key, 18, &secret_key);

        assert!(transaction != equivalent);
    }

    #[test]
    fn add_assets() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let (receiver_key, _) = crypto::gen_keypair();

        let fees_foobar = fee::Builder::new()
            .trade(10, 10)
            .exchange(10, 10)
            .transfer(10, 10)
            .build();

        let fees_bazqux = fee::Builder::new()
            .trade(11, 10)
            .exchange(11, 10)
            .transfer(11, 10)
            .build();

        let asset_foobar = MetaAsset::new(&receiver_key, "foobar", 9, fees_foobar);
        let asset_bazqux = MetaAsset::new(&receiver_key, "bazqux", 18, fees_bazqux);

        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_add_assets()
            .add_asset_value(asset_foobar.clone())
            .add_asset_value(asset_bazqux.clone())
            .build();

        let assets = vec![asset_foobar, asset_bazqux];
        let equivalent = AddAssets::new(&public_key, assets, 0, &secret_key);

        assert!(transaction == equivalent);
    }

    #[test]
    fn create_wallet() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_create_wallet()
            .build();

        let equivalent = CreateWallet::new(&public_key, &secret_key);

        assert!(transaction == equivalent);
    }

    #[test]
    fn del_assets() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let asset = AssetBundle::from_data("foobar", 9, &public_key);
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_del_assets()
            .add_asset_value(asset.clone())
            .seed(6)
            .build();

        let assets = vec![asset];
        let equivalent = DeleteAssets::new(&public_key, assets, 6, &secret_key);

        assert!(transaction == equivalent);
    }

    #[test]
    fn exchange() {
        let (public_key, secret_key) = crypto::gen_keypair();

        let (recipient, _) = crypto::gen_keypair();
        let sender_asset = AssetBundle::from_data("foobar", 9, &public_key);
        let recipient_asset = AssetBundle::from_data("bazqux", 13, &public_key);

        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_exchange()
            .sender_add_asset_value(sender_asset.clone())
            .sender_value(9)
            .recipient(recipient)
            .recipient_add_asset_value(recipient_asset.clone())
            .fee_strategy(1)
            .seed(1)
            .data_info("test_exchange")
            .build();

        let offer = ExchangeOffer::new(
            &public_key,
            vec![sender_asset.clone()],
            9,
            &recipient,
            vec![recipient_asset.clone()],
            1,
        );
        let signature = crypto::sign(&offer.clone().into_bytes(), &secret_key);
        let equivalent = Exchange::new(offer, 1, &signature, "test_exchange", &secret_key);

        assert!(transaction == equivalent);
    }

    #[test]
    fn exchange_with_intermediary() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();

        let (recipient, _) = crypto::gen_keypair();
        let sender_asset = AssetBundle::from_data("foobar", 9, &public_key);
        let recipient_asset = AssetBundle::from_data("bazqux", 13, &public_key);
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_exchange_with_intermediary()
            .intermediary_key_pair(intermediary_public_key, intermediary_secret_key.clone())
            .commision(10)
            .sender_add_asset_value(sender_asset.clone())
            .sender_value(9)
            .recipient(recipient)
            .recipient_add_asset_value(recipient_asset.clone())
            .fee_strategy(1)
            .seed(1)
            .data_info("test_exchange")
            .build();

        let intermediary = Intermediary::new(&intermediary_public_key, 10);

        let offer = ExchangeOfferIntermediary::new(
            intermediary,
            &public_key,
            vec![sender_asset.clone()],
            9,
            &recipient,
            vec![recipient_asset.clone()],
            1,
        );
        let signature = crypto::sign(&offer.clone().into_bytes(), &secret_key);
        let intermediary_signature =
            crypto::sign(&offer.clone().into_bytes(), &intermediary_secret_key);
        let equivalent = ExchangeIntermediary::new(
            offer,
            1,
            &signature,
            &intermediary_signature,
            "test_exchange",
            &secret_key,
        );

        assert!(transaction == equivalent);
    }

    #[test]
    fn mining() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_mining()
            .seed(9)
            .build();

        let equivalent = Mining::new(&public_key, 9, &secret_key);

        assert!(transaction == equivalent);
    }

    #[test]
    fn trade_assets() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let (buyer, _) = crypto::gen_keypair();
        let asset = AssetBundle::from_data("foobar", 9, &public_key);
        let trade_asset = asset.into_trade_asset(9);
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_trade_assets()
            .add_asset_value(trade_asset.clone())
            .buyer(buyer)
            .seed(1)
            .build();

        let offer = TradeOffer::new(&buyer, &public_key, vec![trade_asset]);
        let signature = crypto::sign(&offer.clone().into_bytes(), &secret_key);
        let equivalent = Trade::new(offer, 1, &signature, &secret_key);

        assert!(transaction == equivalent);
    }

    #[test]
    fn trade_assets_with_intermediary() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();
        let (buyer, _) = crypto::gen_keypair();
        let asset = AssetBundle::from_data("foobar", 9, &public_key);
        let trade_asset = asset.into_trade_asset(9);
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_trade_assets_with_intermediary()
            .intermediary_key_pair(intermediary_public_key, intermediary_secret_key.clone())
            .commision(40)
            .add_asset_value(trade_asset.clone())
            .buyer(buyer)
            .seed(1)
            .data_info("trade_test")
            .build();

        let intermediary = Intermediary::new(&intermediary_public_key, 40);
        let offer =
            TradeOfferIntermediary::new(intermediary, &buyer, &public_key, vec![trade_asset]);
        let signature = crypto::sign(&offer.clone().into_bytes(), &secret_key);
        let intermediary_signature =
            crypto::sign(&offer.clone().into_bytes(), &intermediary_secret_key);
        let equivalent = TradeIntermediary::new(
            offer,
            1,
            &signature,
            &intermediary_signature,
            "trade_test",
            &secret_key,
        );

        assert!(transaction == equivalent);
    }

    #[test]
    fn trade_ask_assets() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let (buyer, _) = crypto::gen_keypair();
        let asset = AssetBundle::from_data("foobar", 9, &public_key);
        let trade_asset = asset.into_trade_asset(9);
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_trade_ask_assets()
            .add_asset_value(trade_asset.clone())
            .buyer(buyer)
            .seed(1)
            .data_info("trade_ask_test")
            .build();

        let offer = TradeAskOffer::new(&public_key, vec![trade_asset]);
        let signature = crypto::sign(&offer.clone().into_bytes(), &secret_key);
        let equivalent = TradeAsk::new(&buyer, offer, 1, &signature, "trade_ask_test", &secret_key);

        assert!(transaction == equivalent);
    }

    #[test]
    fn trade_ask_assets_with_intermediary() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();
        let (buyer, _) = crypto::gen_keypair();
        let asset = AssetBundle::from_data("foobar", 9, &public_key);
        let trade_asset = asset.into_trade_asset(9);
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_trade_ask_assets_with_intermediary()
            .add_asset_value(trade_asset.clone())
            .buyer(buyer)
            .intermediary_key_pair(intermediary_public_key, intermediary_secret_key.clone())
            .commision(30)
            .seed(1)
            .data_info("trade_ask_test")
            .build();

        let intermediary = Intermediary::new(&intermediary_public_key, 30);
        let offer = TradeAskOfferIntermediary::new(intermediary, &public_key, vec![trade_asset]);
        let signature = crypto::sign(&offer.clone().into_bytes(), &secret_key);
        let intermediary_signature =
            crypto::sign(&offer.clone().into_bytes(), &intermediary_secret_key);
        let equivalent = TradeAskIntermediary::new(
            &buyer,
            offer,
            1,
            &signature,
            &intermediary_signature,
            "trade_ask_test",
            &secret_key,
        );

        assert!(transaction == equivalent);
    }

    #[test]
    fn transfer() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let (recipient, _) = crypto::gen_keypair();
        let asset = AssetBundle::from_data("foobar", 9, &public_key);
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_transfer()
            .recipient(recipient)
            .amount(9)
            .add_asset_value(asset.clone())
            .seed(1)
            .data_info("info")
            .build();

        let equivalent = Transfer::new(
            &public_key,
            &recipient,
            9,
            vec![asset],
            1,
            "info",
            &secret_key,
        );

        assert!(transaction == equivalent);
    }
}
