use exonum::crypto::PublicKey;
use currency::transactions;
use currency::service::{CONFIGURATION, PERMISSIONS};

/// Transaction permission mask
/// 0 - Tx not Allowd, 1 - Tx allowed
pub const TRANSFER_MASK:                 u64 =           0b_1;
pub const TRANSFER_WITH_FEES_PAYER_MASK: u64 =          0b_10;
pub const ADD_ASSETS_MASK:               u64 =         0b_100;
pub const DELETE_ASSETS_MASK:            u64 =        0b_1000;
pub const TRADE_MASK:                    u64 =       0b_10000;
pub const TRADE_INTERMEDIARY_MASK:       u64 =      0b_100000;
pub const EXCHANGE_MASK:                 u64 =     0b_1000000;
pub const EXCHANGE_INTERMEDIARY_MASK:    u64 =    0b_10000000;
pub const BID_MASK:                      u64 =   0b_100000000;
pub const ASK_MASK:                      u64 =  0b_1000000000;
pub const ALL_ALLOWED_MASK:              u64 = <u64>::max_value();

pub fn mask_for(message_id: u16) -> u64 {
    match message_id {
        transactions::TRANSFER_ID => TRANSFER_MASK,
        transactions::TRANSFER_FEES_PAYER_ID => TRANSFER_WITH_FEES_PAYER_MASK,
        transactions::ADD_ASSETS_ID => ADD_ASSETS_MASK,
        transactions::DELETE_ASSETS_ID => DELETE_ASSETS_MASK,
        transactions::TRADE_ID => TRADE_MASK,
        transactions::TRADE_INTERMEDIARY_ID => TRADE_INTERMEDIARY_MASK,
        transactions::EXCHANGE_ID => EXCHANGE_MASK,
        transactions::EXCHANGE_INTERMEDIARY_ID => EXCHANGE_INTERMEDIARY_MASK,
        transactions::ASK_OFFER_ID => ASK_MASK,
        transactions::BID_OFFER_ID => BID_MASK,
        _ => panic!("Unimplemented permission mask!"),
    }
}

pub fn has_permission(mask: u64, transaction_mask: u64) -> bool {
    (mask & transaction_mask) != 0
}

pub fn is_authorized<'a>(message_id: u16, keys: Vec<&PublicKey>) -> bool
{
    let permissions = PERMISSIONS.read().unwrap();
    let global_mask = CONFIGURATION.read().unwrap().permissions().global_permission_mask();
    let tx_mask = mask_for(message_id);

    for key in keys {
        match permissions.get(key) {
            Some(mask) => {
                if !has_permission(*mask, tx_mask) {
                    return false;
                }
            },
            None => ()
        } 
    }
    return has_permission(global_mask, tx_mask);
}