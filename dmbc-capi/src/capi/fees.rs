use std::ptr;
use std::str::FromStr;

use libc::c_char;

use assets::{Fee, Fees};
use capi::common::parse_str;
use decimal::UFract64;

use error::{Error, ErrorKind};

fn dmbc_fee_create(fixed: u64, fraction: *const c_char, error: *mut Error) -> Option<Fee> {
    let fraction_str = match parse_str(fraction) {
        Ok(fraction_str) => fraction_str,
        Err(err) => unsafe {
            if !error.is_null() {
                *error = err;
            }
            return None;
        },
    };

    let fraction = match UFract64::from_str(fraction_str) {
        Ok(fraction) => fraction,
        Err(err) => unsafe {
            if !error.is_null() {
                *error = Error::new(ErrorKind::Text(err.to_string()));
            }
            return None;
        },
    };

    Some(Fee::new(fixed, fraction))
}

ffi_fn! {
    fn dmbc_fees_create(
        trade_fixed: u64,
        trade_fraction: *const c_char,
        exchnage_fixed: u64,
        exchange_fraction: *const c_char,
        transfer_fixed: u64,
        transfer_fraction: *const c_char,
        error: *mut Error
    ) -> *mut Fees {
        let trade = dmbc_fee_create(trade_fixed, trade_fraction, error);
        let exchange = dmbc_fee_create(exchnage_fixed, exchange_fraction, error);
        let transfer = dmbc_fee_create(transfer_fixed, transfer_fraction, error);

        if trade.is_none() || exchange.is_none() || transfer.is_none() {
            return ptr::null_mut()
        }

        Box::into_raw(
            Box::new(
                Fees::new(
                    trade.unwrap(),
                    exchange.unwrap(),
                    transfer.unwrap()
                )
            )
        )
    }
}

ffi_fn! {
    fn dmbc_fees_free(fees: *const c_char) {
        if !fees.is_null() {
            unsafe { Box::from_raw(fees as *mut Fees); }
        }
    }
}
