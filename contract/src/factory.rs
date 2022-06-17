use casper_types::{account::AccountHash, ContractHash, U256};
use contract_utils::{ContractContext, ContractStorage};

use crate::{
    data::{
        _get_auctions, _get_fee_denominator, _get_fee_wallet, _set_auctions, _set_fee_denominator,
        _set_fee_wallet,
    },
    structs::Time,
};

pub trait Factory<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(&mut self, fee_denominator: U256, fee_wallet: AccountHash) {
        self.set_fee_denominator(fee_denominator);
        self.set_fee_wallet(fee_wallet);
    }

    fn set_fee_denominator(&mut self, fee_denominator: U256) {
        _set_fee_denominator(fee_denominator);
    }

    fn get_fee_denominator(&self) -> U256 {
        _get_fee_denominator()
    }

    /// Set fee wallet for fee, should set to only admin call
    fn set_fee_wallet(&mut self, fee_wallet: AccountHash) {
        _set_fee_wallet(fee_wallet);
    }

    fn get_fee_wallet(&self) -> AccountHash {
        _get_fee_wallet()
    }

    fn add_auction(&mut self, auction_contract: ContractHash, start_time: Time, end_time: Time) {
        let mut auctions = _get_auctions();
        auctions.push((auction_contract, start_time, end_time));
        _set_auctions(auctions);
    }

    fn remove_auction(&mut self, index: usize) {
        let mut auctions = _get_auctions();
        auctions.remove(index);
        _set_auctions(auctions);
    }
}
