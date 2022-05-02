#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;
use crate::alloc::string::{String, ToString};

use alloc::vec::Vec;
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME, BALANCE_OF_ENTRY_POINT_NAME,
        RECIPIENT_RUNTIME_ARG_NAME, TRANSFER_ENTRY_POINT_NAME,
    },
    Address,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::NamedKeys,
    runtime_args, CLTyped, CLValue, ContractHash, Key, RuntimeArgs, URef, U256,
};
mod constants;
mod default_treasury_wallet;
mod detail;
mod entry_points;
mod error;
mod merkle_tree;
mod owner;
mod project;
mod projects;
use constants::{
    CONTRACT_NAME_KEY_NAME, CSPR_AMOUNT_RUNTIME_ARG_NAME, DEFAULT_TREASURY_WALLET_KEY_NAME,
    DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME, INVESTS_KEY_NAME, MERKLE_ROOT_KEY_NAME,
    MERKLE_ROOT_RUNTIME_ARG_NAME, OWNER_KEY_NAME, OWNER_RUNTIME_ARG_NAME, PROJECTS_KEY_NAME,
    PROJECT_CAPACITY_USD_RUNTIME_ARG_NAME, PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME,
    PROJECT_ID_RUNTIME_ARG_NAME, PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME,
    PROJECT_NAME_RUNTIME_ARG_NAME, PROJECT_OPEN_TIME_RUNTIME_ARG_NAME,
    PROJECT_PRIVATE_RUNTIME_ARG_NAME, PROJECT_REWARD_MULTIPLY_RUNTIME_ARG_NAME,
    PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME, PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME,
    PROJECT_SCHEDULES_RUNTIME_ARG_NAME, PROJECT_STATUS_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME, PROJECT_USERS_RUNTIME_ARG_NAME,
    RESULT_KEY_NAME, TREASURY_WALLET_RUNTIME_ARG_NAME,
};
use error::Error;
use project::{Project, Schedule, Status};

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
pub extern "C" fn set_merkle_root() {
    owner::only_owner();
    // let new_merkle_root: &[u8] = runtime::get_named_arg(MERKLE_ROOT_RUNTIME_ARG_NAME);
    // let merkle_tree_root_uref = merkle_tree::merkle_tree_root_uref();
    // merkle_tree::write_merkle_tree_root_to(merkle_tree_root_uref, new_merkle_root);
}

#[no_mangle]
pub extern "C" fn get_merkle_root() {
    owner::only_owner();
    let merkle_tree_root_uref = merkle_tree::merkle_tree_root_uref();
    let merkle_root: Vec<u8> = merkle_tree::read_merkle_tree_root_from(merkle_tree_root_uref);
    store_result(merkle_root.clone());
    runtime::ret(CLValue::from_t(merkle_root).unwrap_or_revert());
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
    let project_token_price: U256 =
        runtime::get_named_arg(PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME);
    let project_token_total_supply: U256 =
        runtime::get_named_arg(PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME);
    let treasury_wallet: AccountHash = runtime::get_named_arg(TREASURY_WALLET_RUNTIME_ARG_NAME);
    let project_token_address: ContractHash =
        runtime::get_named_arg(PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME);
    let status = Status::Upcoming;

    let reward_multiply: U256 = runtime::get_named_arg(PROJECT_REWARD_MULTIPLY_RUNTIME_ARG_NAME);
    let locked_token_amount: U256 = runtime::call_contract(
        project_token_address,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            ADDRESS_RUNTIME_ARG_NAME => detail::get_caller_address().unwrap_or_revert()
        },
    );

    let unlocked_token_amount: U256 = U256::from(0);
    let users_length = U256::from(0);

    let claim_status_key = {
        let dictionary_name =
            project::make_dictionary_item_key(PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME.to_string());
        let uref = storage::new_dictionary(&dictionary_name).unwrap();
        Key::from(uref)
    };

    let users_key = {
        let dictionary_name =
            project::make_dictionary_item_key(PROJECT_USERS_RUNTIME_ARG_NAME.to_string());
        let uref = storage::new_dictionary(&dictionary_name).unwrap();
        Key::from(uref)
    };

    let capacity_usd: U256 = runtime::get_named_arg(PROJECT_CAPACITY_USD_RUNTIME_ARG_NAME);
    let schedules: Vec<Schedule> = runtime::get_named_arg(PROJECT_SCHEDULES_RUNTIME_ARG_NAME);
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
        capacity_usd,
        locked_token_amount,
        unlocked_token_amount,
        status,
        claim_status_key,
        reward_multiply,
        users_key,
        users_length,
        schedules,
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
    let projects_uref = projects::get_projects_uref();
    projects::only_exist_project(projects_uref, project_id.clone());
    let project_status: u32 = runtime::get_named_arg(PROJECT_STATUS_RUNTIME_ARG_NAME);
    project::write_project_field(
        project_id,
        PROJECT_STATUS_RUNTIME_ARG_NAME,
        Status::from_u32(project_status),
    );
}

#[no_mangle]
pub extern "C" fn add_invest() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    project::only_active_project(project_id.as_str());
    // project::only_sale_time(project_id.as_str());

    let account: Address = detail::get_immediate_caller_address().unwrap_or_revert();
    let amount: U256 = runtime::get_named_arg(CSPR_AMOUNT_RUNTIME_ARG_NAME);
    let users_dic_key: Key =
        project::read_project_field(project_id.as_str(), PROJECT_USERS_RUNTIME_ARG_NAME);
    // TODO receive cspr amount
    let dictionary_item_key = project::make_users_dictionary_item_key(account);
    let users_dic_uref = *users_dic_key.as_uref().unwrap();
    let invest_amount: U256 = storage::dictionary_get(users_dic_uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap_or_default();
    let new_invest_amount = invest_amount + amount;
    storage::dictionary_put(users_dic_uref, &dictionary_item_key, amount);

    runtime::ret(CLValue::from_t(new_invest_amount).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn remove_invest() {}

#[no_mangle]
pub extern "C" fn get_invest_info() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let account: Address = detail::get_immediate_caller_address().unwrap_or_revert();
    let users_dic_key: Key =
        project::read_project_field(project_id.as_str(), PROJECT_USERS_RUNTIME_ARG_NAME);
    // TODO receive cspr amount
    let dictionary_item_key = project::make_users_dictionary_item_key(account);
    let invest_amount: U256 =
        storage::dictionary_get(*users_dic_key.as_uref().unwrap(), &dictionary_item_key)
            .unwrap_or_revert()
            .unwrap_or_default();
    store_result(invest_amount);
    runtime::ret(CLValue::from_t(invest_amount).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn claim() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let account: Address = detail::get_immediate_caller_address().unwrap_or_revert();
    let project_token_address: ContractHash =
        project::read_project_field(project_id.as_str(), PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME);
    let schedule_id = 5;
    let transfer_amount = U256::from(500u64);
    let schedules: Vec<Schedule> =
        project::read_project_field(project_id.as_str(), PROJECT_SCHEDULES_RUNTIME_ARG_NAME);
    runtime::call_contract::<()>(
        project_token_address,
        TRANSFER_ENTRY_POINT_NAME,
        runtime_args! {
            RECIPIENT_RUNTIME_ARG_NAME => account,
            AMOUNT_RUNTIME_ARG_NAME => transfer_amount
        },
    );

    runtime::ret(CLValue::from_t(transfer_amount).unwrap_or_revert());
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

    let merkle_root_key: Key = {
        let mut v: Vec<u8> = Vec::new();
        v.push(5);
        let uref: URef = storage::new_uref(v);
        Key::from(uref)
    };

    named_keys.insert(OWNER_KEY_NAME.to_string(), owner_key);
    named_keys.insert(
        DEFAULT_TREASURY_WALLET_KEY_NAME.to_string(),
        default_treasury_wallet_key,
    );
    named_keys.insert(PROJECTS_KEY_NAME.to_string(), projects_dictionary_key);
    named_keys.insert(INVESTS_KEY_NAME.to_string(), invests_dictionary_key);
    named_keys.insert(MERKLE_ROOT_KEY_NAME.to_string(), merkle_root_key);

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
