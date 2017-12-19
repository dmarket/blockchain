
use service::asset::{Asset, MetaAsset};

struct Fee {
    for_tx: u64,
    for_assets: Option<u64>,
    for_trade: Option<u64>,
}

impl Fee {
    pub fn new(tx_fee: u64, assets_fee: Option<u64>, trade_fee: Option<u64>) -> Self {
        Fee { for_tx: tx_fee, for_assets: assets_fee, for_trade: trade_fee }
    }

    pub fn amount(self) -> u64 {
        let mut amount = self.for_tx;
        if let Some(assets_fee) = self.for_assets {
            amount += assets_fee;
        }
        if let Some(trade_fee) = self.for_trade {
            amount += trade_fee;
        }

        amount
    }
}


struct TxCalculator {
    tx_fee: u64,
}

impl TxCalculator {
    pub fn new() -> Self {
        TxCalculator { tx_fee: 0, }
    }

    pub fn tx_fee(self, fee: u64) -> Self {
        TxCalculator { tx_fee: fee }
    }

    pub fn calcluate(self) -> Fee {
        Fee::new(self.tx_fee, None, None)
    }

    pub fn asset_calculator<T>(self) -> AssetCalculator<T> {
        AssetCalculator::<T>::new(self)
    }
}

struct AssetCalculator<T> {
    tx_calculator: TxCalculator,
    per_asset_fee: u64,
    assets: Vec<T>,
}

impl<T> AssetCalculator<T> {
    pub fn new(tx_calc: TxCalculator) -> Self {
        AssetCalculator { tx_calculator: tx_calc, per_asset_fee: 0, assets: vec![], }
    }

    pub fn per_asset_fee(mut self, per_asset_fee: u64) -> Self {
        self.per_asset_fee = per_asset_fee;
        self
    }
}

impl AssetCalculator<MetaAsset> {
    pub fn assets(mut self, assets: &Vec<MetaAsset>) -> Self {
        self.assets = assets.to_vec();
        self
    }

    pub fn calculate(self) -> Fee {
        let mut fee = self.tx_calculator.calcluate();
        fee.for_assets = Some(self.per_asset_fee * MetaAsset::count(&self.assets));
        fee
    }
}

impl AssetCalculator<Asset> {
    pub fn assets(mut self, assets: &Vec<Asset>) -> Self {
        self.assets = assets.to_vec();
        self
    }

    pub fn calculate(self) -> Fee {
        let mut fee = self.tx_calculator.calcluate();
        fee.for_assets = Some(self.per_asset_fee * Asset::count(&self.assets));
        fee
    }

    pub fn trade_calculator(self) -> TradeCalculator {
        TradeCalculator::new(self)
    }
}

struct TradeCalculator {
    asset_calculator: AssetCalculator<Asset>,
    trade_fee: u64,
}

impl TradeCalculator {
    pub fn new(asset_calc: AssetCalculator<Asset>) -> Self {
        TradeCalculator { asset_calculator: asset_calc, trade_fee: 0 }
    }

    pub fn trade_fee(mut self, fee: u64) -> Self {
        self.trade_fee = fee;
        self
    }

    pub fn calcluate(self) -> Fee {
        let mut fee = self.asset_calculator.calculate();
        fee.for_trade = Some(self.trade_fee);
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