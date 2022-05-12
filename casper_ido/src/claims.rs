//! Implementation of claims.
use alloc::{string::String, vec::Vec};

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{account::AccountHash, bytesrepr::ToBytes, Key, URef, U256};

use crate::{constants::CLAIMS_KEY_NAME, detail};

#[inline]
pub(crate) fn claims_uref() -> URef {
    detail::get_uref(CLAIMS_KEY_NAME)
}

/// Creates a dictionary item key for an (project_id, account, schedule_id) pair.
fn make_dictionary_item_key(account: AccountHash, project_id: String, schedule_id: u8) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut project_id.to_bytes().unwrap_or_revert());
    let account_key = Key::from(account);
    preimage.append(&mut account_key.to_bytes().unwrap_or_revert());
    preimage.append(&mut schedule_id.to_bytes().unwrap_or_revert());

    base64::encode(&preimage)
}

/// Writes claim amount for the specific project, account, schedule.
pub(crate) fn write_claim_to(
    project_id: String,
    account: AccountHash,
    schedule_id: u8,
    amount: U256,
) {
    let dictionary_item_key = make_dictionary_item_key(account, project_id, schedule_id);
    let uref = claims_uref();
    storage::dictionary_put(uref, &dictionary_item_key, amount)
}

/// Reads claim amount for the specific project, account, schedule.
pub(crate) fn read_claim_from(project_id: String, account: AccountHash, schedule_id: u8) -> U256 {
    let dictionary_item_key = make_dictionary_item_key(account, project_id, schedule_id);
    let uref = claims_uref();
    storage::dictionary_get(uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap_or_default()
}
