use currency::transactions;

/// Transaction permission mask
/// 0 - Tx not Allowd, 1 - Tx allowed
pub const PM_TRANSFER:                 u64 =           0b_1;
pub const PM_TRANSFER_WITH_FEES_PAYER: u64 =          0b_10;
pub const PM_ADD_ASSETS:               u64 =         0b_100;
pub const PM_DELETE_ASSETS:            u64 =        0b_1000;
pub const PM_TRADE:                    u64 =       0b_10000;
pub const PM_TRADE_INTERMEDIARY:       u64 =      0b_100000;
pub const PM_EXCHANGE:                 u64 =     0b_1000000;
pub const PM_EXCHANGE_INTERMEDIARY:    u64 =    0b_10000000;
pub const PM_BID:                      u64 =   0b_100000000;
pub const PM_ASK:                      u64 =  0b_1000000000;
pub const PM_ALL_ALLOWED:              u64 = <u64>::max_value();

pub fn mask_for(message_id: u16) -> u64 {
    match message_id {
        transactions::TRANSFER_ID => PM_TRANSFER,
        transactions::TRANSFER_FEES_PAYER_ID => PM_TRANSFER_WITH_FEES_PAYER,
        transactions::ADD_ASSETS_ID => PM_ADD_ASSETS,
        transactions::DELETE_ASSETS_ID => PM_DELETE_ASSETS,
        transactions::TRADE_ID => PM_TRADE,
        transactions::TRADE_INTERMEDIARY_ID => PM_TRADE_INTERMEDIARY,
        transactions::EXCHANGE_ID => PM_EXCHANGE,
        transactions::EXCHANGE_INTERMEDIARY_ID => PM_EXCHANGE_INTERMEDIARY,
        _ => panic!("Unimplemented permission mask!"),
    }
}

pub fn has_permission(mask: u64, transaction_mask: u64) -> bool {
    (mask & transaction_mask) != 0
}

pub trait Permissions {
    fn is_authorized(&self) -> bool;
}