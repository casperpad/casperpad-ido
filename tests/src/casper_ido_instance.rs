use std::time::SystemTime;

use alloc::collections::BTreeMap;

use casper_ido_contract::structs::{Schedules, Time};
use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractHash,
    ContractPackageHash, Key, RuntimeArgs, U256,
};
use contract_utils::key_to_str;
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
        pay_token: Option<ContractHash>,
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
                "pay_token" => pay_token,
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

    /// Admin must set auction token before first schedule
    pub fn set_auction_token(&self, sender: AccountHash, auction_token: String, time: SystemTime) {
        self.0.call_contract_with_time(
            sender,
            "set_auction_token",
            runtime_args! {
                "auction_token" => auction_token
            },
            time,
        );
    }

    pub fn add_orders(&self, sender: AccountHash, orders: BTreeMap<String, U256>) {
        self.0.call_contract(
            sender,
            "add_orders",
            runtime_args! {
                "orders" => orders
            },
        );
    }

    pub fn set_merkle_root(&self, sender: AccountHash, merkle_root: String) {
        self.0.call_contract(
            sender,
            "set_merkle_root",
            runtime_args! {
                "merkle_root" => merkle_root
            },
        );
    }

    pub fn create_order(
        &self,
        sender: AccountHash,
        tier: U256,
        proof: Vec<(String, u8)>,
        token: String,
        amount: U256,
    ) {
        self.0.call_contract(
            sender,
            "set_cspr_price",
            runtime_args! {
            "tier" => tier,
            "proof" => proof,
            "token" => token,
            "amount" => amount},
        );
    }

    pub fn cancel_order(&self, sender: AccountHash) {
        self.0
            .call_contract(sender, "cancel_order", runtime_args! {})
    }

    pub fn claim(&self, sender: AccountHash, schedule_time: u64, time: SystemTime) {
        self.0.call_contract_with_time(
            sender,
            "claim",
            runtime_args! {
                "schedule_time" => schedule_time
            },
            time,
        )
    }

    pub fn schedules(&self) -> Schedules {
        self.0.query_named_key("schedules".to_string())
    }

    /// Actually not working????
    pub fn get_order(&self, sender: AccountHash) -> U256 {
        self.0
            .query_dictionary("orders", key_to_str(&Key::from(sender)))
            .unwrap_or_default()
    }

    pub fn result<T: CLTyped + FromBytes>(&self) -> T {
        self.0.query_named_key("result".to_string())
    }
}
