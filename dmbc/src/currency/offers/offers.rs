use currency::offers::Offer;
use exonum::crypto::{PublicKey, Hash};

#[derive(Debug, Eq, PartialEq)]
pub struct CloseOffer {
    pub wallet: PublicKey,
    pub price: u64,
    pub amount: u64,
    pub tx_hash: Hash,
}

encoding_struct! {
    #[derive(Eq, PartialOrd, Ord)]
    struct Offers {
        price:   u64,
        offers:  Vec<Offer>,
    }
}

impl Offers {
    pub fn insert(&mut self, offer: Offer)
    {
        let mut offers = self.offers();
        if offers.len() > 0 {
            let last = offers.iter().last().unwrap();

            if last.wallet() == offer.wallet() && last.tx_hash() == offer.tx_hash() {
                eprintln!("last = {:#?}", last);
                eprintln!("offer = {:#?}", offer);
                panic!("last offer equal add offer");
            }
        }

        offers.push(offer);

        *self = Offers::new(self.price(), offers);
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
                    tx_hash: offers[k].tx_hash().clone(),
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
                    tx_hash: offers[k].tx_hash().clone(),
                });
            } else {
                break;
            }
        }
        offers.retain(|o|o.amount() > 0);

        *self = Offers::new(self.price(), offers);

        closed_offers
    }
}




#[cfg(test)]
mod test {
    use currency::offers::{CloseOffer, Offer, Offers};
    use exonum::crypto;
    use exonum::crypto::gen_keypair;

    #[test]
    #[should_panic]
    fn offers_insert_offer_panic()
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
    fn offers_insert_offer()
    {
        let (wallet, _) = gen_keypair();
        let tx_hash1 = &crypto::hash("tx1".as_bytes());
        let tx_hash2 = &crypto::hash("tx2".as_bytes());

        let amount = 10;
        let price = 12;
        let mut offers = Offers::new(price, vec![]);
        offers.insert(Offer::new(&wallet, amount, tx_hash1));
        assert_eq!(vec![Offer::new(&wallet, amount, tx_hash1)], offers.offers());

        offers.insert(Offer::new(&wallet, amount, tx_hash2));
        assert_eq!(vec![Offer::new(&wallet, amount, tx_hash1), Offer::new(&wallet, amount, tx_hash2)], offers.offers());
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
        let hash1 = crypto::hash("tx1".as_bytes());
        let hash2 = crypto::hash("tx2".as_bytes());
        let hash3 = crypto::hash("tx3".as_bytes());

        let cs = vec![
            CloseOffer { wallet, price, amount: 1, tx_hash: hash1 },
            CloseOffer { wallet, price, amount: 3, tx_hash: hash2 },
            CloseOffer { wallet, price, amount: 1, tx_hash: hash3 }
        ];
        assert_eq!(cs, result);
        assert_eq!(vec![Offer::new(&wallet, 4, &crypto::hash("tx3".as_bytes()))], bids.offers());
    }
}
