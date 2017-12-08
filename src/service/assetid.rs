

encoding_struct! {
    struct AssetID {
        const SIZE = 16;

        field high_u64: u64 [0 => 8]
        field low_u64: u64 [8 => 16]
    }
}

/// Error details for string parsing failures.
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ParseError {
    InvalidLength(usize),
    InvalidString(),
}

impl AssetID {
    pub fn nil() -> AssetID {
        AssetID::new(0, 0)
    }

    pub fn from_bytes(b: &[u8]) -> Result<AssetID, ParseError> {
        let len = b.len();
        if len != 16 {
            return Err(ParseError::InvalidLength(len));
        }

        let mut high: u64 = 0u64;
        let mut low: u64 = 0u64;

        for i in 0..8 {
            let byte = (b[i] as u64) << i * 2;
            high |= byte;
        }

        for i in 8..16 {
            let byte = (b[i] as u64) << (i - 8) * 2;
            low |= byte;
        }

        let asset_id = AssetID::new(high, low);
        Ok(asset_id)
    }
}

impl ToString for AssetID {
    fn to_string(&self) -> String {
        format!("{:16x}", self.high_u64()) + &format!("{:16x}", self.low_u64())
    }
}

impl AssetID {
    pub fn from_str(us: &str) -> Result<AssetID, ParseError> {
        let len = us.len();
        if len != 32 {
            return Err(ParseError::InvalidLength(len));
        }

        let high_u64 = u64::from_str_radix(&us[0..16], 16);
        let low_u64 = u64::from_str_radix(&us[16..32], 16);

        if high_u64.is_ok() && low_u64.is_ok() {
            let asset_id = AssetID::new(high_u64.unwrap(), low_u64.unwrap());
            return Ok(asset_id);
        } else {
            return Err(ParseError::InvalidString());
        }
    }
}
