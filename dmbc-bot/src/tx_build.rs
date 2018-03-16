//pub enum Transaction {
////    CreateWallet = 0,
//    AddAssets = 1,
//    DelAssets = 2,
//    Exchange = 3,
//    ExchangeWithIntermediary = 4,
//    TradeAssets = 5,
//    TradeAssetsWithIntermediary = 6,
//    Transfer = 7,
////    Mining = 8,
//}



impl Transaction {
    pub fn create(tx_n:TransactionNumber) -> String {
        match tx_n {
/*
            TransactionNumber::CreateWallet => {
                let (pk, sk) = crypto::gen_keypair();
                Wallet::new(pk, sk.clone());
                let tx = transaction::Builder::new()
                    .keypair(pk,  sk)
                    .tx_create_wallet()
                    .build();


                serialize(tx)
            }
*/

            TransactionNumber::AddAssets => {
                let amount = self.rng.gen_range(0, MAX_AMOUNT);
                let wallet = self.pick_wallet();
                let fees = fee::Builder::new()
                    .trade(10, 10)
                    .exchange(10, 10)
                    .transfer(10, 10)
                    .build();
                let asset = MetaAsset::new(&wallet.public, ASSET_NAME, amount, fees);
//                self.assets.push(Asset::from_meta_asset(&asset, wallet.public));
                let tx = transaction::Builder::new()
                    .keypair(wallet.public, wallet.secret.clone())
                    .tx_add_assets()
                    .add_asset_value(asset)
                    .build();

                serialize(tx)
            }

            TransactionNumber::DelAssets => {
                let wallet = Flooder::pick_wallet(&mut self);
                let asset = self.split_asset();
                let tx = transaction::Builder::new()
                    .keypair(wallet.0, wallet.1)
                    .tx_del_assets()
                    .add_asset_value(asset)
                    .build();

                serialize(tx)
            }
//
//            OpState::Exchange => {
//                let sender = self.pick_wallet();
//                let receiver = self.pick_wallet();
//
//                let s_asset = self.pick_asset();
//                let r_asset = self.pick_asset();
//
//                let tx = transaction::Builder::new()
//                    .keypair(sender.0, sender.1)
//                    .tx_exchange()
//                    .sender_add_asset_value(s_asset)
//                    .sender_value(9)
//                    .recipient(receiver.0)
//                    .recipient_add_asset_value(r_asset)
//                    .fee_strategy(1)
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::ExchangeWithIntermediary => {
//                let sender = self.pick_wallet();
//                let receiver = self.pick_wallet();
//                let intermediary = self.pick_wallet();
//
//                let s_asset = self.pick_asset();
//                let r_asset = self.pick_asset();
//
//                let tx = transaction::Builder::new()
//                    .keypair(sender.0, sender.1)
//                    .tx_exchange_with_intermediary()
//                    .intermediary_key_pair(intermediary.0, intermediary.1)
//                    .commision(10)
//                    .sender_add_asset_value(s_asset)
//                    .sender_value(9)
//                    .recipient(receiver.0)
//                    .recipient_add_asset_value(r_asset)
//                    .fee_strategy(1)
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::Mining => {
//                let wallet = self.pick_wallet();
//                let tx = transaction::Builder::new()
//                    .keypair(wallet.0, wallet.1)
//                    .tx_mining()
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::TradeAssets => {
//                let seller = self.pick_wallet();
//                let buyer = self.pick_wallet();
//                let asset = self.pick_asset();
//
//                let tx = transaction::Builder::new()
//                    .keypair(seller.0, seller.1)
//                    .tx_trade_assets()
//                    .buyer(buyer.0)
//                    .add_asset_value(asset.into_trade_asset(50))
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::TradeAssetsWithIntermediary => {
//                let seller = self.pick_wallet();
//                let buyer = self.pick_wallet();
//                let intermediary = self.pick_wallet();
//                let asset = self.pick_asset();
//
//                let tx = transaction::Builder::new()
//                    .keypair(seller.0, seller.1)
//                    .tx_trade_assets_with_intermediary()
//                    .buyer(buyer.0)
//                    .intermediary_key_pair(intermediary.0, intermediary.1)
//                    .commision(1_0000_0000)
//                    .add_asset_value(asset.into_trade_asset(50))
//                    .build();
//
//                serialize(tx)
//            }
//
//            OpState::Transfer => {
//                let sender = self.pick_wallet();
//                let receiver = self.pick_wallet();
//                let asset = self.pick_asset();
//                let coins = self.rng.gen_range(0, asset.amount() + 1);
//
//                let tx = transaction::Builder::new()
//                    .keypair(sender.0, sender.1)
//                    .tx_transfer()
//                    .recipient(receiver.0)
//                    .amount(coins as u64)
//                    .add_asset_value(asset)
//                    .build();
//
//                serialize(tx)
//            }
        }
    }
}