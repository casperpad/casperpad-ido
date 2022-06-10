#![allow(dead_code)]
use casper_contract::contract_api::runtime;
use casper_types::{account::AccountHash, runtime_args, ContractHash, RuntimeArgs, U256};

pub struct IFactory {
    pub contract_hash: ContractHash,
}

impl IFactory {
    pub fn new(contract_hash: ContractHash) -> Self {
        IFactory { contract_hash }
    }
    pub fn get_fee_denominator(&self) -> U256 {
        runtime::call_contract(self.contract_hash, "get_fee_denominator", runtime_args! {})
    }
    pub fn get_treasury_wallet(&self) -> AccountHash {
        runtime::call_contract(self.contract_hash, "get_treasury_wallet", runtime_args! {})
    }
    pub fn add_auction(&self, auction: ContractHash) {
        runtime::call_contract(
            self.contract_hash,
            "add_auction",
            runtime_args! {
              "auction" => auction.to_formatted_string(),
            },
        )
    }

    pub fn assert_caller_is_admin(&self, caller: AccountHash) {
        runtime::call_contract(
            self.contract_hash,
            "assert_caller_is_admin",
            runtime_args! {
              "caller" => caller,
            },
        )
    }
}
