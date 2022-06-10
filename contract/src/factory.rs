use casper_types::{account::AccountHash, ContractHash, U256};
use contract_utils::{ContractContext, ContractStorage};

use crate::data::{
    _get_auctions, _get_fee_denominator, _get_treasury_wallet, _set_auctions, _set_fee_denominator,
    _set_treasury_wallet,
};

pub trait Factory<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(&mut self, fee_denominator: U256, treasury_wallet: AccountHash) {
        self.set_fee_denominator(fee_denominator);
        self.set_treasury_wallet(treasury_wallet);
    }

    fn set_fee_denominator(&mut self, fee_denominator: U256) {
        _set_fee_denominator(fee_denominator);
    }

    fn get_fee_denominator(&self) -> U256 {
        _get_fee_denominator()
    }

    /// Set treasury wallet for fee, should set to only admin call
    fn set_treasury_wallet(&mut self, treasury_wallet: AccountHash) {
        _set_treasury_wallet(treasury_wallet);
    }

    fn get_treasury_wallet(&self) -> AccountHash {
        _get_treasury_wallet()
    }

    fn add_auction(&mut self, auction: ContractHash) {
        let mut auctions = _get_auctions();
        auctions.push(auction);
        _set_auctions(auctions);
    }
}
