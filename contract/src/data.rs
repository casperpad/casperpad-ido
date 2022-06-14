use alloc::{string::String, vec::Vec};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_types::{account::AccountHash, ContractHash, Key, U256};
use contract_utils::{get_key, key_and_value_to_str, key_to_str, set_key, Dict};

use crate::{
    structs::{Schedules, Time},
    Error,
};

pub const ORDERS_DIC: &str = "orders";

pub struct Orders {
    dict: Dict,
}

impl Orders {
    pub fn instance() -> Orders {
        Orders {
            dict: Dict::instance(ORDERS_DIC),
        }
    }

    pub fn init() {
        Dict::init(ORDERS_DIC)
    }

    pub fn get(&self, account: &Key) -> Option<U256> {
        self.dict.get(&key_to_str(account))
    }

    pub fn set(&self, account: &Key, value: U256) {
        self.dict.set(&key_to_str(account), value);
    }
}

pub const CLAIMS_DICT: &str = "claims";
pub struct Claims {
    dict: Dict,
}

impl Claims {
    pub fn instance() -> Claims {
        Claims {
            dict: Dict::instance(CLAIMS_DICT),
        }
    }

    pub fn init() {
        Dict::init(CLAIMS_DICT)
    }

    pub fn get(&self, account: &Key, schedule_time: Time) -> Option<bool> {
        self.dict
            .get(&key_and_value_to_str(account, &schedule_time))
    }

    pub fn set(&self, account: &Key, schedule_time: Time, claimed: bool) {
        self.dict
            .set(&key_and_value_to_str(account, &schedule_time), claimed);
    }

    pub fn _remove(&self, account: &Key, schedule_time: Time) {
        self.dict
            .remove::<U256>(&key_and_value_to_str(account, &schedule_time));
    }
}

const INFO: &str = "info";

pub fn set_info(info: &str) {
    set_key(INFO, info);
}

pub fn _get_info() -> String {
    get_key(INFO).unwrap_or_revert()
}

const CREATOR: &str = "creator";

pub fn set_creator(creator: AccountHash) {
    set_key(CREATOR, creator);
}

pub fn get_creator() -> AccountHash {
    get_key(CREATOR).unwrap_or_revert()
}

const MERKLE_ROOT: &str = "merkle_root";

pub fn _set_merkle_root(root: String) {
    set_key(MERKLE_ROOT, root);
}

pub fn _get_merkle_root() -> String {
    get_key(MERKLE_ROOT).unwrap_or_revert_with(Error::InvalidMerkleRoot)
}

const AUCTION_START_TIME: &str = "auction_start_time";

pub fn set_auction_start_time(time: Time) {
    set_key(AUCTION_START_TIME, time);
}

pub fn get_auction_start_time() -> Time {
    get_key(AUCTION_START_TIME).unwrap_or_revert()
}

const AUCTION_END_TIME: &str = "auction_end_time";

pub fn set_auction_end_time(time: Time) {
    set_key(AUCTION_END_TIME, time);
}

pub fn get_auction_end_time() -> Time {
    get_key(AUCTION_END_TIME).unwrap_or_revert()
}

const LAUNCH_TIME: &str = "launch_time";

pub fn set_launch_time(time: Time) {
    set_key(LAUNCH_TIME, time);
}

pub fn _get_launch_time() -> Time {
    get_key(LAUNCH_TIME).unwrap_or_revert()
}

const AUCTION_TOKEN: &str = "auction_token";

pub fn set_auction_token(token: ContractHash) {
    set_key(AUCTION_TOKEN, token);
}

pub fn get_auction_token() -> ContractHash {
    get_key(AUCTION_TOKEN).unwrap_or_revert()
}

const AUCTION_TOKEN_PRICE: &str = "auction_token_price";

pub fn set_auction_token_price(price: U256) {
    set_key(AUCTION_TOKEN_PRICE, price);
}

pub fn get_auction_token_price() -> U256 {
    get_key(AUCTION_TOKEN_PRICE).unwrap_or_revert()
}

const AUCTION_TOKEN_CAPACITY: &str = "auction_token_capacity";

pub fn set_auction_token_capacity(capacity: U256) {
    set_key(AUCTION_TOKEN_CAPACITY, capacity);
}

pub fn get_auction_token_capacity() -> U256 {
    get_key(AUCTION_TOKEN_CAPACITY).unwrap_or_revert()
}

const PAY_TOKEN: &str = "pay_token";

pub fn set_pay_token(pay_token: Option<ContractHash>) {
    set_key(PAY_TOKEN, pay_token);
}

pub fn get_pay_token() -> Option<ContractHash> {
    get_key(PAY_TOKEN).unwrap_or_revert()
}

const SCHEDULES: &str = "schedules";

pub fn set_schedules(schedules: Schedules) {
    set_key(SCHEDULES, schedules);
}

pub fn get_schedules() -> Schedules {
    get_key(SCHEDULES).unwrap_or_revert()
}

const FACTORY_CONTRACT: &str = "factory_contract";

pub fn set_factory_contract(contract_hash: ContractHash) {
    set_key(FACTORY_CONTRACT, contract_hash);
}

pub fn _get_factory_contract() -> ContractHash {
    get_key(FACTORY_CONTRACT).unwrap_or_revert()
}

const TOTAL_PARTICIPANTS: &str = "total_participants";

pub fn _set_total_participants(participants: u64) {
    set_key(TOTAL_PARTICIPANTS, participants);
}

pub fn _get_total_participants() -> u64 {
    get_key(TOTAL_PARTICIPANTS).unwrap_or_revert()
}

const SOLD_AMOUNT: &str = "sold_amount";

pub fn _set_sold_amount(amount: U256) {
    set_key(SOLD_AMOUNT, amount);
}

pub fn _get_sold_amount() -> U256 {
    get_key(SOLD_AMOUNT).unwrap_or_revert()
}

// FACTORY CONTRACT
const FEE_DENOMINATOR: &str = "fee_denominator";

pub fn _set_fee_denominator(fee_denominator: U256) {
    set_key(FEE_DENOMINATOR, fee_denominator);
}

pub fn _get_fee_denominator() -> U256 {
    get_key(FEE_DENOMINATOR).unwrap_or(U256::exp10(4))
}

const TREASURY_WALLET: &str = "treasury_wallet";

pub fn _set_treasury_wallet(treasury_wallet: AccountHash) {
    set_key(TREASURY_WALLET, treasury_wallet);
}

pub fn _get_treasury_wallet() -> AccountHash {
    get_key(TREASURY_WALLET).unwrap_or_revert()
}

const AUCTIONS: &str = "auctions";

pub fn _set_auctions(auctions: Vec<(ContractHash, Time, Time)>) {
    set_key(AUCTIONS, auctions);
}

pub fn _get_auctions() -> Vec<(ContractHash, Time, Time)> {
    get_key(AUCTIONS).unwrap_or_default()
}
