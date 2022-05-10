//! Contains definition of the entry points.
use alloc::{boxed::Box, vec};

use crate::constants::{
    ADD_INVEST_ENTRY_NAME, CLAIM_ENTRY_NAME, CREATE_PROJECT_ENTRY_NAME,
    CSPR_AMOUNT_RUNTIME_ARG_NAME, CSPR_PRICE_RUNTIME_ARG_NAME, GET_PURSE_ENTRY_NAME,
    MERKLE_ROOT_RUNTIME_ARG_NAME, MULTIPLE_TIERS_RUNTIME_ARG_NAME, OWNER_RUNTIME_ARG_NAME,
    PROJECT_ID_RUNTIME_ARG_NAME, PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME,
    PROJECT_NAME_RUNTIME_ARG_NAME, PROJECT_OPEN_TIME_RUNTIME_ARG_NAME,
    PROJECT_PRIVATE_RUNTIME_ARG_NAME, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME,
    PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME, PROJECT_SCHEDULES_RUNTIME_ARG_NAME,
    PROJECT_STATUS_RUNTIME_ARG_NAME, PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME,
    PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME, PROOF_RUNTIME_ARG_NAME, SCHEDULE_ID_RUNTIME_ARG_NAME,
    SET_CSPR_PRICE_ENTRY_NAME, SET_MERKLE_ROOT_ENTRY_NAME, SET_MULTIPLE_TIERS_ENTRY_NAME,
    SET_PROJECT_STATUS_ENTRY_NAME, SET_PURSE_ENTRY_NAME, SET_TIER_ENTRY_NAME,
    TIER_RUNTIME_ARG_NAME, TRANSFER_OWNERSHIP_ENRTY_NAME, TREASURY_WALLET_RUNTIME_ARG_NAME,
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

pub fn add_project() -> EntryPoint {
    EntryPoint::new(
        CREATE_PROJECT_ENTRY_NAME,
        vec![
            Parameter::new(PROJECT_ID_RUNTIME_ARG_NAME, CLType::String),
            Parameter::new(PROJECT_NAME_RUNTIME_ARG_NAME, CLType::String),
            Parameter::new(PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME, CLType::I64),
            Parameter::new(PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME, CLType::I64),
            Parameter::new(PROJECT_OPEN_TIME_RUNTIME_ARG_NAME, CLType::I64),
            Parameter::new(PROJECT_PRIVATE_RUNTIME_ARG_NAME, CLType::Bool),
            Parameter::new(PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME, CLType::Key),
            Parameter::new(PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME, CLType::U256),
            Parameter::new(PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME, CLType::U256),
            Parameter::new(TREASURY_WALLET_RUNTIME_ARG_NAME, CLType::String),
            Parameter::new(
                PROJECT_SCHEDULES_RUNTIME_ARG_NAME,
                CLType::List(Box::new(CLType::Tuple2([
                    Box::new(CLType::I64),
                    Box::new(CLType::U256),
                ]))),
            ),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_project_status() -> EntryPoint {
    EntryPoint::new(
        SET_PROJECT_STATUS_ENTRY_NAME,
        vec![
            Parameter::new(PROJECT_ID_RUNTIME_ARG_NAME, CLType::String),
            Parameter::new(PROJECT_STATUS_RUNTIME_ARG_NAME, CLType::U32),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn add_invest() -> EntryPoint {
    EntryPoint::new(
        ADD_INVEST_ENTRY_NAME,
        vec![
            Parameter::new(PROJECT_ID_RUNTIME_ARG_NAME, CLType::String),
            Parameter::new(CSPR_AMOUNT_RUNTIME_ARG_NAME, CLType::U256),
            Parameter::new(
                PROOF_RUNTIME_ARG_NAME,
                CLType::List(Box::new(CLType::Tuple2([
                    Box::new(CLType::String),
                    Box::new(CLType::U8),
                ]))),
            ),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_merkle_root() -> EntryPoint {
    EntryPoint::new(
        SET_MERKLE_ROOT_ENTRY_NAME,
        vec![Parameter::new(
            MERKLE_ROOT_RUNTIME_ARG_NAME,
            CLType::ByteArray(32),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn claim() -> EntryPoint {
    EntryPoint::new(
        CLAIM_ENTRY_NAME,
        vec![
            Parameter::new(PROJECT_ID_RUNTIME_ARG_NAME, CLType::String),
            Parameter::new(SCHEDULE_ID_RUNTIME_ARG_NAME, CLType::U8),
        ],
        CLType::U256,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_purse() -> EntryPoint {
    EntryPoint::new(
        GET_PURSE_ENTRY_NAME,
        vec![Parameter::new(CSPR_AMOUNT_RUNTIME_ARG_NAME, CLType::U256)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_purse() -> EntryPoint {
    EntryPoint::new(
        SET_PURSE_ENTRY_NAME,
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_cspr_price() -> EntryPoint {
    EntryPoint::new(
        SET_CSPR_PRICE_ENTRY_NAME,
        vec![Parameter::new(CSPR_PRICE_RUNTIME_ARG_NAME, CLType::U256)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_multiple_tiers() -> EntryPoint {
    EntryPoint::new(
        SET_MULTIPLE_TIERS_ENTRY_NAME,
        vec![Parameter::new(
            MULTIPLE_TIERS_RUNTIME_ARG_NAME,
            CLType::List(Box::new(CLType::Tuple2([
                Box::new(CLType::String),
                Box::new(CLType::U256),
            ]))),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_tier() -> EntryPoint {
    EntryPoint::new(
        SET_TIER_ENTRY_NAME,
        vec![Parameter::new(
            TIER_RUNTIME_ARG_NAME,
            CLType::Tuple2([Box::new(CLType::String), Box::new(CLType::U256)]),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
/// Returns the default set of ido contract entry points.
pub fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(transfer_ownership());
    entry_points.add_entry_point(add_project());
    entry_points.add_entry_point(set_project_status());
    entry_points.add_entry_point(add_invest());
    entry_points.add_entry_point(claim());
    entry_points.add_entry_point(set_merkle_root());
    entry_points.add_entry_point(get_purse());
    entry_points.add_entry_point(set_purse());
    entry_points.add_entry_point(set_cspr_price());

    entry_points
}
