//! Contains definition of the entry points.
use alloc::vec;

use crate::constants::{
    GET_OWNER_ENTRY_NAME, OWNER_RUNTIME_ARG_NAME, SET_DEFAULT_TREASURY_WALLET,
    TRANSFER_OWNERSHIP_ENRTY_NAME,
};
use casper_erc20::Address;
use casper_types::{
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

pub fn transfer_ownership() -> EntryPoint {
    EntryPoint::new(
        TRANSFER_OWNERSHIP_ENRTY_NAME,
        vec![Parameter::new(OWNER_RUNTIME_ARG_NAME, Address::cl_type())],
        CLType::Bool,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_owner() -> EntryPoint {
    EntryPoint::new(
        GET_OWNER_ENTRY_NAME,
        vec![],
        Address::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_default_treasury_wallet() -> EntryPoint {
    EntryPoint::new(
        SET_DEFAULT_TREASURY_WALLET,
        vec![],
        Address::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of ERC20 token entry points.
pub fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(transfer_ownership());
    entry_points.add_entry_point(get_owner());
    entry_points
}
