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
    hex::encode(&preimage)
}

pub(crate) fn get_projects_uref() -> URef {
    detail::get_uref(PROJECTS_KEY_NAME)
}

/// Writes project dictionary to projects dictionary
pub(crate) fn write_project_to(projects_uref: URef, project_id: String) {
    let dictionary_item_key = make_dictionary_item_key(project_id.clone().to_string());
    let uref = storage::new_dictionary(project_id.as_str()).unwrap_or_revert();
    storage::dictionary_put(projects_uref, &dictionary_item_key, uref);
}

/// Reads project dictionary to projects dictionary
pub(crate) fn read_project_from(projects_uref: URef, project_id: String) -> URef {
    let dictionary_item_key = make_dictionary_item_key(project_id.to_string());

    storage::dictionary_get(projects_uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap()
}

/// Project must exist
pub(crate) fn only_exist_project(projects_uref: URef, project_id: String) {
    let dictionary_item_key = make_dictionary_item_key(project_id.to_string());

    let uref =
        storage::dictionary_get::<URef>(projects_uref, &dictionary_item_key).unwrap_or_revert();
    match uref {
        Some(_) => (),
        None => runtime::revert(Error::NotExistProject),
    }
}

/// Project must not exist.
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
