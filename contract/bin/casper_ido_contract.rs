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
    collections::BTreeSet,
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
    enums::{Address, BiddingToken},
    structs::{Schedules, Time},
    CasperIdo, IFactory,
};

use casper_types::{
    runtime_args, CLType, CLTyped, ContractHash, ContractPackageHash, EntryPoint, EntryPointAccess,
    EntryPointType, EntryPoints, Group, Parameter, RuntimeArgs, URef, U256,
};
use contract_utils::{ContractContext, OnChainContractStorage, ReentrancyGuard};

#[derive(Default)]
struct CasperIdoContract(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for CasperIdoContract {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl CasperIdo<OnChainContractStorage> for CasperIdoContract {}
impl ReentrancyGuard<OnChainContractStorage> for CasperIdoContract {}

impl CasperIdoContract {
    fn constructor(
        &mut self,
        factory_contract: ContractHash,
        info: &str,
        auction_start_time: Time,
        auction_end_time: Time,
        launch_time: Time,
        auction_token: Option<ContractHash>,
        auction_token_price: U256,
        auction_token_capacity: U256,
        bidding_token: BiddingToken,
        schedules: Schedules,
    ) {
        CasperIdo::init(
            self,
            factory_contract,
            info,
            auction_start_time,
            auction_end_time,
            launch_time,
            auction_token,
            auction_token_price,
            auction_token_capacity,
            bidding_token,
            schedules,
        );
        ReentrancyGuard::init(self);
    }
}

#[no_mangle]
pub extern "C" fn constructor() {
    let factory_contract: ContractHash = {
        let constract_str: String = runtime::get_named_arg("factory_contract");
        ContractHash::from_formatted_str(&constract_str).unwrap()
    };
    let info: String = runtime::get_named_arg("info");
    let creator = runtime::get_caller();
    let auction_created_time = Time::from(runtime::get_blocktime());
    let auction_start_time: Time = runtime::get_named_arg("auction_start_time");
    let auction_end_time: Time = runtime::get_named_arg("auction_end_time");
    let launch_time: Time = runtime::get_named_arg("launch_time");
    let auction_token = {
        let auction_token_string: Option<String> = runtime::get_named_arg("auction_token");
        auction_token_string.map(|str| ContractHash::from_formatted_str(&str).unwrap())
    };
    let auction_token_price: U256 = runtime::get_named_arg("auction_token_price");
    let auction_token_capacity: U256 = runtime::get_named_arg("auction_token_capacity");
    let bidding_token: BiddingToken = runtime::get_named_arg("bidding_token");
    let schedules: Schedules = runtime::get_named_arg("schedules");

    CasperIdoContract::default().constructor(
        factory_contract,
        &info,
        auction_start_time,
        auction_end_time,
        launch_time,
        auction_token,
        auction_token_price,
        auction_token_capacity,
        bidding_token,
        schedules,
    );
}

#[no_mangle]
pub extern "C" fn create_order() {
    let caller = runtime::get_caller();
    let tier: U256 = runtime::get_named_arg("tier");
    let proof: Vec<(String, u8)> = runtime::get_named_arg("proof");
    let token: ContractHash = {
        let token_contract_string: String = runtime::get_named_arg("token");
        ContractHash::from_formatted_str(&token_contract_string).unwrap()
    };
    let amount: U256 = runtime::get_named_arg("amount");
    CasperIdoContract::default().create_order(caller, tier, proof, token, amount);
}

#[no_mangle]
pub extern "C" fn create_order_cspr() {
    let caller = runtime::get_caller();
    let tier: U256 = runtime::get_named_arg("tier");
    let proof: Vec<(String, u8)> = runtime::get_named_arg("proof");
    let deposit_purse: URef = runtime::get_named_arg("deposit_purse");
    CasperIdoContract::default().create_order_cspr(caller, tier, proof, deposit_purse);
}

#[no_mangle]
pub extern "C" fn cancel_order() {
    let caller = runtime::get_caller();

    CasperIdoContract::default().set_reentrancy();
    CasperIdoContract::default().cancel_order(caller);
    CasperIdoContract::default().clear_reentrancy();
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
pub extern "C" fn set_cspr_price() {
    let price: U256 = runtime::get_named_arg("price");
    CasperIdoContract::default().set_cspr_price(price);
}

#[no_mangle]
pub extern "C" fn set_auction_token() {
    let auction_token: ContractHash = {
        let auction_token_str: String = runtime::get_named_arg("auction_token");
        ContractHash::from_formatted_str(&auction_token_str).unwrap()
    };
    CasperIdoContract::default().set_auction_token(auction_token);
}

#[no_mangle]
pub extern "C" fn set_merkle_root() {
    let merkle_root: String = runtime::get_named_arg("merkle_root");
    CasperIdoContract::default().set_merkle_root(merkle_root);
}

#[no_mangle]
pub extern "C" fn call() {
    let contract_name: String = runtime::get_named_arg("contract_name");
    let factory_contract: String = runtime::get_named_arg("factory_contract");
    let info: String = runtime::get_named_arg("info");
    let auction_start_time: Time = runtime::get_named_arg("auction_start_time");
    let auction_end_time: Time = runtime::get_named_arg("auction_end_time");
    let launch_time: Time = runtime::get_named_arg("launch_time");
    let auction_token: Option<String> = runtime::get_named_arg("auction_token");
    let auction_token_price: U256 = runtime::get_named_arg("auction_token_price");
    let auction_token_capacity: U256 = runtime::get_named_arg("auction_token_capacity");
    let bidding_token: BiddingToken = runtime::get_named_arg("bidding_token");
    let schedules: Schedules = runtime::get_named_arg("schedules");

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
        "factory_contract" => factory_contract.clone(),
        "info" => info,
        "auction_start_time" => auction_start_time,
        "auction_end_time" => auction_end_time,
        "launch_time" => launch_time,
        "auction_token" => auction_token,
        "auction_token_price" => auction_token_price,
        "auction_token_capacity" => auction_token_capacity,
        "bidding_token" => bidding_token,
        "schedules" => schedules,
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
    runtime::put_key(
        &format!("{}_contract_hash_wrapped", contract_name),
        storage::new_uref(contract_hash).into(),
    );
    // IMPORTANT!!!
    // IFactory::new(ContractHash::from_formatted_str(&factory_contract).unwrap())
    //     .add_auction(contract_hash);
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![Parameter::new(
            "default_treasury_wallet".to_string(),
            Address::cl_type(),
        )],
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
            Parameter::new("token".to_string(), CLType::String),
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
        "cancel_order",
        vec![],
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
        vec![Parameter::new("auction_token".to_string(), CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "set_cspr_price",
        vec![Parameter::new("price".to_string(), CLType::U256)],
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

    entry_points
}
