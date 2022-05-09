//! Implementation of projects.
use alloc::string::{String, ToString};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{bytesrepr::ToBytes, URef};

use crate::{constants::PROJECTS_KEY_NAME, detail, error::Error};

/// Creates a dictionary item key for a dictionary item.
#[inline]
fn make_dictionary_item_key(project_id: String) -> String {
    let preimage = project_id.to_bytes().unwrap_or_revert();
    // NOTE: As for now dictionary item keys are limited to 64 characters only. Instead of using
    // hashing (which will effectively hash a hash) we'll use base64. Preimage is about 33 bytes for
    // both Address variants, and approximated base64-encoded length will be 4 * (33 / 3) ~ 44
    // characters.
    // Even if the preimage increased in size we still have extra space but even in case of much
    // larger preimage we can switch to base85 which has ratio of 4:5.

    base64::encode(&preimage)
}

pub(crate) fn get_projects_uref() -> URef {
    detail::get_uref(PROJECTS_KEY_NAME)
}

/// Writes token balance of a specified account into a dictionary.
pub(crate) fn write_project_to(projects_uref: URef, project_id: String) {
    let dictionary_item_key = make_dictionary_item_key(project_id.clone().to_string());
    let uref = storage::new_dictionary(project_id.as_str()).unwrap_or_revert();
    storage::dictionary_put(projects_uref, &dictionary_item_key, uref);
}

pub(crate) fn read_project_from(projects_uref: URef, project_id: String) -> URef {
    let dictionary_item_key = make_dictionary_item_key(project_id.to_string());

    storage::dictionary_get(projects_uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap()
}

pub(crate) fn only_exist_project(projects_uref: URef, project_id: String) {
    let dictionary_item_key = make_dictionary_item_key(project_id.to_string());

    let uref =
        storage::dictionary_get::<URef>(projects_uref, &dictionary_item_key).unwrap_or_revert();
    match uref {
        Some(_) => (),
        None => runtime::revert(Error::NotExistProject),
    }
}

pub(crate) fn only_not_exist_project(project_id: String) {
    let projects_uref = get_projects_uref();
    let dictionary_item_key = make_dictionary_item_key(project_id.to_string());

    let uref =
        storage::dictionary_get::<URef>(projects_uref, &dictionary_item_key).unwrap_or_revert();
    match uref {
        Some(_) => runtime::revert(Error::AlreadyExistProject),
        None => (),
    }
}
