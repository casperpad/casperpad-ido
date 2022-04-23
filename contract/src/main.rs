#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use crate::alloc::string::{String};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::Address;
use casper_types::{contracts::NamedKeys, CLValue, Key, URef};

pub mod constants;
mod detail;
mod entry_points;
mod error;
mod owner;
use crate::constants::{CONTRACT_NAME_KEY_NAME, OWNER_KEY_NAME, OWNER_RUNTIME_ARG_NAME};
use error::Error;

#[no_mangle]
pub extern "C" fn transfer_ownership() {
    let new_owner: Address = runtime::get_named_arg(OWNER_RUNTIME_ARG_NAME);
    let caller: Address = detail::get_immediate_caller_address().unwrap_or_revert();
    let owner_uref: URef = owner::owner_uref();
    let current_owner: Address = owner::read_owner_from(owner_uref);
    if caller != current_owner {
        runtime::revert(Error::PermissionDenied)
    }
    owner::write_owner_to(owner_uref, new_owner);
    runtime::ret(CLValue::from_t(true).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn get_owner() {
    let owner: Address = detail::read_from(OWNER_KEY_NAME);
    runtime::ret(CLValue::from_t(owner).unwrap_or_revert());
}

#[no_mangle]
pub extern "C" fn get_contract_name() {
    let contract_name: String = detail::read_from(CONTRACT_NAME_KEY_NAME);
    runtime::ret(CLValue::from_t(contract_name).unwrap_or_revert());
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

    named_keys.insert(OWNER_KEY_NAME.to_string(), owner_key);

    let entry_points = entry_points::default();

    let (contract_hash, _version) =
        storage::new_locked_contract(entry_points, Some(named_keys), None, None);
    let mut contract_hash_key_name: String = String::from(CONTRACT_NAME_KEY_NAME);
    contract_hash_key_name.push_str("_contract_hash");
    runtime::put_key(contract_hash_key_name.as_str(), Key::from(contract_hash));
}

#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
