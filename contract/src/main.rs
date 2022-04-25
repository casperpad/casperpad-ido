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
use casper_types::{account::AccountHash, contracts::NamedKeys, CLValue, Key, URef};

mod constants;
mod default_treasury_wallet;
mod detail;
mod entry_points;
mod error;
mod owner;
mod treasury_wallets;
use constants::{
    CONTRACT_NAME_KEY_NAME, DEFAULT_TREASURY_WALLET_KEY_NAME,
    DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME, OWNER_KEY_NAME, OWNER_RUNTIME_ARG_NAME,
    PROJECT_RUNTIME_ARG_NAME, WALLET_RUNTIME_ARG_NAME,
};
use error::Error;

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
    let owner: Address = detail::get_caller_address().unwrap_or_revert();
    let owner_uref: URef = storage::new_uref(owner).into_read_write();
    let owner_key: Key = Key::from(owner_uref);
    // Set default treasury wallet
    let default_treasury_wallet_hash: AccountHash =
        runtime::get_named_arg(DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME);
    let default_treasury_wallet = Address::from(default_treasury_wallet_hash);
    let default_treasury_wallet_uref: URef =
        storage::new_uref(default_treasury_wallet).into_read_add_write();
    let default_treasury_wallet_key: Key = Key::from(default_treasury_wallet_uref);

    named_keys.insert(OWNER_KEY_NAME.to_string(), owner_key);
    named_keys.insert(
        DEFAULT_TREASURY_WALLET_KEY_NAME.to_string(),
        default_treasury_wallet_key,
    );

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

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
