#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_ido_contract::{
    structs::{Schedules, Time},
    CasperIdo,
};

use casper_types::{
    account::AccountHash, runtime_args, CLType, ContractHash, ContractPackageHash, EntryPoint,
    EntryPointAccess, EntryPointType, EntryPoints, Group, Key, Parameter, RuntimeArgs, URef, U256,
};
use contract_utils::{AdminControl, ContractContext, OnChainContractStorage, ReentrancyGuard};

#[derive(Default)]
struct CasperIdoContract(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for CasperIdoContract {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl CasperIdo<OnChainContractStorage> for CasperIdoContract {}
impl ReentrancyGuard<OnChainContractStorage> for CasperIdoContract {}
impl AdminControl<OnChainContractStorage> for CasperIdoContract {}

impl CasperIdoContract {
    fn constructor(
        &mut self,
        auction_start_time: Time,
        auction_end_time: Time,
        auction_token_price: U256,
        auction_token_capacity: U256,
        pay_token: Option<ContractHash>,
        schedules: Schedules,
        treasury_wallet: AccountHash,
    ) {
        CasperIdo::init(
            self,
            auction_start_time,
            auction_end_time,
            auction_token_price,
            auction_token_capacity,
            pay_token,
            schedules,
            treasury_wallet,
        );
        AdminControl::init(self);
        ReentrancyGuard::init(self);
    }
}

#[no_mangle]
pub extern "C" fn constructor() {
    let creator = runtime::get_caller();
    let auction_start_time: Time = runtime::get_named_arg("auction_start_time");
    let auction_end_time: Time = runtime::get_named_arg("auction_end_time");
    let auction_token_price: U256 = runtime::get_named_arg("auction_token_price");
    let auction_token_capacity: U256 = runtime::get_named_arg("auction_token_capacity");
    let pay_token: Option<ContractHash> = {
        let pay_token_str: Option<String> = runtime::get_named_arg("pay_token");
        pay_token_str.map(|str| ContractHash::from_formatted_str(&str).unwrap())
    };
    let schedules: Schedules = runtime::get_named_arg("schedules");
    let treasury_wallet: AccountHash = {
        let treasury_wallet_str: String = runtime::get_named_arg("treasury_wallet");
        AccountHash::from_formatted_str(&treasury_wallet_str).unwrap()
    };
    CasperIdoContract::default().constructor(
        auction_start_time,
        auction_end_time,
        auction_token_price,
        auction_token_capacity,
        pay_token,
        schedules,
        treasury_wallet,
    );
    let default_admin = runtime::get_caller();
    CasperIdoContract::default().add_admin_without_checked(Key::from(default_admin))
}

#[no_mangle]
pub extern "C" fn create_order() {
    let caller = runtime::get_caller();
    let tier: U256 = runtime::get_named_arg("tier");
    let proof: Vec<(String, u8)> = runtime::get_named_arg("proof");
    let amount: U256 = runtime::get_named_arg("amount");
    CasperIdoContract::default().set_reentrancy();
    CasperIdoContract::default().create_order(caller, tier, proof, amount);
    CasperIdoContract::default().clear_reentrancy();
}

#[no_mangle]
pub extern "C" fn create_order_cspr() {
    let caller = runtime::get_caller();
    let tier: U256 = runtime::get_named_arg("tier");
    let proof: Vec<(String, u8)> = runtime::get_named_arg("proof");
    let deposit_purse: URef = runtime::get_named_arg("deposit_purse");
    CasperIdoContract::default().set_reentrancy();
    CasperIdoContract::default().create_order_cspr(caller, tier, proof, deposit_purse);
    CasperIdoContract::default().clear_reentrancy();
}

#[no_mangle]
pub extern "C" fn add_orders() {
    let orders: BTreeMap<String, U256> = runtime::get_named_arg("orders");
    CasperIdoContract::default().assert_caller_is_admin();
    CasperIdoContract::default().add_orders(orders);
}

#[no_mangle]
pub extern "C" fn claim() {
    let caller = runtime::get_caller();
    let schedule_time: Time = runtime::get_named_arg("schedule_time");

    CasperIdoContract::default().set_reentrancy();
    CasperIdoContract::default().claim(caller, schedule_time);
    CasperIdoContract::default().clear_reentrancy();
}

#[no_mangle]
pub extern "C" fn set_auction_token() {
    let auction_token: ContractHash = {
        let auction_token_str: String = runtime::get_named_arg("auction_token");
        ContractHash::from_formatted_str(&auction_token_str).unwrap()
    };
    let auction_token_capacity: U256 = runtime::get_named_arg("auction_token_capacity");
    CasperIdoContract::default().assert_caller_is_admin();
    CasperIdoContract::default().set_auction_token(auction_token, auction_token_capacity);
}

#[no_mangle]
pub extern "C" fn change_auction_token_price() {
    let auction_token_price: U256 = runtime::get_named_arg("price");
    CasperIdoContract::default().assert_caller_is_admin();
    CasperIdoContract::default().change_auction_token_price(auction_token_price);
}

#[no_mangle]
pub extern "C" fn set_treasury_wallet() {
    let treasury_wallet: AccountHash = {
        let treasury_wallet_str: String = runtime::get_named_arg("treasury_wallet");
        AccountHash::from_formatted_str(&treasury_wallet_str).unwrap()
    };
    CasperIdoContract::default().assert_caller_is_admin();
    CasperIdoContract::default().set_treasury_wallet(treasury_wallet);
}

#[no_mangle]
pub extern "C" fn change_time_schedules() {
    let auction_start_time: Time = runtime::get_named_arg("auction_start_time");
    let auction_end_time: Time = runtime::get_named_arg("auction_end_time");
    let schedules: Schedules = runtime::get_named_arg("schedules");
    CasperIdoContract::default().assert_caller_is_admin();
    CasperIdoContract::default().change_time_schedules(
        auction_start_time,
        auction_end_time,
        schedules,
    );
}

#[no_mangle]
pub extern "C" fn set_merkle_root() {
    let merkle_root: String = runtime::get_named_arg("merkle_root");
    CasperIdoContract::default().assert_caller_is_admin();
    CasperIdoContract::default().set_merkle_root(merkle_root);
}

#[no_mangle]
pub extern "C" fn add_admin() {
    let admin: AccountHash = {
        let admin_string: String = runtime::get_named_arg("admin");
        AccountHash::from_formatted_str(&admin_string).unwrap()
    };
    CasperIdoContract::default().add_admin(Key::from(admin));
}

#[no_mangle]
pub extern "C" fn remove_admin() {
    let admin: AccountHash = {
        let admin_string: String = runtime::get_named_arg("admin");
        AccountHash::from_formatted_str(&admin_string).unwrap()
    };
    CasperIdoContract::default().disable_admin(Key::from(admin));
}

#[no_mangle]
pub extern "C" fn call() {
    let contract_name: String = runtime::get_named_arg("contract_name");
    let auction_start_time: Time = runtime::get_named_arg("auction_start_time");
    let auction_end_time: Time = runtime::get_named_arg("auction_end_time");
    let auction_token_price: U256 = runtime::get_named_arg("auction_token_price");
    let auction_token_capacity: U256 = runtime::get_named_arg("auction_token_capacity");
    let pay_token: Option<String> = runtime::get_named_arg("pay_token");
    let schedules: Schedules = runtime::get_named_arg("schedules");
    let treasury_wallet: String = runtime::get_named_arg("treasury_wallet");
    let (contract_hash, _) = storage::new_contract(
        get_entry_points(),
        None,
        Some(String::from(format!(
            "{}_contract_package_hash",
            contract_name
        ))),
        None,
    );

    let package_hash: ContractPackageHash = ContractPackageHash::new(
        runtime::get_key(&format!("{}_contract_package_hash", contract_name))
            .unwrap_or_revert()
            .into_hash()
            .unwrap_or_revert(),
    );
    let constructor_access: URef =
        storage::create_contract_user_group(package_hash, "constructor", 1, Default::default())
            .unwrap_or_revert()
            .pop()
            .unwrap_or_revert();

    let constructor_args = runtime_args! {
        "auction_start_time" => auction_start_time,
        "auction_end_time" => auction_end_time,
        "auction_token_price" => auction_token_price,
        "auction_token_capacity" => auction_token_capacity,
        "pay_token" => pay_token,
        "schedules" => schedules,
        "treasury_wallet" => treasury_wallet
    };
    let _: () = runtime::call_contract(contract_hash, "constructor", constructor_args);

    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(package_hash, "constructor", urefs)
        .unwrap_or_revert();

    runtime::put_key(
        &format!("{}_contract_hash", contract_name),
        contract_hash.into(),
    );
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![
            Parameter::new("auction_start_time".to_string(), CLType::U64),
            Parameter::new("auction_end_time".to_string(), CLType::U64),
            Parameter::new("auction_token_price".to_string(), CLType::U256),
            Parameter::new("auction_token_capacity".to_string(), CLType::U256),
            Parameter::new("pay_token".to_string(), CLType::String),
            Parameter::new(
                "schedules".to_string(),
                CLType::Map {
                    key: Box::new(CLType::U64),
                    value: Box::new(CLType::U256),
                },
            ),
            Parameter::new("treasury_wallet".to_string(), CLType::U64),
        ],
        CLType::Unit,
        EntryPointAccess::Groups(vec![Group::new("constructor")]),
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "create_order",
        vec![
            Parameter::new("tier".to_string(), CLType::U256),
            Parameter::new(
                "proof".to_string(),
                CLType::List(Box::new(CLType::Tuple2([
                    Box::new(CLType::String),
                    Box::new(CLType::U8),
                ]))),
            ),
            Parameter::new("amount".to_string(), CLType::U256),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "create_order_cspr",
        vec![
            Parameter::new("tier".to_string(), CLType::U256),
            Parameter::new(
                "proof".to_string(),
                CLType::List(Box::new(CLType::Tuple2([
                    Box::new(CLType::String),
                    Box::new(CLType::U8),
                ]))),
            ),
            Parameter::new("deposit_purse".to_string(), CLType::URef),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "add_orders",
        vec![Parameter::new(
            "orders".to_string(),
            CLType::Map {
                key: Box::new(CLType::String),
                value: Box::new(CLType::U256),
            },
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "claim",
        vec![Parameter::new("schedule_time".to_string(), CLType::U64)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_auction_token",
        vec![
            Parameter::new("auction_token".to_string(), CLType::String),
            Parameter::new("auction_token_capacity".to_string(), CLType::U256),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "change_auction_token_price",
        vec![Parameter::new("price".to_string(), CLType::U256)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_treasury_wallet",
        vec![Parameter::new(
            "treasury_wallet".to_string(),
            CLType::String,
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "change_time_schedules",
        vec![
            Parameter::new("auction_start_time".to_string(), CLType::U64),
            Parameter::new("auction_end_time".to_string(), CLType::U64),
            Parameter::new(
                "schedules".to_string(),
                CLType::Map {
                    key: Box::new(CLType::U64),
                    value: Box::new(CLType::U256),
                },
            ),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_merkle_root",
        vec![Parameter::new("merkle_root".to_string(), CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "add_admin",
        vec![Parameter::new("admin".to_string(), CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "remove_admin",
        vec![Parameter::new("admin".to_string(), CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}
