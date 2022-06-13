#![allow(dead_code)]
use casper_contract::contract_api::runtime;
use casper_types::{account::AccountHash, runtime_args, ContractHash, RuntimeArgs, U256};

use crate::structs::Time;

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
    pub fn add_auction(
        &self,
        auction_contract: ContractHash,
        auction_start_time: Time,
        auction_end_time: Time,
    ) {
        runtime::call_contract(
            self.contract_hash,
            "add_auction",
            runtime_args! {
              "auction_contract" => auction_contract.to_formatted_string(),
              "auction_start_time" => auction_start_time,
              "auction_end_time" => auction_end_time
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
