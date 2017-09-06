extern crate exonum;

//use std::collections::HashMap;
use exonum::crypto::PublicKey;
//use exonum::encoding::segments::SegmentField;
use exonum::encoding::Field;
//use exonum::encoding::{Field, Offset, CheckedOffset, Result};
//
//impl<'a> SegmentField<'a> for &'a HashMap<&str, u32> {
//    fn item_size() -> Offset {
//        ::std::mem::size_of::<HashMap<&str, u32>>() as Offset
//    }
//
//    fn count(&self) -> Offset {
//        self.len() as Offset
//    }
//
//    unsafe fn from_buffer(buffer: &'a [u8], from: Offset, count: Offset) -> Self {
//        let to = from + count * Self::item_size();
//        let slice = &buffer[(from as usize)..(to as usize)];
//        ::std::slice::from_raw_parts(slice.as_ptr() as *const Hash,
//                                     slice.len() / Self::item_size() as usize)
//    }
//
//    fn extend_buffer(&self, buffer: &mut Vec<u8>) {
//        let slice = unsafe {
//            ::std::slice::from_raw_parts(self.as_ptr() as *const u8,
//                                         self.len() * Self::item_size() as usize)
//        };
//        buffer.extend_from_slice(slice)
//    }
//
//    fn check_data(_: &'a [u8],
//                  _: CheckedOffset,
//                  _: CheckedOffset,
//                  latest_segment: CheckedOffset) -> Result {
//        Ok(latest_segment)
//    }
//}
//

encoding_struct!{
    struct Asset {
        const SIZE = 12;

        field hash_id:    &str      [00 => 08]
        field amount:      u32      [08 => 12]
    }
}

encoding_struct! {
    struct Wallet {
        const SIZE = 48;

        field pub_key:            &PublicKey          [00 => 32]
        field balance:            u64                 [32 => 40]
        field assets:             Vec<Asset>          [40 => 48]
    }
}

impl Wallet {
    pub fn increase(&mut self, amount: u64) {
        let balance = self.balance() + amount;
        Field::write(&balance, &mut self.raw, 32, 40);
    }

    pub fn decrease(&mut self, amount: u64) {
        let balance = self.balance() - amount;
        Field::write(&balance, &mut self.raw, 32, 40);
    }

    pub fn add_assets(&mut self, asset: Asset) {
        let mut assets = self.assets();

//        let r = assets
//            .into_iter()
//            .filter(|el| { el.hash_id == asset.hash_id })
//            .map(|&mut el| { el.amount += asset.amount });

//        if r.len() == 0 { assets.push(asset); }

        let mut is_add = false;

        for i in 0..assets.len() {
            if assets[i].hash_id() == asset.hash_id() {
                let amount = asset.amount() + assets[i].amount();
                assets[i] = Asset::new(asset.hash_id(), amount);
                is_add = true;
                break;
            }
        }

        if !is_add {
            assets.push(asset);
        }
        Field::write( &assets, &mut self.raw, 40, 48);
    }

//    pub fn del_assets(&mut self, asset: Asset) -> bool {
//        let assets = self.assets();
//
//        for i in 0..assets.len() {
//            if assets[i].hash_id() == asset.hash_id() && assets[i].amount() > asset.amount() {
//                assets[i].amount -= asset.amount();
//                Field::write( &assets, &mut self.raw, 40, 48);
//                return true;
//            }
//        }
//
//        false
//    }
}

