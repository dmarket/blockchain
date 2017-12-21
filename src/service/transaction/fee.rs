use service::asset::{Asset, AssetID, MetaAsset, TradeAsset};

pub struct Fee {
    for_tx: u64,
    for_add_assets: Option<u64>,

    for_marketplace: Option<u64>,
    for_trade_assets: Option<Vec<(AssetID, u64)>>,

    for_exchange: Option<u64>,
}

impl Fee {
    pub fn new(tx_fee: u64) -> Self {
        Fee {
            for_tx: tx_fee,
            for_add_assets: None,
            for_marketplace: None,
            for_trade_assets: None,
            for_exchange: None,
        }
    }

    pub fn amount(&self) -> u64 {
        let mut amount = self.for_tx;
        if let Some(assets_fee) = self.for_add_assets {
            amount += assets_fee;
        }
        if let Some(marketplace_fee) = self.for_marketplace {
            amount += marketplace_fee;
        }
        if let Some(ref trade_assets_fees) = self.for_trade_assets {
            amount += trade_assets_fees.iter().fold(0, |acc, asset| acc + asset.1)
        }
        if let Some(exchange_fee) = self.for_exchange {
            amount += exchange_fee;
        }

        amount
    }

    pub fn for_trade_assets(&self) -> Vec<(AssetID, u64)> {
        if let Some(ref trade_asset_fees) = self.for_trade_assets {
            return trade_asset_fees.clone();
        }

        vec![]
    }
}


pub struct TxCalculator {
    tx_fee: u64,
}

impl TxCalculator {
    pub fn new() -> Self {
        TxCalculator { tx_fee: 0 }
    }

    pub fn tx_fee(self, fee: u64) -> Self {
        TxCalculator { tx_fee: fee }
    }

    pub fn calculate(self) -> Fee {
        Fee::new(self.tx_fee)
    }

    pub fn add_asset_calculator(self) -> AddAssetCalculator {
        AddAssetCalculator::new(self)
    }

    pub fn del_asset_calculator(self) -> DelAssetCalculator {
        DelAssetCalculator::new(self)
    }

    pub fn transfer_callculator(self) -> TransferCalculator {
        TransferCalculator::new(self)
    }

    pub fn exchange_calculator(self) -> ExchangeCalculator {
        ExchangeCalculator::new(self)
    }

    pub fn trade_calculator(self) -> TradeCalculator {
        TradeCalculator::new(self)
    }
}

pub struct AddAssetCalculator {
    tx_calculator: TxCalculator,
    per_asset_fee: u64,
    assets: Vec<MetaAsset>,
}

impl AddAssetCalculator {
    pub fn new(tx_calc: TxCalculator) -> Self {
        AddAssetCalculator {
            tx_calculator: tx_calc,
            per_asset_fee: 0,
            assets: vec![],
        }
    }

    pub fn per_asset_fee(mut self, per_asset_fee: u64) -> Self {
        self.per_asset_fee = per_asset_fee;
        self
    }

    pub fn assets(mut self, assets: &Vec<MetaAsset>) -> Self {
        self.assets = assets.to_vec();
        self
    }

    pub fn calculate(self) -> Fee {
        let mut fee = self.tx_calculator.calculate();
        let count = self.assets.iter().fold(
            0,
            |acc, asset| acc + asset.amount() as u64,
        );
        fee.for_add_assets = Some(self.per_asset_fee * count);
        fee
    }
}

pub struct DelAssetCalculator {
    tx_calculator: TxCalculator,
}

impl DelAssetCalculator {
    pub fn new(tx_calc: TxCalculator) -> Self {
        DelAssetCalculator { tx_calculator: tx_calc }
    }

    pub fn calculate(self) -> Fee {
        self.tx_calculator.calculate()
    }
}

pub struct TransferCalculator {
    tx_calculator: TxCalculator,
}

impl TransferCalculator {
    pub fn new(tx_calc: TxCalculator) -> Self {
        TransferCalculator { tx_calculator: tx_calc }
    }

    pub fn calculate(self) -> Fee {
        self.tx_calculator.calculate()
    }
}

pub struct ExchangeCalculator {
    tx_calculator: TxCalculator,
    per_asset_fee: u64,
    assets: Vec<Asset>,
}

impl ExchangeCalculator {
    pub fn new(tx_calc: TxCalculator) -> Self {
        ExchangeCalculator {
            tx_calculator: tx_calc,
            per_asset_fee: 0,
            assets: vec![],
        }
    }

    pub fn per_asset_fee(mut self, per_asset_fee: u64) -> Self {
        self.per_asset_fee = per_asset_fee;
        self
    }

    pub fn assets(mut self, assets: &Vec<Asset>) -> Self {
        self.assets = assets.to_vec();
        self
    }

    pub fn calculate(self) -> Fee {
        let mut fee = self.tx_calculator.calculate();
        let count = self.assets.iter().fold(
            0,
            |acc, asset| acc + asset.amount() as u64,
        );
        fee.for_exchange = Some(count * self.per_asset_fee);
        fee
    }
}

pub struct TradeCalculator {
    tx_calculator: TxCalculator,
    marketplace_fee: u64,
    per_asset_fee: u64,
    assets: Vec<TradeAsset>,
}

impl TradeCalculator {
    pub fn new(tx_calc: TxCalculator) -> Self {
        TradeCalculator {
            tx_calculator: tx_calc,
            marketplace_fee: 0,
            per_asset_fee: 0,
            assets: vec![],
        }
    }

    pub fn marketplace_fee(mut self, marketplace_fee: u64) -> Self {
        self.marketplace_fee = marketplace_fee;
        self
    }

    pub fn per_asset_fee(mut self, per_asset_fee: u64) -> Self {
        self.per_asset_fee = per_asset_fee;
        self
    }

    pub fn assets(mut self, assets: &Vec<TradeAsset>) -> Self {
        self.assets = assets.to_vec();
        self
    }

    pub fn calculate(self) -> Fee {
        let get_fee = |price: u64, coef: u64| (price as f64 / coef as f64).round() as u64;

        let trade_assets_fees = self.assets
            .iter()
            .map(|asset| {
                (asset.id(), get_fee(asset.price(), self.per_asset_fee))
            })
            .collect();

        let mut fee = self.tx_calculator.calculate();
        fee.for_marketplace = Some(self.marketplace_fee);
        fee.for_trade_assets = Some(trade_assets_fees);

        fee
    }
}

#[cfg(test)]
mod test {
    use super::TxCalculator;
    use service::asset::{AssetID, MetaAsset, TradeAsset};

    #[test]
    fn test_add_asset_calculator() {
        let fee = TxCalculator::new().tx_fee(1000).calculate();
        assert_eq!(fee.amount(), 1000);

        let mut assets: Vec<MetaAsset> = Vec::new();
        let fee = TxCalculator::new()
            .tx_fee(1000)
            .add_asset_calculator()
            .assets(&assets)
            .per_asset_fee(1)
            .calculate();

        assert_eq!(fee.amount(), 1000);

        assets.push(MetaAsset::new("Asset#1", 2));
        assets.push(MetaAsset::new("Asset#2", 6));
        let fee = TxCalculator::new()
            .tx_fee(1000)
            .add_asset_calculator()
            .assets(&assets)
            .per_asset_fee(1)
            .calculate();

        assert_eq!(fee.amount(), 1008);
    }

    #[test]
    fn test_trade_calculator() {
        let mut assets: Vec<TradeAsset> = Vec::new();
        assets.push(TradeAsset::new(AssetID::zero(), 2, 1000));
        assets.push(TradeAsset::new(AssetID::zero(), 6, 1000));
        let fee = TxCalculator::new()
            .tx_fee(1000)
            .trade_calculator()
            .assets(&assets)
            .per_asset_fee(40)  // 1/40 = 0.025
            .calculate();

        assert_eq!(fee.amount(), 1050);
    }
}
