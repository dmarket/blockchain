extern crate dmbc;
extern crate exonum;
extern crate exonum_testkit;

use exonum::messages::Message;

use dmbc::currency::transactions::builders::fee;
use dmbc::currency::configuration::TransactionFees;
use dmbc::currency::transactions::components::FeeStrategy;
use dmbc::currency::error::Error;

use transactions::*;

fn trade_fee(t: u64, r:u64) -> Fees {
    fee::Builder::new()
        .trade(t, r)
        .exchange(0, 0)
        .transfer(0, 0)
        .build()
}

#[test]
fn trade() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_trade_fee = 100;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, transaction_trade_fee, 0));

    let meta_data = "asset";
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, 2, trade_fee(10, 10))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let units = 2;
    let price_per_unit = 1000;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
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
fn trade_fee_strategy() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_trade_fee = 100;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, transaction_trade_fee, 0));

    // Recipient pays
    let meta_data = "asset";
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, 2, trade_fee(50, 50))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let units = 2;
    let price_per_unit = 1000;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);

    let seller_wallet = get_wallet(&api, &seller_public_key);
    let buyer_wallet = get_wallet(&api, &buyer_public_key);
    let seller_expected_balance = DMC_1 + units * price_per_unit;
    let buyer_expected_balance = DMC_1 - units * price_per_unit - transaction_trade_fee;

    assert!(seller_wallet.assets().is_empty());
    assert_eq!(seller_wallet.balance(), seller_expected_balance);

    assert!(!buyer_wallet.assets().is_empty());
    assert_eq!(buyer_wallet.balance(), buyer_expected_balance);

    // Sender pays fee
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, 2, trade_fee(50, 50))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Sender)
        .seed(12)
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);

    let seller_wallet = get_wallet(&api, &seller_public_key);
    let buyer_wallet = get_wallet(&api, &buyer_public_key);

    let seller_expected_balance = DMC_1 + units * price_per_unit - transaction_trade_fee;
    let buyer_expected_balance = DMC_1 - units * price_per_unit;

    assert!(seller_wallet.assets().is_empty());
    assert_eq!(seller_wallet.balance(), seller_expected_balance);
    
    assert!(!buyer_wallet.assets().is_empty());
    assert_eq!(buyer_wallet.balance(), buyer_expected_balance);

    // Both pay fee
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, 2, trade_fee(50, 50))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::RecipientAndSender)
        .seed(12)
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);

    let seller_wallet = get_wallet(&api, &seller_public_key);
    let buyer_wallet = get_wallet(&api, &buyer_public_key);

    let seller_expected_balance = DMC_1 + units * price_per_unit - transaction_trade_fee / 2;
    let buyer_expected_balance = DMC_1 - units * price_per_unit - transaction_trade_fee / 2;

    assert!(seller_wallet.assets().is_empty());
    assert_eq!(seller_wallet.balance(), seller_expected_balance);
    
    assert!(!buyer_wallet.assets().is_empty());
    assert_eq!(buyer_wallet.balance(), buyer_expected_balance);

    // Invalid fee strategy
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, 2, trade_fee(50, 50))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Intermediary)
        .seed(12)
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Ok(())), s);
}

#[test]
fn trade_insuffisient_funds() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_trade_fee = 100;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, transaction_trade_fee, 0));

    let meta_data = "asset";
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, 2, trade_fee(10, 10))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine_empty();

    let units = 2;
    let price_per_unit = 1000;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);
}

#[test]
fn trade_assets_not_found() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_trade_fee = 100;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, transaction_trade_fee, 0));

    let meta_data = "asset";
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let units = 2;
    let price_per_unit = 1000;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Err(Error::AssetNotFound)), s);
}

#[test]
fn trade_insuffisient_assets() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_trade_fee = 100;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, transaction_trade_fee, 0));

    let meta_data = "asset";
    let (seller_public_key, seller_secret_key) = WalletMiner::new()
        .add_asset(meta_data, 2, trade_fee(10, 10))
        .mine(&mut testkit);

    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let units = 10;
    let price_per_unit = 1000;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Err(Error::InsufficientAssets)), s);
}


#[test]
fn trade_insuffisient_funds_for_execution() {
    let mut testkit = init_testkit();
    let api = testkit.api();
    let transaction_trade_fee = 100;
    set_configuration(&mut testkit, TransactionFees::new(0, 0, 0, 0, transaction_trade_fee, 0));

    let meta_data = "asset";
    let (seller_public_key, seller_secret_key) = WalletMiner::new().mine_empty();
    let (buyer_public_key, buyer_secret_key) = WalletMiner::new().mine(&mut testkit);

    let units = 10;
    let price_per_unit = 1000;

    let tx_trade = transaction::Builder::new()
        .keypair(buyer_public_key, buyer_secret_key)
        .tx_trade_assets()
        .add_asset(&meta_data, units, price_per_unit)
        .seller(seller_public_key, seller_secret_key)
        .fee_strategy(FeeStrategy::Recipient)
        .seed(12)
        .build();

    post_tx(&api, &tx_trade);
    testkit.create_block();

    let s = get_status(&api, &tx_trade.hash());
    assert_eq!(Ok(Err(Error::InsufficientFunds)), s);
}