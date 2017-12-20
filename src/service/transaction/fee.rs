use service::asset::{Asset, MetaAsset, TradeAsset};

pub struct Fee {
    for_tx: u64,
    for_assets: Option<u64>,

    for_trade: Option<u64>,
    for_marketplace: Option<u64>,

    for_exchange: Option<u64>,
}

impl Fee {
    pub fn new(tx_fee: u64) -> Self {
        Fee {
            for_tx: tx_fee,
            for_assets: None,
            for_trade: None,
            for_marketplace: None,
            for_exchange: None,
        }
    }

    pub fn amount(self) -> u64 {
        let mut amount = self.for_tx;
        if let Some(assets_fee) = self.for_assets {
            amount += assets_fee;
        }
        if let Some(trade_fee) = self.for_trade {
            amount += trade_fee;
        }
        if let Some(marketplace_fee) = self.for_marketplace {
            amount += marketplace_fee;
        }
        if let Some(exchange_fee) = self.for_exchange {
            amount += exchange_fee;
        }

        amount
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
        fee.for_assets = Some(self.per_asset_fee * count);
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
    trade_fee: u64,
    per_asset_fee: u64,
    assets: Vec<TradeAsset>,
}

impl TradeCalculator {
    pub fn new(tx_calc: TxCalculator) -> Self {
        TradeCalculator {
            tx_calculator: tx_calc,
            trade_fee: 0,
            per_asset_fee: 0,
            assets: vec![],
        }
    }

    pub fn trade_fee(mut self, trade_fee: u64) -> Self {
        self.trade_fee = trade_fee;
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
        let mut fee = self.tx_calculator.calculate();
        let price = self.assets.iter().fold(
            0, 
            |sum, asset| sum + asset.price(),
        );
        fee.for_marketplace = Some(price * self.trade_fee);

        let count = self.assets.iter().fold(
            0,
            |acc, asset| acc + asset.amount() as u64,
        );

        fee.for_trade = Some(count * self.per_asset_fee);
        fee
    }
}

#[cfg(test)]
mod test {
    use super::TxCalculator;
    use service::asset::{Asset, AssetID};

    #[test]
    fn test_calculator() {
        let fee = TxCalculator::new().tx_fee(1000).calcluate();
        assert_eq!(fee.amount(), 1000);

        let mut assets: Vec<Asset> = Vec::new();
        let fee = TxCalculator::new()
            .tx_fee(1000)
            .asset_calculator::<Asset>()
            .assets(&assets)
            .per_asset_fee(1)
            .calculate();

        assert_eq!(fee.amount(), 1000);

        assets.push(Asset::new(AssetID::zero(), 2));
        assets.push(Asset::new(AssetID::zero(), 6));
        let fee = TxCalculator::new()
            .tx_fee(1000)
            .asset_calculator::<Asset>()
            .assets(&assets)
            .per_asset_fee(1)
            .calculate();

        assert_eq!(fee.amount(), 1008);

        let fee = TxCalculator::new()
            .tx_fee(1000)
            .asset_calculator::<Asset>()
            .assets(&assets)
            .per_asset_fee(1)
            .trade_calculator()
            .trade_fee(20)
            .calcluate();

        assert_eq!(fee.amount(), 1028);
    }
}
