use casper_ido_contract::{
    enums::{Address, BiddingToken},
    structs::{Auction, Schedules, Tiers, Time},
};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractHash,
    ContractPackageHash, RuntimeArgs, U256,
};
use test_env::{TestContract, TestEnv};

pub struct CasperIdoInstance(TestContract);

impl CasperIdoInstance {
    pub fn new(
        env: &TestEnv,
        factory_contract: String,
        contract_name: &str,
        sender: AccountHash,
        info: &str,
        auction_start_time: Time,
        auction_end_time: Time,
        launch_time: Time,
        auction_token: Option<String>,
        auction_token_price: U256,
        auction_token_capacity: U256,
        bidding_token: BiddingToken,
        schedules: Schedules,
    ) -> CasperIdoInstance {
        CasperIdoInstance(TestContract::new(
            env,
            "casper_ido_contract.wasm",
            contract_name,
            sender,
            runtime_args! {
                "factory_contract" => factory_contract,
                "info" => info,
                "auction_start_time" => auction_start_time,
                "auction_end_time" => auction_end_time,
                "launch_time" => launch_time,
                "auction_token" => auction_token,
                "auction_token_price" => auction_token_price,
                "auction_token_capacity" => auction_token_capacity,
                "bidding_token" => bidding_token,
                "schedules" => schedules,
            },
        ))
    }

    pub fn contract_package_hash(&self) -> ContractPackageHash {
        self.0.contract_package_hash()
    }

    pub fn contract_hash(&self) -> ContractHash {
        self.0.contract_hash()
    }

    pub fn set_cspr_price(&self, sender: AccountHash, price: U256) {
        self.0.call_contract(
            sender,
            "set_cspr_price",
            runtime_args! {
                "price" => price
            },
        );
    }

    pub fn create_order(
        &self,
        sender: AccountHash,
        auction_id: &str,
        proof: Vec<(String, u8)>,
        token: String,
        amount: U256,
    ) {
        self.0.call_contract(
            sender,
            "set_cspr_price",
            runtime_args! {
            "auction_id" => auction_id,
            "proof" => proof,
            "token" => token,
            "amount" => amount},
        );
    }

    pub fn result<T: CLTyped + FromBytes>(&self) -> T {
        self.0.query_named_key("result".to_string())
    }

    pub fn result2<T: CLTyped + FromBytes>(&self) -> T {
        self.0.query_named_key("result2".to_string())
    }
}
