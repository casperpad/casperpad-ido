//! Implementation of purse.

use casper_contract::contract_api::runtime;
use casper_types::{Key, URef};

use crate::{constants::PURSE_KEY_NAME, detail};

/// Sets main purse which handle CSPR.
pub(crate) fn set_main_purse(purse: URef) {
    runtime::put_key(PURSE_KEY_NAME, Key::from(purse))
}
/// Reads main purse
pub(crate) fn get_main_purse() -> URef {
    detail::get_uref(PURSE_KEY_NAME)
}
