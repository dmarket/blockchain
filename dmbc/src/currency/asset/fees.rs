encoding_struct! {
    struct Fee {
        const SIZE = 16;

        field tax:   u64  [0 => 8]
        field ratio: u64  [8 => 16]
    }
}

encoding_struct! {
    struct Fees {
        const SIZE = 24;

        field trade:    Fee [ 0 => 8]
        field exchange: Fee [ 8 => 16]
        field transfer: Fee [16 => 24]
    }
}

impl Fee {
    pub fn for_price(&self, price: u64) -> u64 {
        let price_ratio = price / self.ratio();
        self.tax() + price_ratio
    }
}

