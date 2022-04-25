//! Contains definition of the entry points.
use alloc::vec;

use crate::constants::{
    GET_OWNER_ENTRY_NAME, OWNER_RUNTIME_ARG_NAME, PROJECT_RUNTIME_ARG_NAME,
    SET_DEFAULT_TREASURY_WALLET, SET_PROJECT_TREASURY_WALLET, TRANSFER_OWNERSHIP_ENRTY_NAME,
    WALLET_RUNTIME_ARG_NAME,
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

pub fn set_project_treasury_wallet() -> EntryPoint {
    EntryPoint::new(
        SET_PROJECT_TREASURY_WALLET,
        vec![
            Parameter::new(PROJECT_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(WALLET_RUNTIME_ARG_NAME, Address::cl_type()),
        ],
        Address::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of ido contract entry points.
pub fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(transfer_ownership());
    entry_points.add_entry_point(get_owner());
    entry_points.add_entry_point(set_default_treasury_wallet());
    entry_points.add_entry_point(set_project_treasury_wallet());

    entry_points
}
