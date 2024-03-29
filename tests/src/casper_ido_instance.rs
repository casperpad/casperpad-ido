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
        contract_name: &str,
        sender: AccountHash,
        auction_start_time: Time,
        auction_end_time: Time,
        auction_token_price: U256,
        auction_token_capacity: U256,
        pay_token: Option<String>,
        schedules: Schedules,
        treasury_wallet: String,
    ) -> CasperIdoInstance {
        let exist_version: Option<String> = None;
        CasperIdoInstance(TestContract::new(
            env,
            "casper_ido_contract.wasm",
            contract_name,
            sender,
            runtime_args! {
                "auction_start_time" => auction_start_time,
                "auction_end_time" => auction_end_time,
                "auction_token_price" => auction_token_price,
                "auction_token_capacity" => auction_token_capacity,
                "pay_token" => pay_token,
                "schedules" => schedules,
                "treasury_wallet" => treasury_wallet,
                "contract_package_hash" => exist_version

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
    pub fn set_auction_token(
        &self,
        sender: AccountHash,
        auction_token: String,
        auction_token_capacity: U256,
        time: SystemTime,
    ) {
        self.0.call_contract_with_time(
            sender,
            "set_auction_token",
            runtime_args! {
                "auction_token" => auction_token,
                "auction_token_capacity" => auction_token_capacity
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

    pub fn change_time_schedules(
        self,
        sender: AccountHash,
        auction_start_time: Time,
        auction_end_time: Time,
        launch_time: Time,
        schedules: Schedules,
    ) {
        self.0.call_contract(
            sender,
            "add_orders",
            runtime_args! {
                "auction_start_time" => auction_start_time,
                "auction_end_time" => auction_end_time,
                "launch_time" => launch_time,
                "schedules" => schedules,
            },
        );
    }

    pub fn set_treasury_wallet(&self, sender: AccountHash, treasury_wallet: String) {
        self.0.call_contract(
            sender,
            "set_treasury_wallet",
            runtime_args! {
                "treasury_wallet" => treasury_wallet,
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
        amount: U256,
        time: SystemTime,
    ) {
        self.0.call_contract_with_time(
            sender,
            "create_order",
            runtime_args! {
                "tier" => tier,
                "proof" => proof,
                "amount" => amount
            },
            time,
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
