//! Implementation of default treasury wallet.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};

use casper_types::{account::AccountHash, URef};

use crate::{constants::DEFAULT_TREASURY_WALLET_KEY_NAME, detail};

#[inline]
pub(crate) fn default_treasury_wallet_uref() -> URef {
    detail::get_uref(DEFAULT_TREASURY_WALLET_KEY_NAME)
}

/// Reads default treasury wallet from a specified [`URef`].
pub(crate) fn read_default_treasury_wallet_from(uref: URef) -> AccountHash {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

/// Writes default treasury wallet to a specific [`URef`].
pub(crate) fn write_default_tresury_wallet_to(uref: URef, value: AccountHash) {
    storage::write(uref, value);
}
