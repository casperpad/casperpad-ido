//! Implementation of owner.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_erc20::Address;
use casper_types::URef;

use crate::{constants::OWNER_KEY_NAME, detail};

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

// pub(crate) fn
