//! Implementation of owner.

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::Address;
use casper_types::URef;

use crate::{constants::OWNER_KEY_NAME, detail, error::Error};

#[inline]
pub(crate) fn owner_uref() -> URef {
    detail::get_uref(OWNER_KEY_NAME)
}

/// Reads a owner from a specified [`URef`].
pub(crate) fn read_owner_from(uref: URef) -> Address {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

/// Writes a owner to a specific [`URef`].
pub(crate) fn write_owner_to(uref: URef, value: Address) {
    storage::write(uref, value);
}

pub(crate) fn only_owner() {
    let caller: Address = detail::get_immediate_caller_address().unwrap_or_revert();
    let owner_uref: URef = owner_uref();
    let current_owner: Address = read_owner_from(owner_uref);
    if caller != current_owner {
        runtime::revert(Error::PermissionDenied)
    }
}
