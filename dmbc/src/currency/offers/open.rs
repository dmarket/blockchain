use exonum::crypto::Hash;
use exonum::crypto::PublicKey;

/*
json tables Bid,Ask:

{
    "assetId" : [
        {
            "price": "10000",                                       - dmc
            "offers" : [
                { "amount": "7", "wallet":"wallet1", "tx_hash":"tx1" },              - fist transaction with price 10000
                { "amount": "3", "wallet":"wallet2", "tx_hash":"tx2" },              - second transaction with price 10000
                { "amount": "6", "wallet":"wallet1", "tx_hash":"tx3" },              - third transaction with price 10000
            ]
        },
        {
            "price": "10200",                                       - dmc
            "offers" : [
                { "amount": "5", "wallet":"wallet1", "tx_hash":"tx4"  },              - fist transaction with price 10200
                { "amount": "3", "wallet":"wallet2", "tx_hash":"tx5"  },              - second transaction with price 10200
            ]
        }
    ]
}
*/


encoding_struct! {
    #[derive(Eq, PartialOrd, Ord)]
    struct Offer {
        wallet: &PublicKey,
        amount: u64,
        tx_hash: &Hash,
    }
}

impl Offer {
    pub fn remove_amount(&mut self, amount: u64) {
        *self = Offer::new(self.wallet(), self.amount() - amount, &self.tx_hash());
    }

    pub fn add_amount(&mut self, amount: u64) {
        *self = Offer::new(self.wallet(), self.amount() + amount, &self.tx_hash());
    }
}

encoding_struct! {
    #[derive(Eq, PartialOrd, Ord)]
    struct Offers {
        price:   u64,
        offers:  Vec<Offer>,
    }
}
#[derive(Debug, Eq, PartialEq)]
pub struct CloseOffer {
    pub wallet: PublicKey,
    pub price: u64,
    pub amount: u64,
}

impl Offers {
    pub fn insert(&mut self, offer: Offer)
    {
        let mut o = self.offers();
        if o.len() == 0 {
            *self = Offers::new(self.price(), vec![offer]);
            return;
        }
        let n = o.len() - 1;

        if o[n].wallet() == offer.wallet() && o[n].tx_hash() == offer.tx_hash() {
            o[n].add_amount(offer.amount());
        } else {
            o.push(offer);
        }

        *self = Offers::new(self.price(), o);
    }

    pub fn close(&mut self, amount: u64) -> Vec<CloseOffer>
    {
        let mut closed_offers: Vec<CloseOffer> = vec![];
        let mut amount_closed = 0u64;
        let mut offers = self.offers();
        for k in 0..offers.len() {
            if amount - amount_closed > 0 && offers[k].amount() > amount - amount_closed {
                offers[k].remove_amount(amount - amount_closed);
                closed_offers.push(CloseOffer {
                    wallet: offers[k].wallet().clone(),
                    price: self.price(),
                    amount: amount - amount_closed,
                });
                amount_closed = amount;
            } else if offers[k].amount() <= amount - amount_closed {
                amount_closed += offers[k].amount();
                let a = offers[k].amount();
                offers[k].remove_amount(a);
                closed_offers.push(CloseOffer {
                    wallet: offers[k].wallet().clone(),
                    price: self.price(),
                    amount: a,
                });
            } else {
                break;
            }
        }
        self.clear_empty_offers(offers);

        closed_offers
    }

    fn clear_empty_offers(&mut self, offers: Vec<Offer>)
    {
        let mut o: Vec<Offer> = vec![];
        for offer in offers {
            if offer.amount() > 0 {
                o.push(offer)
            }
        }
        *self = Offers::new(self.price(), o);
    }
}

#[cfg(test)]
mod test {
    use currency::offers::{Offer, Offers};
    use currency::offers::open::CloseOffer;
    use exonum::crypto;
    use exonum::crypto::gen_keypair;


    #[test]
    fn offers_insert_offer()
    {
        let (wallet, _) = gen_keypair();
        let tx_hash = &crypto::hash("tx1".as_bytes());
        let amount = 10;
        let price = 12;
        let mut offers = Offers::new(price, vec![Offer::new(&wallet, amount, tx_hash)]);

        offers.insert(Offer::new(&wallet, amount, tx_hash));
        assert_eq!(vec![Offer::new(&wallet, 2 * amount, tx_hash)], offers.offers());
    }

    #[test]
    fn offers_close_offer() {
        let (wallet, _) = gen_keypair();
        let price = 10;
        let o = vec![
            Offer::new(&wallet, 1, &crypto::hash("tx1".as_bytes())),
            Offer::new(&wallet, 3, &crypto::hash("tx2".as_bytes())),
            Offer::new(&wallet, 5, &crypto::hash("tx3".as_bytes()))
        ];
        let mut bids = Offers::new(price, o);

        let result = bids.close(5);
        let cs = vec![
            CloseOffer { wallet, price, amount: 1 },
            CloseOffer { wallet, price, amount: 3 },
            CloseOffer { wallet, price, amount: 1 }
        ];
        assert_eq!(cs, result);
        assert_eq!(vec![Offer::new(&wallet, 4, &crypto::hash("tx3".as_bytes()))], bids.offers());
    }
}



encoding_struct! {
    struct OpenOffers {
        bids: Vec<Offers>,   //bids was sorted by price first max last min
        asks: Vec<Offers>,
    }
}

impl OpenOffers {
    pub fn new_open_offers() -> Self { OpenOffers::new(vec![], vec![]) }

    /// Push new bid into the BidAsk.
    pub fn add_bid(&mut self, price: u64, offer: Offer)
    {
        let mut bids = self.bids();
        let mut new_bids: Vec<Offers> = vec![];
        if bids.len() == 0 {
            new_bids.push(Offers::new(price, vec![offer]))
        } else if bids[bids.len() - 1].price() > price {
            new_bids = bids;
            new_bids.push(Offers::new(price, vec![offer]));
        } else if bids[0].price() < price {
            new_bids.push(Offers::new(price, vec![offer]));
            new_bids.append(&mut bids);
        } else {
            for k in (0..bids.len()).rev() {
                if bids[k].price() == price {
                    bids[k].insert(offer);
                    new_bids = bids;
                    break;
                }
                if bids[k].price() > price {
                    new_bids.extend_from_slice(&bids[0..k + 1]);
                    new_bids.push(Offers::new(price, vec![offer]));
                    new_bids.extend_from_slice(&bids[k + 1..]);
                    break;
                }
            }
        }

        *self = OpenOffers::new(new_bids, self.asks());
    }

    pub fn add_ask(&mut self, price: u64, offer: Offer)
    {
        let mut open_offers = self.asks();
        let mut new_open_offers: Vec<Offers> = vec![];
        if open_offers.len() == 0 {
            new_open_offers.push(Offers::new(price, vec![offer]));
        } else if open_offers[open_offers.len() - 1].price() < price {
            new_open_offers = open_offers;
            new_open_offers.push(Offers::new(price, vec![offer]));
        } else if open_offers[0].price() > price {
            new_open_offers.push(Offers::new(price, vec![offer]));
            new_open_offers.append(&mut open_offers);
        } else {
            for k in (0..open_offers.len()).rev() {
                if open_offers[k].price() == price {
                    open_offers[k].insert(offer);
                    new_open_offers = open_offers;
                    break;
                }
                if open_offers[k].price() < price {
                    new_open_offers.extend_from_slice(&open_offers[0..k + 1]);
                    new_open_offers.push(Offers::new(price, vec![offer]));
                    new_open_offers.extend_from_slice(&open_offers[k + 1..]);
                    break;
                }
            }
        }

        *self = OpenOffers::new(self.bids(), new_open_offers);
    }

    pub fn close_bid(&mut self, price: u64, amount: u64) -> Vec<CloseOffer>
    {
        let mut closed_offers: Vec<CloseOffer> = vec![];
        if self.bids().len() == 0 || self.bids()[self.bids().len() - 1].price() > price {
            return closed_offers;
        }
        let mut bids = self.bids();
        let mut current_amount = amount;
        for k in (0..bids.len()).rev() {
            if bids[k].price() > price {
                break;
            } else {
                let mut offers = bids[k].close(current_amount);
                for o in offers.iter() {
                    current_amount -= o.amount;
                }
                closed_offers.append(&mut offers);
            }
        }

        self.clear_empty_price(true, &mut bids);

        closed_offers
    }

    pub fn close_ask(&mut self, price: u64, amount: u64) -> Vec<CloseOffer>
    {
        let mut closed_offers: Vec<CloseOffer> = vec![];
        if self.asks().len() == 0 || self.asks()[self.asks().len() - 1].price() < price {
            return closed_offers;
        }
        let mut asks = self.asks();
        let mut current_amount = amount;
        for k in (0..asks.len()).rev() {
            if asks[k].price() < price {
                break;
            } else {
                let mut offers = asks[k].close(current_amount);
                for o in offers.iter() {
                    current_amount -= o.amount;
                }
                closed_offers.append(&mut offers);
            }
        }

        self.clear_empty_price(false, &mut asks);

        closed_offers
    }

    fn clear_empty_price(&mut self, bid: bool, offers: &mut Vec<Offers>)
    {
        for pos in (0..offers.len()).rev() {
            if offers[pos].offers().len() == 0 {
                let _o = offers.pop();
            } else {
                break;
            }
        }
        if bid {
            *self = OpenOffers::new(offers.to_vec(), self.asks());
        } else {
            *self = OpenOffers::new(self.bids(), offers.to_vec());
        }
    }
}

#[cfg(test)]
mod test_open_offers {
    use currency::offers::{Offer, OpenOffers};
    use exonum::crypto;
    use exonum::crypto::gen_keypair;

    #[test]
    fn add_bid() {
        let (wallet, _) = gen_keypair();
        let mut open_offers = OpenOffers::new(vec![], vec![]);

        let first_bid = Offer::new(&wallet, 3, &crypto::hash("tx1".as_bytes()));
        let first_bid_price = 30;

        open_offers.add_bid(first_bid_price, first_bid.clone());
        assert_eq!(open_offers.bids()[0].price(), first_bid_price);
        assert_eq!(open_offers.bids()[0].offers(), vec![first_bid.clone()]);

        let second_bid = Offer::new(&wallet, 1, &crypto::hash("tx2".as_bytes()));
        let second_bid_price = 10;

        open_offers.add_bid(second_bid_price, second_bid.clone());
        assert_eq!(open_offers.bids()[1].price(), second_bid_price);
        assert_eq!(open_offers.bids()[1].offers(), vec![second_bid.clone()]);
        assert_eq!(open_offers.bids()[0].price(), first_bid_price);

        let third_bid = Offer::new(&wallet, 7, &crypto::hash("tx3".as_bytes()));
        let third_bid_price = 15;
        open_offers.add_bid(third_bid_price, third_bid.clone());
        assert_eq!(open_offers.bids()[2].price(), second_bid_price);
        assert_eq!(open_offers.bids()[1].price(), third_bid_price);
        assert_eq!(open_offers.bids()[1].offers(), vec![third_bid.clone()]);
        assert_eq!(open_offers.bids()[0].price(), first_bid_price);

        open_offers.add_bid(second_bid_price, second_bid.clone());
        assert_eq!(open_offers.bids()[2].offers(), vec![Offer::new(&wallet, 2*1, &crypto::hash("tx2".as_bytes()))]);
    }

    #[test]
    fn add_ask() {
        let (wallet, _) = gen_keypair();
        let mut open_offers = OpenOffers::new(vec![], vec![]);

        let first_ask = Offer::new(&wallet, 3, &crypto::hash("tx1".as_bytes()));
        let first_ask_price = 30;

        open_offers.add_ask(first_ask_price, first_ask.clone());
        assert_eq!(open_offers.asks()[0].price(), first_ask_price);
        assert_eq!(open_offers.asks()[0].offers(), vec![first_ask.clone()]);

        let second_ask = Offer::new(&wallet, 1, &crypto::hash("tx2".as_bytes()));
        let second_ask_price = 10;

        open_offers.add_ask(second_ask_price, second_ask.clone());
        assert_eq!(open_offers.asks()[0].price(), second_ask_price);
        assert_eq!(open_offers.asks()[0].offers(), vec![second_ask.clone()]);
        assert_eq!(open_offers.asks()[1].price(), first_ask_price);

        let third_ask = Offer::new(&wallet, 7, &crypto::hash("tx3".as_bytes()));
        let third_ask_price = 15;
        open_offers.add_ask(third_ask_price, third_ask.clone());
        assert_eq!(open_offers.asks()[0].price(), second_ask_price);
        assert_eq!(open_offers.asks()[1].price(), third_ask_price);
        assert_eq!(open_offers.asks()[1].offers(), vec![third_ask.clone()]);
        assert_eq!(open_offers.asks()[2].price(), first_ask_price);

        open_offers.add_ask(second_ask_price, second_ask.clone());
        assert_eq!(open_offers.asks()[0].offers(), vec![Offer::new(&wallet, 2*1, &crypto::hash("tx2".as_bytes()))]);
    }

    #[test]
    fn close_bid() {
        let (wallet1, _) = gen_keypair();
        let (wallet2, _) = gen_keypair();
        let (wallet3, _) = gen_keypair();
        let mut open_offers = OpenOffers::new(vec![], vec![]);

        let first_bid = Offer::new(&wallet1, 3, &crypto::hash("tx1".as_bytes()));
        let second_bid = Offer::new(&wallet2, 1, &crypto::hash("tx2".as_bytes()));
        let third_bid = Offer::new(&wallet3, 7, &crypto::hash("tx3".as_bytes()));

        let first_bid_price = 30;
        let second_bid_price = 10;
        let third_bid_price = 15;

        let buyer_price: u64 = 17;
        let buyer_amount: u64 = 10;

        open_offers.add_bid(first_bid_price, first_bid.clone());
        open_offers.add_bid(first_bid_price, second_bid.clone());
        open_offers.add_bid(second_bid_price, second_bid.clone());  //10 * 1 need coins
        open_offers.add_bid(second_bid_price, third_bid.clone());   //10 * 7 need coins
        open_offers.add_bid(third_bid_price, third_bid.clone());   //15 * 2

        let need_coins: u64 =
            second_bid_price * second_bid.amount() +
                second_bid_price * third_bid.amount() +
                third_bid_price * (buyer_amount - second_bid.amount() - third_bid.amount());

        let closed_offers = open_offers.close_bid(buyer_price, buyer_amount);

        let mut summary_assets: u64 = 0;
        let mut summary_coins: u64 = 0;
        for offer in closed_offers.iter() {
            summary_coins += offer.price * offer.amount;
            summary_assets += offer.amount;
        }

        let mut sample_offers = OpenOffers::new(vec![], vec![]);
        sample_offers.add_bid(first_bid_price, first_bid.clone());
        sample_offers.add_bid(first_bid_price, second_bid.clone());
        sample_offers.add_bid(third_bid_price, Offer::new(&wallet3, 5, &crypto::hash("tx3".as_bytes())));

        assert_eq!(sample_offers, open_offers);
        assert_eq!(buyer_amount, summary_assets);
        assert_eq!(need_coins, summary_coins);
    }

    //todo: добавить тест для close_bid и для close_ask когда нет других оферов по данному ассету

    #[test]
    fn close_ask() {
        let (wallet1, _) = gen_keypair();
        let (wallet2, _) = gen_keypair();
        let (wallet3, _) = gen_keypair();
        let mut open_offers = OpenOffers::new(vec![], vec![]);

        let first_ask = Offer::new(&wallet1, 3, &crypto::hash("tx1".as_bytes()));
        let second_ask = Offer::new(&wallet2, 1, &crypto::hash("tx2".as_bytes()));
        let third_ask = Offer::new(&wallet3, 7, &crypto::hash("tx3".as_bytes()));

        let first_ask_price = 30;
        let second_ask_price = 10;
        let third_ask_price = 15;

        let seller_price: u64 = 17;
        let seller_amount: u64 = 10;

        open_offers.add_ask(first_ask_price, first_ask.clone());
        open_offers.add_ask(first_ask_price, second_ask.clone());
        open_offers.add_ask(second_ask_price, second_ask.clone());
        open_offers.add_ask(second_ask_price, third_ask.clone());
        open_offers.add_ask(third_ask_price, third_ask.clone());

        let need_amount: u64 = first_ask.amount() + second_ask.amount();
        let need_coins: u64 = first_ask_price * need_amount;

        let closed_offers = open_offers.close_ask(seller_price, seller_amount);

        let mut summary_amount: u64 = 0;
        let mut summary_coins: u64 = 0;
        for offer in closed_offers.iter() {
            summary_coins += offer.price * offer.amount;
            summary_amount += offer.amount;
        }

        let mut sample_offers = OpenOffers::new(vec![], vec![]);
        sample_offers.add_ask(second_ask_price, second_ask.clone());
        sample_offers.add_ask(second_ask_price, third_ask.clone());
        sample_offers.add_ask(third_ask_price, third_ask.clone());

        assert_eq!(sample_offers, open_offers);
        assert_eq!(need_amount, summary_amount);
        assert_eq!(need_coins, summary_coins);
    }
}

