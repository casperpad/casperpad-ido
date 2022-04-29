#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;
use crate::alloc::string::{String, ToString};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::{Address, ERC20};
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, contracts::NamedKeys, CLTyped, CLValue, ContractHash,
    Key, URef, U256,
};

mod constants;
mod default_treasury_wallet;
mod detail;
mod entry_points;
mod error;
mod invests;
mod owner;
mod project;
mod projects;
use constants::{
    CONTRACT_NAME_KEY_NAME, CSPR_AMOUNT_RUNTIME_ARG_NAME, DEFAULT_TREASURY_WALLET_KEY_NAME,
    DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME, INVESTS_KEY_NAME, OWNER_KEY_NAME,
    OWNER_RUNTIME_ARG_NAME, PROJECTS_KEY_NAME, PROJECT_ID_RUNTIME_ARG_NAME,
    PROJECT_NAME_RUNTIME_ARG_NAME, PROJECT_OPEN_TIME_RUNTIME_ARG_NAME,
    PROJECT_PRIVATE_RUNTIME_ARG_NAME, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME,
    PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME, PROJECT_STATUS_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    RESULT_KEY_NAME, TREASURY_WALLET_RUNTIME_ARG_NAME, USERS_KEY_NAME,
};
use error::Error;
use project::{Project, Status};

fn store_result<T: CLTyped + ToBytes>(result: T) {
    match runtime::get_key(RESULT_KEY_NAME) {
        Some(Key::URef(uref)) => storage::write(uref, result),
        Some(_) => unreachable!(),
        None => {
            let new_uref = storage::new_uref(result);
            runtime::put_key(RESULT_KEY_NAME, new_uref.into());
        }
    }
}

#[no_mangle]
pub extern "C" fn transfer_ownership() {
    owner::only_owner();
    let new_owner_hash: AccountHash = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let owner_uref: URef = owner::owner_uref();
    owner::write_owner_to(owner_uref, Address::from(new_owner_hash));
    runtime::ret(CLValue::from_t(true).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn get_owner() {
    let owner: Address = detail::read_from(OWNER_KEY_NAME);
    runtime::ret(CLValue::from_t(owner).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn set_default_treasury_wallet() {
    owner::only_owner();
    let new_default_treasury_wallet: Address =
        runtime::get_named_arg(DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME);
    let default_treasury_wallet_uref = default_treasury_wallet::default_treasury_wallet_uref();
    default_treasury_wallet::write_default_tresury_wallet_to(
        default_treasury_wallet_uref,
        new_default_treasury_wallet,
    )
}

#[no_mangle]
pub extern "C" fn get_default_treasury_wallet() {
    let default_treasury_wallet_uref = default_treasury_wallet::default_treasury_wallet_uref();
    let default_treasury_wallet: Address =
        default_treasury_wallet::read_default_treasury_wallet_from(default_treasury_wallet_uref);
    store_result(default_treasury_wallet);
    runtime::ret(CLValue::from_t(default_treasury_wallet).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn add_project() {
    owner::only_owner();
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let project_name: String = runtime::get_named_arg(PROJECT_NAME_RUNTIME_ARG_NAME);
    let project_sale_start_time: i64 =
        runtime::get_named_arg(PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME);
    let project_sale_end_time: i64 = runtime::get_named_arg(PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME);
    let project_open_time: i64 = runtime::get_named_arg(PROJECT_OPEN_TIME_RUNTIME_ARG_NAME);
    let project_private: bool = runtime::get_named_arg(PROJECT_PRIVATE_RUNTIME_ARG_NAME);
    let project_token_symbol: String =
        runtime::get_named_arg(PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME);
    let project_token_price: u32 = runtime::get_named_arg(PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME);
    let project_token_total_supply: u32 =
        runtime::get_named_arg(PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME);
    let treasury_wallet: AccountHash = runtime::get_named_arg(TREASURY_WALLET_RUNTIME_ARG_NAME);
    let project_token_address: ContractHash =
        runtime::get_named_arg(PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME);
    let status = Status::Completed;

    let users_length = U256::from(0);
    let claim_status_key = {
        let uref = storage::new_dictionary(USERS_KEY_NAME).unwrap();
        Key::from(uref)
    };

    let project = Project::new(
        &project_id,
        &project_name,
        project_private,
        project_sale_start_time,
        project_sale_end_time,
        project_open_time,
        treasury_wallet,
        project_token_address,
        project_token_price,
        project_token_symbol,
        project_token_total_supply,
        status,
        claim_status_key,
        users_length,
    );

    project::write_project(project);
    runtime::ret(CLValue::from_t(true).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn get_project_info_by_id() {
    owner::only_owner();
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);

    let project = project::read_project(project_id.as_str());
    store_result(project.clone());
    runtime::ret(CLValue::from_t(project).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn set_project_status() {
    owner::only_owner();
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let project_status: Status = runtime::get_named_arg(PROJECT_STATUS_RUNTIME_ARG_NAME);
    project::write_project_field(project_id, PROJECT_STATUS_RUNTIME_ARG_NAME, project_status);
}

#[no_mangle]
pub extern "C" fn add_invest() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let account: Address = detail::get_caller_address().unwrap_or_revert();
    let amount: U256 = runtime::get_named_arg(CSPR_AMOUNT_RUNTIME_ARG_NAME);
    invests::write_invest_to(project_id, account, amount);
    // TODO receive cspr amount

    runtime::ret(CLValue::from_t(true).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn remove_invest() {}

#[no_mangle]
pub extern "C" fn get_invest_info() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let account: Address = detail::get_caller_address().unwrap_or_revert();
    let amount: U256 = invests::read_invest_from(project_id, account);
    store_result(amount);
    runtime::ret(CLValue::from_t(amount).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn claim() {
    // let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    // let account: Address = detail::get_caller_address().unwrap_or_revert();
    // let schedule_id = 5;
    // runtime::call_versioned_contract(
    //     contract_package_hash,
    //     contract_version,
    //     entry_point_name,
    //     runtime_args,
    // )
}

#[no_mangle]
pub extern "C" fn call() {
    // The key shouldn't already exist in the named keys.
    let missing_key = runtime::get_key(CONTRACT_NAME_KEY_NAME);
    if missing_key.is_some() {
        runtime::revert(Error::KeyAlreadyExists);
    }
    let missing_key = runtime::get_key(CONTRACT_NAME_KEY_NAME);
    if missing_key.is_some() {
        runtime::revert(Error::KeyAlreadyExists);
    }

    // Save named_keys
    let mut named_keys = NamedKeys::new();

    // Set Contract owner
    let owner_key: Key = {
        let owner: Address = detail::get_caller_address().unwrap_or_revert();
        let owner_uref: URef = storage::new_uref(owner).into_read_write();
        Key::from(owner_uref)
    };
    // Set default treasury wallet
    let default_treasury_wallet_key: Key = {
        let default_treasury_wallet_hash: AccountHash =
            runtime::get_named_arg(DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME);
        let default_treasury_wallet = Address::from(default_treasury_wallet_hash);
        let default_treasury_wallet_uref: URef =
            storage::new_uref(default_treasury_wallet).into_read_add_write();
        Key::from(default_treasury_wallet_uref)
    };
    // Initialize project dictionary
    let projects_dictionary_key: Key = {
        let uref: URef = storage::new_dictionary(PROJECTS_KEY_NAME).unwrap_or_revert();
        runtime::remove_key(PROJECTS_KEY_NAME);
        Key::from(uref)
    };

    let invests_dictionary_key: Key = {
        let uref: URef = storage::new_dictionary(INVESTS_KEY_NAME).unwrap_or_revert();
        runtime::remove_key(INVESTS_KEY_NAME);
        Key::from(uref)
    };

    named_keys.insert(OWNER_KEY_NAME.to_string(), owner_key);
    named_keys.insert(
        DEFAULT_TREASURY_WALLET_KEY_NAME.to_string(),
        default_treasury_wallet_key,
    );
    named_keys.insert(PROJECTS_KEY_NAME.to_string(), projects_dictionary_key);
    named_keys.insert(INVESTS_KEY_NAME.to_string(), invests_dictionary_key);

    let entry_points = entry_points::default();

    let (contract_hash, _version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(String::from(CONTRACT_NAME_KEY_NAME)),
        None,
    );
    let mut contract_hash_key_name: String = String::from(CONTRACT_NAME_KEY_NAME);
    contract_hash_key_name.push_str("_contract_hash");
    runtime::put_key(contract_hash_key_name.as_str(), Key::from(contract_hash));
}

// #[panic_handler]
// fn my_panic(_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }
