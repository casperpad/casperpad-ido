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
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::{
    constants::{
        ADDRESS_RUNTIME_ARG_NAME, ALLOWANCE_ENTRY_POINT_NAME, AMOUNT_RUNTIME_ARG_NAME,
        BALANCE_OF_ENTRY_POINT_NAME, DECIMALS_ENTRY_POINT_NAME, RECIPIENT_RUNTIME_ARG_NAME,
        SPENDER_RUNTIME_ARG_NAME, SYMBOL_ENTRY_POINT_NAME, TOTAL_SUPPLY_ENTRY_POINT_NAME,
        TRANSFER_ENTRY_POINT_NAME, TRANSFER_FROM_ENTRY_POINT_NAME,
    },
    Address,
};
use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, CLValue, ContractHash, Key,
    RuntimeArgs, URef, U256, U512,
};
mod claims;
mod constants;
mod detail;
mod entry_points;
mod error;
mod invests;
mod merkle_tree;
mod owner;
mod project;
mod projects;
mod purse;
mod tiers;

use constants::{
    CLAIMS_KEY_NAME, CONTRACT_NAME_KEY_NAME, CSPR_AMOUNT_RUNTIME_ARG_NAME,
    CSPR_PRICE_RUNTIME_ARG_NAME, INVESTS_KEY_NAME, MERKLE_ROOT_KEY_NAME,
    MERKLE_ROOT_RUNTIME_ARG_NAME, MULTIPLE_TIERS_RUNTIME_ARG_NAME, OWNER_KEY_NAME,
    OWNER_RUNTIME_ARG_NAME, PROJECTS_KEY_NAME, PROJECT_ID_RUNTIME_ARG_NAME,
    PROJECT_NAME_RUNTIME_ARG_NAME, PROJECT_OPEN_TIME_RUNTIME_ARG_NAME,
    PROJECT_PRIVATE_RUNTIME_ARG_NAME, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME,
    PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME, PROJECT_SCHEDULES_RUNTIME_ARG_NAME,
    PROJECT_STATUS_RUNTIME_ARG_NAME, PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_CAPACITY_RUNTIME_ARG_NAME, PROJECT_TOKEN_DECIMALS_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME, PROJECT_TOTAL_INVESTS_AMOUNT_RUNTIME_ARG_NAME,
    PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME, PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME,
    PROOF_RUNTIME_ARG_NAME, PURSE_KEY_NAME, SCHEDULE_ID_RUNTIME_ARG_NAME, TIERS_KEY_NAME,
    TIER_RUNTIME_ARG_NAME, TREASURY_WALLET_RUNTIME_ARG_NAME,
};

use detail::store_result;
use error::Error;
use project::{Project, Status};

#[no_mangle]
pub extern "C" fn transfer_ownership() {
    owner::only_owner();
    let new_owner_hash: AccountHash = {
        let new_owner_string: String = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
        AccountHash::from_formatted_str(new_owner_string.as_str()).unwrap()
    };
    let owner_uref: URef = owner::owner_uref();
    owner::write_owner_to(owner_uref, new_owner_hash);
    runtime::ret(CLValue::from_t(true).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn add_project() {
    owner::only_owner();
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    projects::only_not_exist_project(project_id.clone());
    let project_name: String = runtime::get_named_arg(PROJECT_NAME_RUNTIME_ARG_NAME);
    let project_sale_start_time: i64 =
        runtime::get_named_arg(PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME);
    let project_sale_end_time: i64 = runtime::get_named_arg(PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME);
    let project_open_time: i64 = runtime::get_named_arg(PROJECT_OPEN_TIME_RUNTIME_ARG_NAME);
    let project_private: bool = runtime::get_named_arg(PROJECT_PRIVATE_RUNTIME_ARG_NAME);
    let project_token_price: U256 =
        runtime::get_named_arg(PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME);
    let treasury_wallet: AccountHash = {
        let treasury_wallet_key: String = runtime::get_named_arg(TREASURY_WALLET_RUNTIME_ARG_NAME);
        AccountHash::from_formatted_str(treasury_wallet_key.as_str()).unwrap()
    };

    let project_token_address: ContractHash = {
        let project_token_address_key: Key =
            runtime::get_named_arg(PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME);
        let project_token_address_hash = project_token_address_key.into_hash().unwrap();
        ContractHash::new(project_token_address_hash)
    };
    let status = Status::Pending;

    let schedules: Vec<(i64, U256)> = runtime::get_named_arg(PROJECT_SCHEDULES_RUNTIME_ARG_NAME);
    let cspr_price: U256 = U256::zero();

    let project_token_symbol: String = runtime::call_contract(
        project_token_address,
        SYMBOL_ENTRY_POINT_NAME,
        runtime_args! {},
    );
    let project_token_decimals: u8 = runtime::call_contract(
        project_token_address,
        DECIMALS_ENTRY_POINT_NAME,
        runtime_args! {},
    );
    let project_token_total_supply: U256 = runtime::call_contract(
        project_token_address,
        TOTAL_SUPPLY_ENTRY_POINT_NAME,
        runtime_args! {},
    );

    let locked_token_amount: U256 = {
        let amount_to_lock: U256 = runtime::get_named_arg(PROJECT_TOKEN_CAPACITY_RUNTIME_ARG_NAME);
        let allownce_token_amount: U256 = runtime::call_contract(
            project_token_address,
            ALLOWANCE_ENTRY_POINT_NAME,
            runtime_args! {
                OWNER_RUNTIME_ARG_NAME => detail::get_immediate_caller_address().unwrap_or_revert(),
                SPENDER_RUNTIME_ARG_NAME => detail::get_caller_address().unwrap_or_revert()
            },
        );
        if amount_to_lock > allownce_token_amount {
            runtime::revert(Error::InsufficientAllowance);
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

    let unlocked_token_amount: U256 = U256::from(0u32);
    let users_length = U256::from(0u32);
    let total_invests_amount = U256::from(0u32);
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
        project_token_decimals,
        project_token_symbol,
        project_token_total_supply,
        locked_token_amount,
        unlocked_token_amount,
        status,
        users_length,
        schedules,
        cspr_price,
        total_invests_amount,
    );

    project::write_project(project);
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
pub extern "C" fn set_cspr_price() {
    owner::only_owner();

    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let cspr_price: U256 = runtime::get_named_arg(CSPR_PRICE_RUNTIME_ARG_NAME);

    let projects_uref = projects::get_projects_uref();
    projects::only_exist_project(projects_uref, project_id.clone());
    project::write_project_field(project_id, CSPR_PRICE_RUNTIME_ARG_NAME, cspr_price);
}

/// Add invest to project befor invest admin must set cspr price by calling set_cspr_price.
#[no_mangle]
pub extern "C" fn add_invest() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let proof: Vec<(String, u8)> = runtime::get_named_arg(PROOF_RUNTIME_ARG_NAME);
    let amount_u512: U512 = runtime::get_named_arg(CSPR_AMOUNT_RUNTIME_ARG_NAME);

    let purse_uref: URef = purse::get_main_purse();
    let purse_balance = system::get_purse_balance(purse_uref).unwrap_or_default();

    if purse_balance.lt(&amount_u512) || amount_u512.eq(&U512::zero()) {
        runtime::revert(Error::InsufficientBalance);
    }

    // Since amount is used to calculate erc20 amount we need to convert data type
    let amount = U256::from(amount_u512.as_u128());

    let account: AccountHash = *detail::get_immediate_caller_address()
        .unwrap_or_revert()
        .as_account_hash()
        .unwrap_or_revert();

    merkle_tree::verify_whitelist(proof);
    project::only_sale_time(project_id.as_str());

    // Update user invest amount
    let invest_amount: U256 = invests::read_invest_from(project_id.clone(), account);
    if invest_amount.eq(&U256::zero()) {
        // This is first invest, so increase participated user count
        let users_length: U256 =
            project::read_project_field(project_id.as_str(), PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME);
        let new_users_length = users_length.checked_add(U256::one()).unwrap_or_revert();
        project::write_project_field(
            project_id.clone(),
            PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME,
            new_users_length,
        );
    }
    let new_invest_amount: U256 = invest_amount.checked_add(amount).unwrap_or_revert();
    invests::write_invest_to(project_id.clone(), account, new_invest_amount);

    // Update total_invests_amount
    let current_invests_amount: U256 = project::read_project_field(
        project_id.as_str(),
        PROJECT_TOTAL_INVESTS_AMOUNT_RUNTIME_ARG_NAME,
    );
    let total_invests_amount = current_invests_amount
        .checked_add(amount)
        .unwrap_or_revert();
    project::write_project_field(
        project_id.clone(),
        PROJECT_TOTAL_INVESTS_AMOUNT_RUNTIME_ARG_NAME,
        total_invests_amount,
    );

    // Transfer CSPR
    let treasury_wallet: AccountHash =
        project::read_project_field(project_id.as_str(), TREASURY_WALLET_RUNTIME_ARG_NAME);

    let cspr_transfer_amount = U512::from(amount.as_u128());

    system::transfer_from_purse_to_account(
        purse_uref,
        treasury_wallet,
        cspr_transfer_amount, // Consider this convert
        None,
    )
    .unwrap_or_revert();
}

// #[no_mangle]
// pub extern "C" fn remove_invest() {}

#[no_mangle]
pub extern "C" fn claim() {
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let schedule_id: u8 = runtime::get_named_arg(SCHEDULE_ID_RUNTIME_ARG_NAME);
    let account: AccountHash = *detail::get_immediate_caller_address()
        .unwrap_or_revert()
        .as_account_hash()
        .unwrap_or_revert();
    let cliams_dictionary_uref = claims::claims_uref();
    let claim_amount: U256 = claims::read_claim_from(project_id.clone(), account, schedule_id);

    if claim_amount.gt(&U256::zero()) {
        runtime::revert(Error::AlreadyClaimed);
    }

    let schedules: Vec<(i64, U256)> =
        project::read_project_field(project_id.as_str(), PROJECT_SCHEDULES_RUNTIME_ARG_NAME);

    let schedule_to_claim = *schedules.get(usize::from(schedule_id)).unwrap_or_revert();
    project::only_after_time(schedule_to_claim.0);

    let project_token_address: ContractHash =
        project::read_project_field(project_id.as_str(), PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME);

    // Get user vest amount
    let invest_amount = invests::read_invest_from(project_id.clone(), account); // cspr amount
    let percent_decimal = U256::exp10(5); // Percent decimal is 5
    let transfer_amount_in_cspr = invest_amount
        .checked_mul(schedule_to_claim.1)
        .unwrap_or_default()
        .checked_div(percent_decimal)
        .unwrap_or_default();

    let cspr_price_in_usd: U256 = {
        // this value should be with moute 0.5* 10 **18=> 1cspr = 0.5 usd
        let project_uref = project::project_dictionary_uref(project_id.clone());
        project::read_project_field(project_id.as_str(), CSPR_PRICE_RUNTIME_ARG_NAME)
    };

    let token_price_in_usd: U256 = {
        // this value should be with moute 0.01* 10 ** 18=> 1token = 0.01usd
        let project_uref = project::project_dictionary_uref(project_id.clone());
        project::read_project_field(
            project_id.as_str(),
            PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
        )
    };

    let token_price_in_cspr: U256 = token_price_in_usd
        .checked_mul(U256::exp10(9))
        .unwrap()
        .checked_div(cspr_price_in_usd)
        .unwrap();

    let transfer_amount = {
        let token_decimals: u8 = project::read_project_field(
            project_id.as_str(),
            PROJECT_TOKEN_DECIMALS_RUNTIME_ARG_NAME,
        );
        transfer_amount_in_cspr
            .checked_div(token_price_in_cspr)
            .unwrap()
            .checked_mul(U256::exp10(usize::from(token_decimals)))
            .unwrap()
    };

    let token_balance_of_contract: U256 = runtime::call_contract(
        project_token_address,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            ADDRESS_RUNTIME_ARG_NAME => detail::get_caller_address().unwrap_or_revert()
        },
    );
    if transfer_amount > token_balance_of_contract {
        runtime::revert(Error::InsufficientBalance);
    }

    runtime::call_contract::<()>(
        project_token_address,
        TRANSFER_ENTRY_POINT_NAME,
        runtime_args! {
            RECIPIENT_RUNTIME_ARG_NAME => Address::from(account),
            AMOUNT_RUNTIME_ARG_NAME => transfer_amount
        },
    );

    // Update project total unlocked amount
    let unlocked_token_amount: U256 = project::read_project_field(
        project_id.as_str(),
        PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME,
    );
    project::write_project_field(
        project_id.clone(),
        PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME,
        unlocked_token_amount
            .checked_add(transfer_amount)
            .unwrap_or_revert(),
    );

    // Write user`s claim status
    claims::write_claim_to(project_id, account, schedule_id, transfer_amount);
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

/// set multiple tiers `Vec<(account-hash-0000000...00, project_id, amount)>`
#[no_mangle]
pub extern "C" fn set_multiple_tiers() {
    owner::only_owner();
    let tiers: Vec<(String, String, U256)> =
        runtime::get_named_arg(MULTIPLE_TIERS_RUNTIME_ARG_NAME);
    tiers::write_multiple_tiers(tiers);
}

#[no_mangle]
pub extern "C" fn set_tier() {
    owner::only_owner();
    let tier: (String, String, U256) = runtime::get_named_arg(TIER_RUNTIME_ARG_NAME);
    tiers::write_tier(tier);
}

#[no_mangle]
pub extern "C" fn get_purse() {
    let purse_uref: URef = purse::get_main_purse();
    runtime::ret(CLValue::from_t(purse_uref).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn set_purse() {
    let purse_uref: URef = system::create_purse();
    purse::set_main_purse(purse_uref);
}

#[no_mangle]
pub extern "C" fn call() {
    // Save named_keys
    let mut named_keys = NamedKeys::new();

    // Set Contract owner
    let owner_key: Key = {
        let owner: AccountHash = runtime::get_caller();
        let owner_uref: URef = storage::new_uref(owner).into_read_write();
        Key::from(owner_uref)
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

    let tiers_dictionary_key: Key = {
        let uref: URef = storage::new_dictionary(TIERS_KEY_NAME).unwrap_or_revert();
        runtime::remove_key(TIERS_KEY_NAME);
        Key::from(uref)
    };
    // Merkle
    let merkle_root_key: Key = {
        let root: [u8; 32] = [0u8; 32];
        let uref: URef = storage::new_uref(root);
        Key::from(uref)
    };

    // Main Purse
    let purse_key = {
        let purse_uref = system::create_purse();
        let uref = storage::new_uref(Key::from(purse_uref)).into_read();
        Key::from(uref)
    };

    named_keys.insert(OWNER_KEY_NAME.to_string(), owner_key);
    named_keys.insert(PROJECTS_KEY_NAME.to_string(), projects_dictionary_key);
    named_keys.insert(MERKLE_ROOT_KEY_NAME.to_string(), merkle_root_key);
    named_keys.insert(PURSE_KEY_NAME.to_string(), purse_key);
    named_keys.insert(INVESTS_KEY_NAME.to_string(), invests_dictionary_key);
    named_keys.insert(CLAIMS_KEY_NAME.to_string(), claims_dictionary_key);
    named_keys.insert(TIERS_KEY_NAME.to_string(), tiers_dictionary_key);

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
