//! Contains definition of the entry points.
use alloc::{string::String, vec};

use casper_erc20::Address;
use casper_types::{
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

use crate::constants::{
    ADD_INVEST_ENTRY_NAME, CREATE_PROJECT_ENTRY_NAME, CSPR_AMOUNT_RUNTIME_ARG_NAME,
    DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME, GET_DEFAULT_TREASURY_WALLET_ENTRY_NAME,
    GET_INVEST_INFO_ENTRY_NAME, GET_OWNER_ENTRY_NAME, GET_PROJECT_INFO_ENTRY_NAME,
    OWNER_RUNTIME_ARG_NAME, PROJECT_END_TIME_RUNTIME_ARG_NAME, PROJECT_ID_RUNTIME_ARG_NAME,
    PROJECT_NAME_RUNTIME_ARG_NAME, PROJECT_PRIVATE_RUNTIME_ARG_NAME,
    PROJECT_START_TIME_RUNTIME_ARG_NAME, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    SET_DEFAULT_TREASURY_WALLET_ENTRY_NAME, TRANSFER_OWNERSHIP_ENRTY_NAME,
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
        SET_DEFAULT_TREASURY_WALLET_ENTRY_NAME,
        vec![Parameter::new(
            DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME,
            Address::cl_type(),
        )],
        Address::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_default_treasury_wallet() -> EntryPoint {
    EntryPoint::new(
        GET_DEFAULT_TREASURY_WALLET_ENTRY_NAME,
        vec![],
        Address::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn add_project() -> EntryPoint {
    EntryPoint::new(
        CREATE_PROJECT_ENTRY_NAME,
        vec![
            Parameter::new(PROJECT_ID_RUNTIME_ARG_NAME, String::cl_type()),
            Parameter::new(PROJECT_NAME_RUNTIME_ARG_NAME, String::cl_type()),
            Parameter::new(PROJECT_START_TIME_RUNTIME_ARG_NAME, CLType::I64),
            Parameter::new(PROJECT_END_TIME_RUNTIME_ARG_NAME, CLType::I64),
            Parameter::new(PROJECT_PRIVATE_RUNTIME_ARG_NAME, CLType::Bool),
            Parameter::new(PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME, String::cl_type()),
            Parameter::new(PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME, CLType::U32),
            Parameter::new(PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME, CLType::U32),
        ],
        CLType::Bool,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_project_info_by_id() -> EntryPoint {
    EntryPoint::new(
        GET_PROJECT_INFO_ENTRY_NAME,
        vec![Parameter::new(
            PROJECT_ID_RUNTIME_ARG_NAME,
            String::cl_type(),
        )],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn add_invest() -> EntryPoint {
    EntryPoint::new(
        ADD_INVEST_ENTRY_NAME,
        vec![
            Parameter::new(PROJECT_ID_RUNTIME_ARG_NAME, String::cl_type()),
            Parameter::new(CSPR_AMOUNT_RUNTIME_ARG_NAME, CLType::U256),
        ],
        CLType::Bool,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_invest_info() -> EntryPoint {
    EntryPoint::new(
        GET_INVEST_INFO_ENTRY_NAME,
        vec![Parameter::new(
            PROJECT_ID_RUNTIME_ARG_NAME,
            String::cl_type(),
        )],
        CLType::U256,
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
    entry_points.add_entry_point(get_default_treasury_wallet());
    entry_points.add_entry_point(add_project());
    entry_points.add_entry_point(get_project_info_by_id());
    entry_points.add_entry_point(add_invest());
    entry_points.add_entry_point(get_invest_info());

    entry_points
}
