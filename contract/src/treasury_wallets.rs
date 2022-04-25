//! Implementation of project->treasury_wallet mapping.
use alloc::string::String;

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{bytesrepr::ToBytes, URef};

use crate::{constants::TREASURY_WALLET_KEY_NAME, detail, Address};

/// Creates a dictionary item key for a dictionary item.
#[inline]
fn make_dictionary_item_key(project: Address) -> String {
    let preimage = project.to_bytes().unwrap_or_revert();
    // NOTE: As for now dictionary item keys are limited to 64 characters only. Instead of using
    // hashing (which will effectively hash a hash) we'll use base64. Preimage is about 33 bytes for
    // both Address variants, and approximated base64-encoded length will be 4 * (33 / 3) ~ 44
    // characters.
    // Even if the preimage increased in size we still have extra space but even in case of much
    // larger preimage we can switch to base85 which has ratio of 4:5.
    base64::encode(&preimage)
}

pub(crate) fn get_wallets_uref() -> URef {
    detail::get_uref(TREASURY_WALLET_KEY_NAME)
}

pub(crate) fn write_wallets_to(
    wallets_uref: URef,
    project_address: Address,
    wallet_address: Address,
) {
    let dictionary_item_key = make_dictionary_item_key(project_address);
    storage::dictionary_put(wallets_uref, &dictionary_item_key, wallet_address);
}

/// Reads project`s treasury_wallet address
///
/// If a given account does not have walletss in the system, then default wallet is returned. TODO
pub(crate) fn read_wallets_from(wallets_uref: URef, project_address: Address) -> Address {
    let dictionary_item_key = make_dictionary_item_key(project_address);

    storage::dictionary_get(wallets_uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap()
}
