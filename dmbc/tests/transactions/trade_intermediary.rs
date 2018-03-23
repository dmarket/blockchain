extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::messages::Message;

use dmbc::currency::transactions::builders::fee;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::transactions::components::FeeStrategy;
// use dmbc::currency::error::Error;

use transactions::*;

fn get_trade_fee(t: u64, r:u64) -> Fees {
    fee::Builder::new()
        .trade(t, r)
        .exchange(0, 0)
        .transfer(0, 0)
        .build()
}

#[test]
fn trade_interemedary() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_trade_fee = 100;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, transaction_trade_fee, 0));

    let meta_data = "asset";
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, 2, get_trade_fee(10, 10))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (intermediary_public_key, intermediary_secret_key) = WalletMiner::new().mine(&mut testkit);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(50)
        .add_asset(meta_data, 2, 1000)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .data_info("trade_test")
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);

    let seller_wallet = get_wallet(&api, &seller_public_key);
    let buyer_wallet = get_wallet(&api, &buyer_public_key);
    assert!(seller_wallet.assets().is_empty());
    assert!(!buyer_wallet.assets().is_empty());
}

#[test]
fn trade_intermediary_fee_strategy() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_fee = 100;
    let units = 2;
    let price_per_unit = 1000;
    let commission = 50;
    let trade_tax = 0;
    let trade_ratio = 1;

    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, transaction_fee, 0));

    // Recipient pays
    let meta_data = "asset";
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, get_trade_fee(trade_tax, trade_ratio))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (intermediary_public_key, intermediary_secret_key) = WalletMiner::new().mine_empty();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(commission)
        .add_asset(meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(1)
        .data_info("trade_test")
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);

    let seller_wallet = get_wallet(&api, &seller_public_key);
    let buyer_wallet = get_wallet(&api, &buyer_public_key);
    let intermediary_wallet = get_wallet(&api, &intermediary_public_key);
    
    let asset_price = units * price_per_unit;
    let trade_fee = trade_tax + asset_price / trade_ratio;
    let seller_expected_balance = DMC_1 + asset_price + trade_fee;
    let buyer_expected_balance = DMC_1 - asset_price - transaction_fee - commission - trade_fee;
    let intermediary_expected_balance = commission;

    assert!(seller_wallet.assets().is_empty());
    assert_eq!(seller_wallet.balance(), seller_expected_balance);

    assert!(!buyer_wallet.assets().is_empty());
    assert_eq!(buyer_wallet.balance(), buyer_expected_balance);

    assert_eq!(intermediary_wallet.balance(), intermediary_expected_balance);

    // Sender pays fee
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, get_trade_fee(trade_tax, trade_ratio))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (intermediary_public_key, intermediary_secret_key) = WalletMiner::new().mine_empty();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(commission)
        .add_asset(meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Sender)
        .seed(1)
        .data_info("trade_test")
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);

    let seller_wallet = get_wallet(&api, &seller_public_key);
    let buyer_wallet = get_wallet(&api, &buyer_public_key);
    let intermediary_wallet = get_wallet(&api, &intermediary_public_key);

    let asset_price = units * price_per_unit;
    let seller_expected_balance = DMC_1 + asset_price - commission - transaction_fee;
    let buyer_expected_balance = DMC_1 - asset_price;
    let intermediary_expected_balance = commission;

    assert!(seller_wallet.assets().is_empty());
    assert_eq!(seller_wallet.balance(), seller_expected_balance);
    
    assert!(!buyer_wallet.assets().is_empty());
    assert_eq!(buyer_wallet.balance(), buyer_expected_balance);

    assert_eq!(intermediary_wallet.balance(), intermediary_expected_balance);

    // Both pay fee
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, get_trade_fee(trade_tax, trade_ratio))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (intermediary_public_key, intermediary_secret_key) = WalletMiner::new().mine_empty();

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(commission)
        .add_asset(meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(1)
        .data_info("trade_test")
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);

    let seller_wallet = get_wallet(&api, &seller_public_key);
    let buyer_wallet = get_wallet(&api, &buyer_public_key);
    let intermediary_wallet = get_wallet(&api, &intermediary_public_key);

    let asset_price = units * price_per_unit;
    let trade_fee = trade_tax + asset_price / trade_ratio;
    let seller_expected_balance = DMC_1 + asset_price + trade_fee/2 - transaction_fee/2 - commission/2;
    let buyer_expected_balance = DMC_1 - asset_price - trade_fee/2 - transaction_fee/2 - commission/2;
    let intermediary_expected_balance = commission;

    assert!(seller_wallet.assets().is_empty());
    assert_eq!(seller_wallet.balance(), seller_expected_balance);
    
    assert!(!buyer_wallet.assets().is_empty());
    assert_eq!(buyer_wallet.balance(), buyer_expected_balance);

    assert_eq!(intermediary_wallet.balance(), intermediary_expected_balance);

    // Invalid fee strategy
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, units, get_trade_fee(trade_tax, trade_ratio))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);
    let (intermediary_public_key, intermediary_secret_key) = WalletMiner::new().mine(&mut testkit);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets_with_intermediary()
        .intermediary_key_pair(intermediary_public_key, intermediary_secret_key)
        .commission(commission)
        .add_asset(meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .seed(1)
        .data_info("trade_test")
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);

    let seller_wallet = get_wallet(&api, &seller_public_key);
    let buyer_wallet = get_wallet(&api, &buyer_public_key);
    let intermediary_wallet = get_wallet(&api, &intermediary_public_key);

    let asset_price = units * price_per_unit;
    let trade_fee = trade_tax + asset_price / trade_ratio;
    let seller_expected_balance = DMC_1 + asset_price;
    let buyer_expected_balance = DMC_1 - asset_price;
    let intermediary_expected_balance = DMC_1 - transaction_fee;

    assert!(seller_wallet.assets().is_empty());
    assert_eq!(seller_wallet.balance(), seller_expected_balance);
    
    assert!(!buyer_wallet.assets().is_empty());
    assert_eq!(buyer_wallet.balance(), buyer_expected_balance);

    assert_eq!(intermediary_wallet.balance(), intermediary_expected_balance);
}