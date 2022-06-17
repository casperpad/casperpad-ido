use casper_ido_contract::structs::Time;
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractHash,
    ContractPackageHash, RuntimeArgs, U256,
};
use test_env::{TestContract, TestEnv};

pub struct FactoryContractInstance(TestContract);

impl FactoryContractInstance {
    pub fn new(
        env: &TestEnv,
        contract_name: &str,
        sender: AccountHash,
        fee_wallet: String,
        fee_denominator: U256,
    ) -> FactoryContractInstance {
        FactoryContractInstance(TestContract::new(
            env,
            "factory_contract.wasm",
            contract_name,
            sender,
            runtime_args! {
                "fee_wallet" => fee_wallet,
                "fee_denominator" => fee_denominator
            },
        ))
    }

    pub fn contract_package_hash(&self) -> ContractPackageHash {
        self.0.contract_package_hash()
    }

    pub fn contract_hash(&self) -> ContractHash {
        self.0.contract_hash()
    }

    pub fn set_fee_wallet(&self, sender: AccountHash, fee_wallet: String) {
        self.0.call_contract(
            sender,
            "set_fee_wallet",
            runtime_args! {
                "fee_wallet" => fee_wallet,
            },
        )
    }

    pub fn get_fee_wallet(&self) -> AccountHash {
        self.0.query_named_key("fee_wallet".to_string())
    }

    pub fn set_fee_denominator(&self, sender: AccountHash, fee_denominator: U256) {
        self.0.call_contract(
            sender,
            "set_fee_denominator",
            runtime_args! {
                "fee_denominator" => fee_denominator,
            },
        )
    }

    pub fn get_fee_denominator(&self) -> U256 {
        self.0.query_named_key("fee_denominator".to_string())
    }

    pub fn add_auction(
        &self,
        sender: AccountHash,
        auction_contract: String,
        auction_start_time: Time,
        auction_end_time: Time,
    ) {
        self.0.call_contract(
            sender,
            "add_auction",
            runtime_args! {
                "auction_contract" => auction_contract,
                "auction_start_time" => auction_start_time,
                "auction_end_time" => auction_end_time,
            },
        )
    }

    pub fn remove_auction(&self, sender: AccountHash, index: u32) {
        self.0.call_contract(
            sender,
            "remove_auction",
            runtime_args! {
                "index" => index,
            },
        )
    }

    pub fn auctions(&self) -> Vec<(ContractHash, u64, u64)> {
        self.0.query_named_key("auctions".to_string())
    }

    pub fn result<T: CLTyped + FromBytes>(&self) -> T {
        self.0.query_named_key("result".to_string())
    }
}
