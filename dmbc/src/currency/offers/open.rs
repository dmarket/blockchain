use exonum::crypto::PublicKey;
use exonum::crypto::Hash;

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
    pub fn insert(&mut self, offer: Offer) -> &mut Self
    {
        let mut o = self.offers();
        o.push(offer);
        *self = Offers::new(self.price(), o);

        self
    }

    pub fn close(&mut self, amount: u64) -> Vec<CloseOffer>
    {
        let mut closed_offers: Vec<CloseOffer> = vec![];
        let mut amount_closed = 0u64;
        let mut offers = self.offers();
        for k in 0..offers.len() {
            if amount - amount_closed > 0 && offers[k].amount() > amount - amount_closed {
                offers[k].remove_amount(amount - amount_closed);
                closed_offers.push(CloseOffer{
                    wallet: offers[k].wallet().clone(),
                    price: self.price(),
                    amount: amount - amount_closed,
                });
                amount_closed = amount;
            } else if offers[k].amount() <= amount - amount_closed {
                amount_closed += offers[k].amount();
                let a = offers[k].amount();
                offers[k].remove_amount(a);
                closed_offers.push(CloseOffer{
                    wallet: offers[k].wallet().clone(),
                    price: self.price(),
                    amount: a,
                });
            } else {
                break
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
    use exonum::crypto::gen_keypair;
    use exonum::crypto;
    use currency::offers::{Offers, Offer};
    use currency::offers::open::CloseOffer;


    #[test]
    fn offers_insert_offer()
    {
        let (wallet, _) = gen_keypair();
        let tx_hash = &crypto::hash("tx1".as_bytes());
        let amount = 10;
        let price = 12;
        let mut offers = Offers::new(price, vec![Offer::new(&wallet, amount, tx_hash)]);

        offers.insert(Offer::new(&wallet, amount, tx_hash));
        assert_eq!(vec![Offer::new(&wallet, amount, tx_hash), Offer::new(&wallet, amount, tx_hash)], offers.offers());
    }

    #[test]
    fn offers_close_offer(){
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
            CloseOffer{wallet, price, amount:1},
            CloseOffer{wallet, price, amount:3},
            CloseOffer{wallet, price, amount:1}
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
        } else if bids[bids.len()-1].price() > price {
            new_bids = bids;
            new_bids.push(Offers::new(price, vec![offer]));
        } else {
            for k in (0..bids.len()).rev() {
                if bids[k].price() == price {
                    bids[k].insert(offer);
                    new_bids = bids;
                    break
                }
                if bids[k].price() > price {
                    new_bids.extend_from_slice(&bids[0..k+1]);
                    new_bids.push(Offers::new(price, vec![offer]));
                    new_bids.extend_from_slice(&bids[k+1..]);
                    break
                }
            }
        }

        *self = OpenOffers::new(new_bids, self.asks());
    }

    pub fn close_bid(&mut self, price: u64, amount: u64) -> Vec<CloseOffer>
    {
        let mut closed_offers: Vec<CloseOffer> = vec![];
        if self.bids().len() == 0 && self.bids()[self.bids().len()-1].price() > price {
            return closed_offers;
        }
        let mut bids = self.bids();
        let mut current_amount = amount;
        for k in (0..bids.len()).rev() {
            if bids[k].price() > price {
                break
            } else {
                let mut offers = bids[k].close(current_amount);
                for o in offers.iter() {
                    current_amount -= o.amount;
                }
                closed_offers.append(&mut offers);
            }
        }

        *self = OpenOffers::new(bids, self.asks());

        closed_offers
    }
}

#[cfg(test)]
mod test_open_offers {
    use exonum::crypto::gen_keypair;
    use exonum::crypto;
//    use currency::assets::AssetId;
    use currency::offers::{OpenOffers, Offer};
    use currency::offers::open::CloseOffer;

    #[test]
    fn add_bid() {
        let (wallet, _) = gen_keypair();
        let mut bid_ask = OpenOffers::new(vec![], vec![]);

        let first_offer = Offer::new(&wallet, 3, &crypto::hash("tx1".as_bytes()));
        let first_offer_price = 30;

        bid_ask.add_bid(first_offer_price, first_offer.clone());
        println!("{:?}",bid_ask);

        assert_eq!(bid_ask.bids()[0].price(), first_offer_price);
        assert_eq!(bid_ask.bids()[0].offers(), vec![first_offer.clone()]);

        bid_ask.add_bid(first_offer_price, first_offer.clone());
        println!("{:?}",bid_ask);
        assert_eq!(bid_ask.bids()[0].price(), first_offer_price);
        assert_eq!(bid_ask.bids()[0].offers(), vec![first_offer.clone(),first_offer.clone()]);

        let second_offer = Offer::new(&wallet, 1, &crypto::hash("tx2".as_bytes()));
        let second_offer_price = 10;

        bid_ask.add_bid(second_offer_price, second_offer.clone());
        println!("{:?}",bid_ask);
        assert_eq!(bid_ask.bids()[1].price(), second_offer_price);
        assert_eq!(bid_ask.bids()[1].offers(), vec![second_offer.clone()]);
        assert_eq!(bid_ask.bids()[0].price(), first_offer_price);

        let third_offer = Offer::new(&wallet, 7, &crypto::hash("tx3".as_bytes()));
        let third_offer_price = 15;
        bid_ask.add_bid(third_offer_price, third_offer.clone());
        println!("{:?}",bid_ask);
        assert_eq!(bid_ask.bids()[2].price(), second_offer_price);
        assert_eq!(bid_ask.bids()[1].price(), third_offer_price);
        assert_eq!(bid_ask.bids()[1].offers(), vec![third_offer.clone()]);
        assert_eq!(bid_ask.bids()[0].price(), first_offer_price);

        bid_ask.add_bid(second_offer_price, second_offer.clone());
        assert_eq!(bid_ask.bids()[2].offers(), vec![second_offer.clone(), second_offer.clone()]);
    }

    #[test]
    fn close_bid() {
        let (wallet1, _) = gen_keypair();
        let (wallet2, _) = gen_keypair();
        let (wallet3, _) = gen_keypair();
        let mut bid_ask = OpenOffers::new(vec![], vec![]);

        let first_offer  = Offer::new(&wallet1, 3, &crypto::hash("tx1".as_bytes()));
        let second_offer = Offer::new(&wallet2, 1, &crypto::hash("tx2".as_bytes()));
        let third_offer  = Offer::new(&wallet3, 7, &crypto::hash("tx3".as_bytes()));

        let first_offer_price  = 30;
        let second_offer_price = 10;
        let third_offer_price  = 15;

        bid_ask.add_bid(first_offer_price,  first_offer.clone());
        bid_ask.add_bid(first_offer_price,  second_offer.clone());
        bid_ask.add_bid(second_offer_price, second_offer.clone());  //10 * 1 need coins
        bid_ask.add_bid(second_offer_price, third_offer.clone());   //10 * 7 need coins
        bid_ask.add_bid(third_offer_price,  third_offer.clone());   //15 * 2

        let need_coins:u64  = 10 * 1 + 10* 7 + 15 * 2;

        let buyer_price:u64 = 17;
        let buyer_amount:u64 = 10;

        let offers = bid_ask.close_bid(buyer_price, buyer_amount);

        let mut summary_assets:u64 = 0;
        let mut summary_coins:u64 = 0;
        for offer in offers.iter() {
            summary_coins += offer.price*offer.amount;
            summary_assets += offer.amount;
        }

        assert_eq!(buyer_amount, summary_assets);
        assert_eq!(need_coins, summary_coins);
    }
}

