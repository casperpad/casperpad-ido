use alloc::{
    format,
    string::{String, ToString},
};
use casper_contract::{contract_api::system, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{URef, U256};
use contract_utils::{get_key, set_key, Dict};

use crate::{enums::Address, structs::Auction};

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

pub fn auction_purse(auction_id: &String) -> URef {
    get_key(&create_purse_key(auction_id)).unwrap_or_revert()
}

const TREASURY_WALLET_KEY: &str = "treasury_wallet";

pub struct TreasuryWallet {
    key: &'static str,
}

impl TreasuryWallet {
    pub fn instance() -> TreasuryWallet {
        TreasuryWallet {
            key: TREASURY_WALLET_KEY,
        }
    }

    pub fn init(address: Address) {
        TreasuryWallet::instance().set_treasury_wallet(address);
    }

    pub fn set_treasury_wallet(&self, address: Address) {
        set_key(self.key, "address");
    }

    pub fn get_treasury_wallet(&self) -> Address {
        get_key(self.key).unwrap_or_revert()
    }
}

const FEE_DENOMINATOR_KEY: &str = "fee_denominator";

pub struct FeeDeNominator {
    key: &'static str,
}

impl FeeDeNominator {
    pub fn instance() -> FeeDeNominator {
        FeeDeNominator {
            key: FEE_DENOMINATOR_KEY,
        }
    }

    pub fn init(fee_denominator: U256) {
        FeeDeNominator::instance().set_fee_denominator(fee_denominator);
    }

    pub fn set_fee_denominator(&self, fee_denominator: U256) {
        set_key(self.key, fee_denominator);
    }

    pub fn get_fee_denominator(&self) -> U256 {
        get_key(self.key).unwrap_or(U256::exp10(5))
    }
}
