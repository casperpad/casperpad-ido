//! Implementation of invests.
use alloc::{string::String, vec::Vec};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{bytesrepr::ToBytes, URef, U256};

use crate::{constants::INVESTS_KEY_NAME, detail, Address};

#[inline]
pub(crate) fn invests_uref() -> URef {
    detail::get_uref(INVESTS_KEY_NAME)
}

/// Creates a dictionary item key for an (owner, spender) pair.
fn make_dictionary_item_key(project_id: String, account: Address) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut project_id.to_bytes().unwrap_or_revert());
    preimage.append(&mut account.to_bytes().unwrap_or_revert());

    let key_bytes = runtime::blake2b(&preimage);
    hex::encode(&key_bytes)
}

/// Writes an invest for owner and spender for a specific amount.
pub(crate) fn write_invest_to(project_id: String, account: Address, amount: U256) {
    let dictionary_item_key = make_dictionary_item_key(project_id, account);
    let uref = invests_uref();
    storage::dictionary_put(uref, &dictionary_item_key, amount)
}

/// Reads an invest for a owner and spender
pub(crate) fn read_invest_from(project_id: String, account: Address) -> U256 {
    let dictionary_item_key = make_dictionary_item_key(project_id, account);
    let uref = invests_uref();
    storage::dictionary_get(uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap_or_default()
}
