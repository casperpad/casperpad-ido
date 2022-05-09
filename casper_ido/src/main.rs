#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;
use core::convert::TryInto;

use crate::alloc::string::{String, ToString};

use alloc::{boxed::Box, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, ALLOWANCE_ENTRY_POINT_NAME, AMOUNT_RUNTIME_ARG_NAME,
        BALANCE_OF_ENTRY_POINT_NAME, DECIMALS_ENTRY_POINT_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SPENDER_RUNTIME_ARG_NAME, TRANSFER_ENTRY_POINT_NAME, TRANSFER_FROM_ENTRY_POINT_NAME,
    },
    Address,
};
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, contracts::NamedKeys, runtime_args, CLType, CLTyped,
    CLValue, ContractHash, HashAddr, Key, RuntimeArgs, URef, U256,
};
// use std::{collections::BTreeMap, convert::TryInto};
mod claims;
mod constants;
mod default_treasury_wallet;
mod detail;
mod entry_points;
mod error;
mod invests;
mod merkle_tree;
mod owner;
mod project;
mod projects;

use constants::{
    CLAIMS_KEY_NAME, CONTRACT_NAME_KEY_NAME, CSPR_AMOUNT_RUNTIME_ARG_NAME,
    DEFAULT_TREASURY_WALLET_KEY_NAME, DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME, INVESTS_KEY_NAME,
    MERKLE_ROOT_KEY_NAME, MERKLE_ROOT_RUNTIME_ARG_NAME, OWNER_KEY_NAME, OWNER_RUNTIME_ARG_NAME,
    PROJECTS_KEY_NAME, PROJECT_CAPACITY_USD_RUNTIME_ARG_NAME,
    PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME, PROJECT_ID_RUNTIME_ARG_NAME,
    PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME, PROJECT_NAME_RUNTIME_ARG_NAME,
    PROJECT_OPEN_TIME_RUNTIME_ARG_NAME, PROJECT_PRIVATE_RUNTIME_ARG_NAME,
    PROJECT_REWARD_MULTIPLY_RUNTIME_ARG_NAME, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME,
    PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME, PROJECT_SCHEDULES_RUNTIME_ARG_NAME,
    PROJECT_STATUS_RUNTIME_ARG_NAME, PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME, PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME, PROJECT_USERS_RUNTIME_ARG_NAME,
    PROOF_RUNTIME_ARG_NAME, PURSE_KEY_NAME, RESULT_KEY_NAME, SCHEDULE_ID_RUNTIME_ARG_NAME,
    TREASURY_WALLET_RUNTIME_ARG_NAME,
};

use error::Error;
use project::{Project, Status};

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
    let new_default_treasury_wallet: AccountHash =
        runtime::get_named_arg(DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME);
    let default_treasury_wallet_uref = default_treasury_wallet::default_treasury_wallet_uref();
    default_treasury_wallet::write_default_tresury_wallet_to(
        default_treasury_wallet_uref,
        new_default_treasury_wallet,
    )
}

#[no_mangle]
pub extern "C" fn add_project() {
    // TODO only verified project creators!!!!
    // owner::only_owner();
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    projects::only_not_exist_project(project_id.clone());
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
    let treasury_wallet_key: Key = runtime::get_named_arg(TREASURY_WALLET_RUNTIME_ARG_NAME);
    let treasury_wallet_hash: HashAddr = treasury_wallet_key.into_hash().unwrap();
    let treasury_wallet = AccountHash::new(treasury_wallet_hash);

    let project_token_address = {
        let project_token_address_key: Key =
            runtime::get_named_arg(PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME);
        let project_token_address_hash = project_token_address_key.into_hash().unwrap();
        ContractHash::new(project_token_address_hash)
    };
    let status = Status::Upcoming;
    let reward_multiply: U256 = runtime::get_named_arg(PROJECT_REWARD_MULTIPLY_RUNTIME_ARG_NAME);
    let capacity_usd: U256 = runtime::get_named_arg(PROJECT_CAPACITY_USD_RUNTIME_ARG_NAME);
    let schedules: Vec<(i64, U256)> = runtime::get_named_arg(PROJECT_SCHEDULES_RUNTIME_ARG_NAME);

    let locked_token_amount: U256 = {
        let amount_to_lock: U256 =
            runtime::get_named_arg(PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME);
        let allownce_token_amount: U256 = runtime::call_contract(
            project_token_address,
            ALLOWANCE_ENTRY_POINT_NAME,
            runtime_args! {
                OWNER_RUNTIME_ARG_NAME => detail::get_immediate_caller_address().unwrap_or_revert(),
                SPENDER_RUNTIME_ARG_NAME => detail::get_caller_address().unwrap_or_revert()
            },
        );
        if amount_to_lock > allownce_token_amount {
            runtime::revert(Error::InsufficientAllowance)
        }
        runtime::call_contract::<()>(
            project_token_address,
            TRANSFER_FROM_ENTRY_POINT_NAME,
            runtime_args! {
                OWNER_RUNTIME_ARG_NAME => detail::get_immediate_caller_address().unwrap_or_revert(),
                RECIPIENT_RUNTIME_ARG_NAME => detail::get_caller_address().unwrap_or_revert(),
                AMOUNT_RUNTIME_ARG_NAME => amount_to_lock
            },
        );
        runtime::call_contract(
            project_token_address,
            BALANCE_OF_ENTRY_POINT_NAME,
            runtime_args! {
                ADDRESS_RUNTIME_ARG_NAME => detail::get_caller_address().unwrap_or_revert()
            },
        )
    };

    let unlocked_token_amount: U256 = U256::from(0);
    let users_length = U256::from(0);

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
        reward_multiply,
        users_length,
        schedules,
    );

    project::write_project(project);
    runtime::ret(CLValue::from_t(true).unwrap_or_revert());
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
    let proof: Vec<(String, u8)> = runtime::get_named_arg(PROOF_RUNTIME_ARG_NAME);

    let account: AccountHash = *detail::get_immediate_caller_address()
        .unwrap_or_revert()
        .as_account_hash()
        .unwrap_or_revert();

    merkle_tree::verify_whitelist(proof);
    // project::only_sale_time(project_id.as_str());

    // Read user invest amount

    // let amount: U256 = runtime::get_named_arg(CSPR_AMOUNT_RUNTIME_ARG_NAME);

    // let invest_amount = invests::read_invest_from(project_id.clone(), account);

    // let new_invest_amount = invest_amount + amount;
    // invests::write_invest_to(project_id, account, new_invest_amount);

    // runtime::ret(CLValue::from_t(new_invest_amount).unwrap_or_revert());
}

// #[no_mangle]
// pub extern "C" fn remove_invest() {}

#[no_mangle]
pub extern "C" fn claim() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let schedule_id: u8 = runtime::get_named_arg(SCHEDULE_ID_RUNTIME_ARG_NAME);
    // let schedule_id = 0;
    let schedules: Vec<(i64, U256)> =
        project::read_project_field(project_id.as_str(), PROJECT_SCHEDULES_RUNTIME_ARG_NAME);

    let schedule_to_claim = *schedules.get(usize::from(schedule_id)).unwrap_or_revert();
    project::only_after_time(schedule_to_claim.0);

    let project_token_address: ContractHash =
        project::read_project_field(project_id.as_str(), PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME);

    let token_decimal: u8 = runtime::call_contract(
        project_token_address,
        DECIMALS_ENTRY_POINT_NAME,
        runtime_args! {},
    );

    // Get user vest amount

    let account: AccountHash = *detail::get_immediate_caller_address()
        .unwrap_or_revert()
        .as_account_hash()
        .unwrap_or_revert();

    let invest_amount = invests::read_invest_from(project_id.clone(), account);
    let percent_decimal = U256::from(1000);
    let transfer_amount_in_cspr = invest_amount
        .checked_mul(percent_decimal) // Percent decimal is 3
        .unwrap_or_default()
        .checked_div(schedule_to_claim.1)
        .unwrap_or_default();

    // TODO: Consider calc method
    let transfer_amount = U256::from(20i32);
    let locked_token_amount: U256 = runtime::call_contract(
        project_token_address,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            ADDRESS_RUNTIME_ARG_NAME => detail::get_caller_address().unwrap_or_revert()
        },
    );
    if transfer_amount > locked_token_amount {
        runtime::revert(Error::InsufficientBalance);
    }
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
pub extern "C" fn emergency_claim() {}

#[no_mangle]
pub extern "C" fn set_merkle_root() {
    owner::only_owner();
    let new_merkle_root: String = runtime::get_named_arg(MERKLE_ROOT_RUNTIME_ARG_NAME);
    let merkle_tree_root_uref = merkle_tree::merkle_tree_root_uref();
    merkle_tree::write_merkle_tree_root_to(merkle_tree_root_uref, new_merkle_root);
}

#[no_mangle]
pub extern "C" fn get_purse() {
    let purse_uref: URef = detail::get_uref(PURSE_KEY_NAME);
    runtime::ret(CLValue::from_t(purse_uref).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn call() {
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
        let default_treasury_wallet = *detail::get_caller_address()
            .unwrap_or_revert()
            .as_account_hash()
            .unwrap();
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
    let claims_dictionary_key: Key = {
        let uref: URef = storage::new_dictionary(CLAIMS_KEY_NAME).unwrap_or_revert();
        runtime::remove_key(CLAIMS_KEY_NAME);
        Key::from(uref)
    };
    // Merkle
    let merkle_root_key: Key = {
        let mut v: Vec<u8> = Vec::new();
        v.push(5);
        v.push(12);
        v.push(234);
        v.push(43);
        v.push(67);
        v.push(63);
        let uref: URef = storage::new_uref(v);
        Key::from(uref)
    };

    // Main Purse
    let purse_key = {
        let purse_uref = system::create_purse();
        Key::from(purse_uref)
    };

    named_keys.insert(OWNER_KEY_NAME.to_string(), owner_key);
    named_keys.insert(
        DEFAULT_TREASURY_WALLET_KEY_NAME.to_string(),
        default_treasury_wallet_key,
    );
    named_keys.insert(PROJECTS_KEY_NAME.to_string(), projects_dictionary_key);
    named_keys.insert(MERKLE_ROOT_KEY_NAME.to_string(), merkle_root_key);
    named_keys.insert(PURSE_KEY_NAME.to_string(), purse_key);
    named_keys.insert(INVESTS_KEY_NAME.to_string(), invests_dictionary_key);
    named_keys.insert(CLAIMS_KEY_NAME.to_string(), claims_dictionary_key);

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