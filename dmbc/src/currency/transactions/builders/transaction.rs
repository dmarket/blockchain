#![allow(missing_docs)]

use exonum::crypto;
use exonum::crypto::{PublicKey, SecretKey};
use exonum::storage::StorageValue;

use currency;
use currency::assets::{AssetBundle, AssetId, Fees, MetaAsset, TradeAsset};
use currency::transactions::add_assets::AddAssets;
use currency::transactions::delete_assets::DeleteAssets;
use currency::transactions::exchange::{Exchange, ExchangeOffer};
use currency::transactions::components::{FeeStrategy, Intermediary};
use currency::transactions::exchange_intermediary::{ExchangeIntermediary,
                                                    ExchangeOfferIntermediary};
use currency::transactions::mine::Mine;
use currency::transactions::trade::{Trade, TradeOffer};
use currency::transactions::trade_intermediary::{TradeIntermediary, TradeOfferIntermediary};
use currency::transactions::transfer::Transfer;

pub struct Builder {
    public_key: Option<PublicKey>,
    secret_key: Option<SecretKey>,
    network_id: u8,
    protocol_version: u8,
    service_id: u16,
}

struct TransactionMetadata {
    public_key: PublicKey,
    secret_key: SecretKey,
    network_id: u8,
    protocol_version: u8,
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

    pub fn network_id(self, network_id: u8) -> Self {
        Builder { network_id, ..self }
    }

    pub fn protocol_version(self, protocol_version: u8) -> Self {
        Builder { protocol_version, ..self }
    }

    pub fn service_id(self, service_id: u16) -> Self {
        Builder { service_id, ..self }
    }

    pub fn tx_add_assets(self) -> AddAssetBuilder {
        self.validate();
        AddAssetBuilder::new(self.into())
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

    pub fn tx_mine(self) -> MineBuilder {
        self.validate();
        MineBuilder::new(self.into())
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

    sender: Option<PublicKey>,
    sender_secret: Option<SecretKey>,

    sender_assets: Vec<AssetBundle>,
    sender_value: u64,

    recipient_assets: Vec<AssetBundle>,

    fee_strategy: FeeStrategy,

    seed: u64,

    data_info: Option<String>,
}

impl ExchangeBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        ExchangeBuilder {
            meta,

            sender: None,
            sender_secret: None,

            sender_assets: Vec::new(),
            sender_value: 0,

            recipient_assets: Vec::new(),

            fee_strategy: FeeStrategy::Recipient,

            seed: 0,

            data_info: None,
        }
    }
    pub fn sender(self, pub_key: PublicKey) -> Self {
        ExchangeBuilder {
            sender: Some(pub_key),
            ..self
        }
    }

    pub fn sender_secret(self, secret_key: SecretKey) -> Self {
        ExchangeBuilder {
            sender_secret: Some(secret_key),
            ..self
        }
    }


    pub fn sender_add_asset(self, name: &str, count: u64) -> Self {
        let asset = AssetBundle::from_data(name, count, &self.sender.unwrap());
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

    pub fn recipient_add_asset(self, name: &str, count: u64) -> Self {
        let asset = AssetBundle::from_data(name, count, &self.meta.public_key);
        self.recipient_add_asset_value(asset)
    }

    pub fn recipient_add_asset_value(mut self, asset: AssetBundle) -> Self {
        self.recipient_assets.push(asset);
        self
    }

    pub fn fee_strategy(self, fee_strategy: FeeStrategy) -> Self {
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
            self.sender.as_ref().unwrap(),
            self.sender_assets,
            self.sender_value,
            &self.meta.public_key,
            self.recipient_assets,
            self.fee_strategy as u8,
        );
        let sender_signature = crypto::sign(&offer.clone().into_bytes(), &self.sender_secret.unwrap());
        Exchange::new(
            offer,
            self.seed,
            &sender_signature,
            &self.data_info.unwrap_or_default(),
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
        assert!(self.sender.is_some());
        assert!(self.sender_secret.is_some());
    }
}

pub struct ExchangeIntermediaryBuilder {
    meta: TransactionMetadata,

    intermediary_public_key: Option<PublicKey>,
    intermediary_secret_key: Option<SecretKey>,
    commission: u64,

    sender_assets: Vec<AssetBundle>,
    sender_value: u64,

    recipient: Option<PublicKey>,
    recipient_assets: Vec<AssetBundle>,

    fee_strategy: FeeStrategy,

    seed: u64,

    data_info: Option<String>,
}

impl ExchangeIntermediaryBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        ExchangeIntermediaryBuilder {
            meta,

            intermediary_public_key: None,
            intermediary_secret_key: None,
            commission: 0,

            sender_assets: Vec::new(),
            sender_value: 0,

            recipient: None,
            recipient_assets: Vec::new(),

            fee_strategy: FeeStrategy::Recipient,

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

    pub fn commission(self, commission: u64) -> Self {
        ExchangeIntermediaryBuilder {
            commission,
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

    pub fn fee_strategy(self, fee_strategy: FeeStrategy) -> Self {
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
            Intermediary::new(&self.intermediary_public_key.unwrap(), self.commission);

        let offer = ExchangeOfferIntermediary::new(
            intermediary,
            &self.meta.public_key,
            self.sender_assets,
            self.sender_value,
            self.recipient.as_ref().unwrap(),
            self.recipient_assets,
            self.fee_strategy as u8,
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

pub struct MineBuilder {
    meta: TransactionMetadata,
    seed: u64,
}

impl MineBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        MineBuilder { meta, seed: 0 }
    }

    pub fn seed(self, seed: u64) -> Self {
        MineBuilder { seed, ..self }
    }

    pub fn build(self) -> Mine {
        self.verify();
        Mine::new(&self.meta.public_key, self.seed, &self.meta.secret_key)
    }

    fn verify(&self) {}
}

pub struct TradeBuilder {
    meta: TransactionMetadata,
    seller_public: Option<PublicKey>,
    seller_secret: Option<SecretKey>,
    assets: Vec<TradeAsset>,
    data_for_assets: Vec<(String, u64, u64)>,
    fee_strategy: FeeStrategy,
    seed: u64,
}

impl TradeBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TradeBuilder {
            meta,
            seller_public: None,
            seller_secret: None, 
            assets: Vec::new(),
            data_for_assets: Vec::new(),
            fee_strategy: FeeStrategy::Recipient,
            seed: 0,
        }
    }

    pub fn seller(self, pub_key: PublicKey, sec_key: SecretKey) -> Self {
        TradeBuilder {
            seller_public: Some(pub_key),
            seller_secret: Some(sec_key),
            ..self
        }
    }

    pub fn add_asset(mut self, name: &str, count: u64, price: u64) -> Self {
        self.data_for_assets.push((name.to_string(), count, price));
        self
    }

    pub fn add_asset_value(mut self, asset: TradeAsset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn fee_strategy(self, fee_strategy: FeeStrategy) -> Self {
        TradeBuilder {
            fee_strategy,
            ..self
        }
    }

    pub fn seed(self, seed: u64) -> Self {
        TradeBuilder { seed, ..self }
    }

    pub fn build(mut self) -> Trade {
        self.verify();

        for (name, count, price) in self.data_for_assets {
            let id = AssetId::from_data(&name, &self.seller_public.unwrap());
            let asset = TradeAsset::new(id, count, price);
            self.assets.push(asset);
        }

        let offer = TradeOffer::new(
            &self.meta.public_key, 
            &self.seller_public.unwrap(), 
            self.assets, 
            self.fee_strategy as u8
        );
        let signature = crypto::sign(&offer.clone().into_bytes(), &self.seller_secret.unwrap());
        Trade::new(offer, self.seed, &signature, &self.meta.secret_key)
    }

    fn verify(&self) {
        assert!(self.seller_public.is_some());
        assert!(self.seller_secret.is_some());
    }
}

pub struct TradeIntermediaryBuilder {
    meta: TransactionMetadata,
    seller_public: Option<PublicKey>,
    seller_secret: Option<SecretKey>,
    intermediary_public_key: Option<PublicKey>,
    intermediary_secret_key: Option<SecretKey>,
    commission: u64,

    assets: Vec<TradeAsset>,
    data_for_assets: Vec<(String, u64, u64)>,
    fee_strategy: FeeStrategy,
    seed: u64,
    data_info: Option<String>,
}

impl TradeIntermediaryBuilder {
    fn new(meta: TransactionMetadata) -> Self {
        TradeIntermediaryBuilder {
            meta,
            seller_public: None,
            seller_secret: None,
            intermediary_public_key: None,
            intermediary_secret_key: None,
            commission: 0,
            assets: Vec::new(),
            data_for_assets: Vec::new(),
            fee_strategy: FeeStrategy::Recipient,
            seed: 0,
            data_info: None,
        }
    }

    pub fn seller(self, pub_key: PublicKey, sec_key: SecretKey) -> Self {
        TradeIntermediaryBuilder {
            seller_public: Some(pub_key),
            seller_secret: Some(sec_key),
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

    pub fn commission(self, commission: u64) -> Self {
        TradeIntermediaryBuilder {
            commission: commission,
            ..self
        }
    }

    pub fn add_asset(mut self, name: &str, count: u64, price: u64) -> Self {
        self.data_for_assets.push((name.to_string(), count, price));
        self
    }

    pub fn add_asset_value(mut self, asset: TradeAsset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn fee_strategy(self, fee_strategy: FeeStrategy) -> Self {
        TradeIntermediaryBuilder {
            fee_strategy,
            ..self
        }
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

    pub fn build(mut self) -> TradeIntermediary {
        self.verify();

        for (name, count, price) in self.data_for_assets {
            let id = AssetId::from_data(&name, &self.seller_public.unwrap());
            let asset = TradeAsset::new(id, count, price);
            self.assets.push(asset);
        }

        let intermediary =
            Intermediary::new(&self.intermediary_public_key.unwrap(), self.commission);

        let offer = TradeOfferIntermediary::new(
            intermediary,
            &self.meta.public_key,
            &self.seller_public.unwrap(),
            self.assets,
            self.fee_strategy as u8,
        );
        let seller_signature = crypto::sign(&offer.clone().into_bytes(), &self.seller_secret.unwrap());
        let intermediary_signature = crypto::sign(
            &offer.clone().into_bytes(),
            &self.intermediary_secret_key.unwrap(),
        );
        TradeIntermediary::new(
            offer,
            self.seed,
            &seller_signature,
            &intermediary_signature,
            &self.data_info.unwrap_or_default(),
            &self.meta.secret_key,
        )
    }

    fn verify(&self) {
        assert!(self.seller_public.is_some());
        assert!(self.seller_secret.is_some());
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

    use currency::assets::{AssetBundle, MetaAsset, TradeAsset};

    use currency::transactions::add_assets::AddAssets;
    use currency::transactions::delete_assets::DeleteAssets;
    use currency::transactions::exchange::{Exchange, ExchangeOffer};
    use currency::transactions::components::{FeeStrategy, Intermediary};
    use currency::transactions::exchange_intermediary::{ExchangeIntermediary,
                                                        ExchangeOfferIntermediary};
    use currency::transactions::mine::Mine;
    use currency::transactions::trade::{Trade, TradeOffer};
    use currency::transactions::trade_intermediary::{TradeIntermediary,
                                                     TradeOfferIntermediary};
    use currency::transactions::transfer::Transfer;

    use currency::transactions::builders::fee;
    use currency::transactions::builders::transaction;

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
            .tx_mine()
            .seed(9)
            .build();

        let equivalent = Mine::new(&public_key, 18, &secret_key);

        assert_ne!(transaction, equivalent);
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

        assert_eq!(transaction, equivalent);
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

        assert_eq!(transaction, equivalent);
    }

    #[test]
    fn exchange() {
        let (recipient_pk, recipient_sk) = crypto::gen_keypair();

        let (sender_pk, sender_sk) = crypto::gen_keypair();
        let sender_asset = AssetBundle::from_data("foobar", 9, &sender_pk);
        let recipient_asset = AssetBundle::from_data("bazqux", 13, &recipient_pk);

        let transaction = transaction::Builder::new()
            .keypair(recipient_pk, recipient_sk.clone())
            .tx_exchange()
            .sender(sender_pk)
            .sender_secret(sender_sk.clone())
            .sender_add_asset_value(sender_asset.clone())
            .sender_value(9)
            .recipient_add_asset_value(recipient_asset.clone())
            .fee_strategy(FeeStrategy::Recipient)
            .seed(1)
            .data_info("test_exchange")
            .build();

        let offer = ExchangeOffer::new(
            &sender_pk,
            vec![sender_asset.clone()],
            9,
            &recipient_pk,
            vec![recipient_asset.clone()],
            1,
        );
        let sender_signature = crypto::sign(&offer.clone().into_bytes(), &sender_sk.clone());
        let equivalent = Exchange::new(offer, 1, &sender_signature, "test_exchange", &recipient_sk);

        assert_eq!(transaction, equivalent);
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
            .commission(10)
            .sender_add_asset_value(sender_asset.clone())
            .sender_value(9)
            .recipient(recipient)
            .recipient_add_asset_value(recipient_asset.clone())
            .fee_strategy(FeeStrategy::Recipient)
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

        assert_eq!(transaction, equivalent);
    }

    #[test]
    fn mine() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_mine()
            .seed(9)
            .build();

        let equivalent = Mine::new(&public_key, 9, &secret_key);

        assert_eq!(transaction, equivalent);
    }

    #[test]
    fn trade_assets() {
        let (public_key, secret_key) = crypto::gen_keypair();
        let (seller_public, seller_secret) = crypto::gen_keypair();
        let asset = AssetBundle::from_data("foobar", 9, &public_key);
        let trade_asset = TradeAsset::from_bundle(asset, 10);
        let transaction = transaction::Builder::new()
            .keypair(public_key, secret_key.clone())
            .tx_trade_assets()
            .add_asset_value(trade_asset.clone())
            .seller(seller_public, seller_secret.clone())
            .fee_strategy(FeeStrategy::Recipient)
            .seed(1)
            .build();

        let offer = TradeOffer::new(&public_key, &seller_public, vec![trade_asset], FeeStrategy::Recipient as u8);
        let signature = crypto::sign(&offer.clone().into_bytes(), &seller_secret);
        let equivalent = Trade::new(offer, 1, &signature, &secret_key);

        assert_eq!(transaction, equivalent);
    }

    #[test]
    fn trade_assets_with_intermediary() {
        let (buyer_public_key, buyer_secret_key) = crypto::gen_keypair();
        let (intermediary_public_key, intermediary_secret_key) = crypto::gen_keypair();
        let (seller_public_key, seller_secret_key) = crypto::gen_keypair();
        let asset = AssetBundle::from_data("foobar", 9, &seller_public_key);
        let trade_asset = TradeAsset::from_bundle(asset, 10);
        let transaction = transaction::Builder::new()
            .keypair(buyer_public_key, buyer_secret_key.clone())
            .tx_trade_assets_with_intermediary()
            .intermediary_key_pair(intermediary_public_key, intermediary_secret_key.clone())
            .commission(40)
            .add_asset_value(trade_asset.clone())
            .seller(seller_public_key, seller_secret_key.clone())
            .fee_strategy(FeeStrategy::Recipient)
            .seed(1)
            .data_info("trade_test")
            .build();

        let intermediary = Intermediary::new(&intermediary_public_key, 40);
        let offer =
            TradeOfferIntermediary::new(
                intermediary, 
                &buyer_public_key, 
                &seller_public_key, 
                vec![trade_asset], 
                FeeStrategy::Recipient as u8
            );
        let seller_signature = crypto::sign(&offer.clone().into_bytes(), &seller_secret_key);
        let intermediary_signature =
            crypto::sign(&offer.clone().into_bytes(), &intermediary_secret_key);
        let equivalent = TradeIntermediary::new(
            offer,
            1,
            &seller_signature,
            &intermediary_signature,
            "trade_test",
            &buyer_secret_key,
        );

        assert_eq!(transaction, equivalent);
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

        assert_eq!(transaction, equivalent);
    }
}
