use currency::offers::{Offer, Offers, CloseOffer};

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
        if bids.len() == 0 || bids[bids.len() - 1].price() > price {
            bids.push(Offers::new(price, vec![offer]));
        } else if bids[0].price() < price {
            bids.insert(0, Offers::new(price, vec![offer]));
        } else {
            for k in (0..bids.len()).rev() {
                if bids[k].price() == price {
                    bids[k].insert(offer);
                    break;
                }
                if bids[k].price() > price {
                    bids.insert(k+1, Offers::new(price, vec![offer]));
                    break;
                }
            }
        }

        *self = OpenOffers::new(bids, self.asks());
    }

    pub fn add_ask(&mut self, price: u64, offer: Offer)
    {
        let mut asks = self.asks();
        if asks.len() == 0 || asks[asks.len() - 1].price() < price{
            asks.push(Offers::new(price, vec![offer]));
        } else if asks[0].price() > price {
            asks.insert(0, Offers::new(price, vec![offer]));
        } else {
            for k in (0..asks.len()).rev() {
                if asks[k].price() == price {
                    asks[k].insert(offer);
                    break;
                }
                if asks[k].price() < price {
                    asks.insert(k+1, Offers::new(price, vec![offer]));
                    break;
                }
            }
        }

        *self = OpenOffers::new(self.bids(), asks);
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
                for o in &offers {
                    current_amount -= o.amount;
                }

                if bids[k].offers().len() == 0 {
                    bids.pop();
                }

                closed_offers.append(&mut offers);
            }
        }

        *self = OpenOffers::new(bids, self.asks());

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
                for o in &offers {
                    current_amount -= o.amount;
                }
                if asks[k].offers().len() == 0 {
                    asks.pop();
                }
                closed_offers.append(&mut offers);
            }
        }

        *self = OpenOffers::new(self.bids(), asks);

        closed_offers
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

