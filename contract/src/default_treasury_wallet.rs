//! Implementation of default treasury wallet.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_erc20::Address;
use casper_types::URef;

use crate::{constants::DEFAULT_TREASURY_WALLET_KEY_NAME, detail};

#[inline]
pub(crate) fn default_treasury_wallet_uref() -> URef {
    detail::get_uref(DEFAULT_TREASURY_WALLET_KEY_NAME)
}

/// Reads a owner from a specified [`URef`].
pub(crate) fn read_default_treasury_wallet_from(uref: URef) -> Address {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

/// Writes a owner to a specific [`URef`].
pub(crate) fn write_default_tresury_wallet_to(uref: URef, value: Address) {
    storage::write(uref, value);
}
