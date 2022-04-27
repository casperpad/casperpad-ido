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
use casper_erc20::Address;
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, contracts::NamedKeys, CLTyped, CLValue, Key, URef,
    U256,
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
mod treasury_wallets;
use constants::{
    CONTRACT_NAME_KEY_NAME, CSPR_AMOUNT_RUNTIME_ARG_NAME, DEFAULT_TREASURY_WALLET_KEY_NAME,
    DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME, OWNER_KEY_NAME, OWNER_RUNTIME_ARG_NAME,
    PROJECTS_KEY_NAME, PROJECT_END_TIME_RUNTIME_ARG_NAME, PROJECT_ID_RUNTIME_ARG_NAME,
    PROJECT_NAME_RUNTIME_ARG_NAME, PROJECT_PRIVATE_RUNTIME_ARG_NAME, PROJECT_RUNTIME_ARG_NAME,
    PROJECT_START_TIME_RUNTIME_ARG_NAME, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    RESULT_KEY_NAME, USERS_KEY_NAME, WALLET_RUNTIME_ARG_NAME,
};
use error::Error;
use project::{Project, TokenInfo};

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
    let caller: Address = detail::get_immediate_caller_address().unwrap_or_revert();
    let owner_uref: URef = owner::owner_uref();
    let current_owner: Address = owner::read_owner_from(owner_uref);
    if caller != current_owner {
        runtime::revert(Error::PermissionDenied)
    }
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
    runtime::ret(CLValue::from_t(default_treasury_wallet).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn set_project_treasury_wallet() {
    let project: Address = runtime::get_named_arg(PROJECT_RUNTIME_ARG_NAME);
    let wallet: Address = runtime::get_named_arg(WALLET_RUNTIME_ARG_NAME);
    let treasury_wallets_uref = treasury_wallets::get_wallets_uref();
    treasury_wallets::write_wallets_to(treasury_wallets_uref, project, wallet);
}

#[no_mangle]
pub extern "C" fn get_project_treasury_wallet() {
    let project: Address = runtime::get_named_arg(PROJECT_RUNTIME_ARG_NAME);
    let treasury_wallets_uref = treasury_wallets::get_wallets_uref();
    let wallet_address = treasury_wallets::read_wallets_from(treasury_wallets_uref, project);
    runtime::ret(CLValue::from_t(wallet_address).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn add_project() {
    owner::only_owner();
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let project_name: String = runtime::get_named_arg(PROJECT_NAME_RUNTIME_ARG_NAME);
    let project_start_time: i64 = runtime::get_named_arg(PROJECT_START_TIME_RUNTIME_ARG_NAME);
    let project_end_time: i64 = runtime::get_named_arg(PROJECT_END_TIME_RUNTIME_ARG_NAME);
    let project_private: bool = runtime::get_named_arg(PROJECT_PRIVATE_RUNTIME_ARG_NAME);
    let project_token_symbol: String =
        runtime::get_named_arg(PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME);
    let project_token_price: u32 = runtime::get_named_arg(PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME);
    let project_token_total_supply: u32 =
        runtime::get_named_arg(PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME);
    let token_info = TokenInfo {
        token_price: project_token_price,
        token_symbol: project_token_symbol,
        total_supply: project_token_total_supply,
    };

    let status_key = {
        let uref = storage::new_uref(U256::from(0i32)).into_read_write();
        Key::from(uref)
    };

    let users_length_key = {
        let uref = storage::new_uref(U256::from(0i32)).into_read_write();
        Key::from(uref)
    };
    let claim_status_key = {
        let uref = storage::new_dictionary(USERS_KEY_NAME).unwrap();
        Key::from(uref)
    };

    let project = Project::new(
        &project_id,
        &project_name,
        project_private,
        project_start_time,
        project_end_time,
        token_info,
        status_key,
        claim_status_key,
        users_length_key,
    );
    let projects_uref = projects::get_projects_uref();
    projects::write_project_to(projects_uref, project_id, project);
    runtime::ret(CLValue::from_t(true).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn get_project_info_by_id() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);

    let projects_uref = projects::get_projects_uref();
    let project = projects::read_project_from(projects_uref, project_id);
    store_result(project.clone());
    runtime::ret(CLValue::from_t(project).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn add_invest() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let account: Address = detail::get_caller_address().unwrap_or_revert();
    let amount: U256 = runtime::get_named_arg(CSPR_AMOUNT_RUNTIME_ARG_NAME);
    invests::write_invest_to(project_id, account, amount);
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
    let uref: URef = storage::new_dictionary(PROJECTS_KEY_NAME).unwrap_or_revert();
    let projects_dictionary_key: Key = {
        runtime::remove_key(PROJECTS_KEY_NAME);
        Key::from(uref)
    };

    named_keys.insert(OWNER_KEY_NAME.to_string(), owner_key);
    named_keys.insert(
        DEFAULT_TREASURY_WALLET_KEY_NAME.to_string(),
        default_treasury_wallet_key,
    );
    named_keys.insert(PROJECTS_KEY_NAME.to_string(), projects_dictionary_key);

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
