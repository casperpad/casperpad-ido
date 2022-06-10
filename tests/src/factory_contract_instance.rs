use casper_ido_contract::enums::Address;
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
        treasury_wallet: String,
        fee_denominator: U256,
    ) -> FactoryContractInstance {
        FactoryContractInstance(TestContract::new(
            env,
            "factory_contract.wasm",
            contract_name,
            sender,
            runtime_args! {
                "treasury_wallet" => treasury_wallet,
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

    pub fn set_treasury_wallet(&self, sender: AccountHash, treasury_wallet: String) {
        self.0.call_contract(
            sender,
            "set_treasury_wallet",
            runtime_args! {
                "treasury_wallet" => treasury_wallet,
            },
        )
    }

    pub fn get_treasury_wallet(&self) -> AccountHash {
        self.0.query_named_key("treasury_wallet".to_string())
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

    pub fn add_auction(&self, sender: AccountHash, auction: String) {
        self.0.call_contract(
            sender,
            "add_auction",
            runtime_args! {
                "auction" => auction,
            },
        )
    }

    pub fn result<T: CLTyped + FromBytes>(&self) -> T {
        self.0.query_named_key("result".to_string())
    }
}
