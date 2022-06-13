#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{
    collections::BTreeSet,
    format,
    string::{String, ToString},
    vec,
};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_ido_contract::{structs::Time, Factory};
use casper_types::{
    account::AccountHash, runtime_args, ApiError, CLType, ContractHash, ContractPackageHash,
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Group, Key, Parameter, RuntimeArgs,
    URef, U256,
};
use contract_utils::{
    set_key, AdminControl, ContractContext, OnChainContractStorage, ReentrancyGuard,
};

#[derive(Default)]
struct FactoryContract(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for FactoryContract {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl Factory<OnChainContractStorage> for FactoryContract {}
impl AdminControl<OnChainContractStorage> for FactoryContract {}
impl ReentrancyGuard<OnChainContractStorage> for FactoryContract {}

impl FactoryContract {
    fn constructor(&mut self, fee_denominator: U256, treasury_wallet: AccountHash) {
        Factory::init(self, fee_denominator, treasury_wallet);
        AdminControl::init(self);
        ReentrancyGuard::init(self);
    }
}

#[no_mangle]
pub extern "C" fn constructor() {
    let fee_denominator: U256 = runtime::get_named_arg("fee_denominator");
    let treasury_wallet: AccountHash = {
        let treasury_wallet_str: String = runtime::get_named_arg("treasury_wallet");
        AccountHash::from_formatted_str(&treasury_wallet_str).unwrap()
    };
    FactoryContract::default().constructor(fee_denominator, treasury_wallet);
    let default_admin = runtime::get_caller();

    FactoryContract::default().add_admin_without_checked(Key::from(default_admin));
    set_key("install_time", u64::from(runtime::get_blocktime()));
}

#[no_mangle]
pub extern "C" fn assert_caller_is_admin() {
    let caller: AccountHash = runtime::get_named_arg("caller");
    if !FactoryContract::default().is_admin(Key::from(caller)) {
        runtime::revert(ApiError::PermissionDenied);
    }
}

#[no_mangle]
pub extern "C" fn add_admin() {
    let admin: AccountHash = {
        let admin_string: String = runtime::get_named_arg("admin");
        AccountHash::from_formatted_str(&admin_string).unwrap()
    };
    FactoryContract::default().add_admin(Key::from(admin));
}

#[no_mangle]
pub extern "C" fn remove_admin() {
    let admin: AccountHash = {
        let admin_string: String = runtime::get_named_arg("admin");
        AccountHash::from_formatted_str(&admin_string).unwrap()
    };
    FactoryContract::default().disable_admin(Key::from(admin));
}

#[no_mangle]
pub extern "C" fn set_fee_denominator() {
    FactoryContract::default().assert_caller_is_admin();
    let fee_denominator: U256 = runtime::get_named_arg("fee_denominator");
    FactoryContract::default().set_fee_denominator(fee_denominator);
}

#[no_mangle]
pub extern "C" fn set_treasury_wallet() {
    FactoryContract::default().assert_caller_is_admin();
    let treasury_wallet: AccountHash = {
        let treasury_wallet: String = runtime::get_named_arg("treasury_wallet");
        AccountHash::from_formatted_str(&treasury_wallet).unwrap()
    };
    FactoryContract::default().set_treasury_wallet(treasury_wallet);
}

#[no_mangle]
pub extern "C" fn add_auction() {
    FactoryContract::default().assert_caller_is_admin();
    let auction_contract: ContractHash = {
        let auction_contract_string: String = runtime::get_named_arg("auction_contract");
        ContractHash::from_formatted_str(&auction_contract_string).unwrap()
    };
    let auction_start_time: Time = runtime::get_named_arg("auction_start_time");
    let auction_end_time: Time = runtime::get_named_arg("auction_end_time");
    FactoryContract::default().add_auction(auction_contract, auction_start_time, auction_end_time);
}

#[no_mangle]
pub extern "C" fn call() {
    let contract_name: String = runtime::get_named_arg("contract_name");
    let treasury_wallet: String = runtime::get_named_arg("treasury_wallet");
    let fee_denominator: U256 = runtime::get_named_arg("fee_denominator");

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
        "treasury_wallet" => treasury_wallet,
        "fee_denominator" => fee_denominator
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
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![
            Parameter::new("treasury_wallet".to_string(), CLType::String),
            Parameter::new("fee_denominator".to_string(), CLType::U256),
        ],
        CLType::Unit,
        EntryPointAccess::Groups(vec![Group::new("constructor")]),
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_fee_denominator",
        vec![Parameter::new("fee_denominator".to_string(), CLType::U256)],
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

    entry_points.add_entry_point(EntryPoint::new(
        "add_auction",
        vec![Parameter::new("auction".to_string(), CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}
