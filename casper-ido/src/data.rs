use casper_contract::{contract_api::system, unwrap_or_revert::UnwrapOrRevert};
use contract_utils::{get_key, set_key, Dict};

use crate::structs::Auction;

const AUCTIONS_DICT: &str = "auctions";

pub struct Auctions {
    dict: Dict,
}

impl Auctions {
    pub fn instance() -> Auctions {
        Auctions {
            dict: Dict::instance(AUCTIONS_DICT),
        }
    }

    pub fn init() {
        Dict::init(AUCTIONS_DICT)
    }

    pub fn get(&self, id: &String) -> Option<Auction> {
        self.dict.get(id)
    }

    pub fn set(&self, id: &String, value: Auction) {
        self.dict.set(id, value);
    }

    pub fn remove(&self, key: &String) {
        self.dict.remove::<Auction>(&key.to_string());
    }
}

fn create_purse_key(value: &String) -> String {
    format!("{}_purse", value)
}

pub fn create_auction_purse(auction_id: &String) {
    let purse = system::create_purse();

    set_key(&create_purse_key(auction_id), purse);
}

pub fn auction_purse(auction_id: &String) {
    get_key(&create_purse_key(auction_id)).unwrap_or_revert()
}
