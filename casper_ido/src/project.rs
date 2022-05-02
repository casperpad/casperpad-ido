use core::{convert::TryInto, fmt::Debug};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use casper_erc20::{
    constants::{ADDRESS_RUNTIME_ARG_NAME, BALANCE_OF_ENTRY_POINT_NAME},
    Address,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::{self, FromBytes, ToBytes},
    runtime_args, BlockTime, CLType, CLTyped, ContractHash, Key, RuntimeArgs, URef, U256,
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        PROJECT_CAPACITY_USD_RUNTIME_ARG_NAME, PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME,
        PROJECT_ID_RUNTIME_ARG_NAME, PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME,
        PROJECT_NAME_RUNTIME_ARG_NAME, PROJECT_OPEN_TIME_RUNTIME_ARG_NAME,
        PROJECT_PRIVATE_RUNTIME_ARG_NAME, PROJECT_REWARD_MULTIPLY_RUNTIME_ARG_NAME,
        PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME, PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME,
        PROJECT_SCHEDULES_RUNTIME_ARG_NAME, PROJECT_STATUS_RUNTIME_ARG_NAME,
        PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
        PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME,
        PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME, PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME,
        PROJECT_USERS_RUNTIME_ARG_NAME, TREASURY_WALLET_RUNTIME_ARG_NAME,
    },
    detail,
    error::Error,
    projects,
};
// #[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Status {
    Upcoming = 1,
    Going = 2,
    Completed = 3,
    Paused = 4,
    Cancelled = 5,
}

impl Status {
    pub fn from_u32(value: u32) -> Status {
        match value {
            1 => Status::Upcoming,
            2 => Status::Going,
            3 => Status::Completed,
            4 => Status::Paused,
            5 => Status::Cancelled,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

impl CLTyped for Status {
    fn cl_type() -> CLType {
        CLType::U32
    }
}

// Serialize for Status
impl ToBytes for Status {
    #[inline(always)]
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        Ok((*self as u32).into_bytes().unwrap().to_vec())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        32
    }

    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
}

// Deserialize for Status
impl FromBytes for Status {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (result, remainder) = u32::from_bytes(bytes).unwrap();
        let project: Status = Status::from_u32(result);
        Ok((project, remainder))
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Schedule {
    pub unlock_time: i64,
    pub unlock_percent: i64,
}

impl CLTyped for Schedule {
    fn cl_type() -> CLType {
        CLType::ByteArray(16)
    }
}

impl ToBytes for Schedule {
    #[inline(always)]
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut preimage = Vec::new();
        preimage.append(&mut self.unlock_time.to_bytes().unwrap());
        preimage.append(&mut self.unlock_percent.to_bytes().unwrap());
        Ok(preimage)
        // Ok((*self as u32).into_bytes().unwrap().to_vec())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        128
    }

    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
}

impl FromBytes for Schedule {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (unlock_time, remainder1) = i64::from_bytes(bytes).unwrap();
        let (unlock_percent, remainder2) = i64::from_bytes(remainder1).unwrap();
        let schedule = Schedule {
            unlock_time,
            unlock_percent,
        };
        Ok((schedule, remainder2))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub private: bool,
    pub sale_start_time: i64,
    pub sale_end_time: i64,
    pub open_time: i64,
    pub token_address: ContractHash,
    pub token_price: U256,
    pub token_symbol: String,
    pub total_supply: U256,
    pub capacity_usd: U256,
    pub locked_token_amount: U256,
    pub unlocked_token_amount: U256,
    pub treasury_wallet: AccountHash,
    pub status: Status,
    pub claim_status: Key,
    pub users: Key,
    pub users_length: U256,
    pub reward_multiply: U256, // decimal is 3
    pub schedules: Vec<Schedule>,
}

impl Project {
    pub fn new(
        id: &str,
        name: &str,
        private: bool,
        sale_start_time: i64,
        sale_end_time: i64,
        open_time: i64,
        treasury_wallet: AccountHash,
        token_address: ContractHash,
        token_price: U256,
        token_symbol: String,
        total_supply: U256,
        capacity_usd: U256,
        locked_token_amount: U256,
        unlocked_token_amount: U256,
        status: Status,
        claim_status: Key,
        reward_multiply: U256,
        users: Key,
        users_length: U256,
        schedules: Vec<Schedule>,
    ) -> Self {
        Self {
            id: String::from(id),
            name: String::from(name),
            private,
            sale_start_time,
            sale_end_time,
            open_time,
            treasury_wallet,
            status,
            claim_status,
            token_address,
            token_price,
            token_symbol,
            total_supply,
            capacity_usd,
            users_length,
            schedules,
            locked_token_amount,
            unlocked_token_amount,
            users,
            reward_multiply,
        }
    }
    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[inline]
pub(crate) fn project_dictionary_uref(project_id: String) -> URef {
    let projects_uref = projects::get_projects_uref();
    projects::read_project_from(projects_uref, project_id)
}

/// Creates a dictionary item key
pub(crate) fn make_dictionary_item_key(field: String) -> String {
    let mut preimage = Vec::new();

    preimage.append(&mut field.to_bytes().unwrap_or_revert());

    let key_bytes = runtime::blake2b(&preimage);
    hex::encode(&key_bytes)
}

pub(crate) fn make_users_dictionary_item_key(account: Address) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut account.to_bytes().unwrap_or_revert());

    let key_bytes = runtime::blake2b(&preimage);
    hex::encode(&key_bytes)
}

///
pub(crate) fn write_project_field<T: CLTyped + ToBytes>(project_id: String, field: &str, value: T) {
    let dictionary_item_key = make_dictionary_item_key(field.to_string());
    let uref = project_dictionary_uref(project_id);
    storage::dictionary_put(uref, &dictionary_item_key, value);
}

/// Reads an invest for a owner and spender
pub(crate) fn read_project_field<T: CLTyped + ToBytes + FromBytes>(
    project_id: &str,
    field: &str,
) -> T {
    let dictionary_item_key = make_dictionary_item_key(field.to_string());
    let uref = project_dictionary_uref(project_id.to_string());
    storage::dictionary_get(uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap_or_revert()
}

pub(crate) fn write_project(project: Project) {
    let projects_uref = projects::get_projects_uref();
    projects::write_project_to(projects_uref, project.id.clone());
    write_project_field(
        project.id.clone(),
        PROJECT_ID_RUNTIME_ARG_NAME,
        project.id.clone(),
    );
    write_project_field(
        project.id.clone(),
        PROJECT_NAME_RUNTIME_ARG_NAME,
        project.name,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_PRIVATE_RUNTIME_ARG_NAME,
        project.private,
    );
    write_project_field(
        project.id.clone(),
        TREASURY_WALLET_RUNTIME_ARG_NAME,
        project.treasury_wallet,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME,
        project.sale_start_time,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME,
        project.sale_end_time,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_OPEN_TIME_RUNTIME_ARG_NAME,
        project.open_time,
    );

    write_project_field(
        project.id.clone(),
        PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME,
        project.token_address,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
        project.token_price,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME,
        project.token_symbol,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME,
        project.total_supply,
    );

    write_project_field(
        project.id.clone(),
        PROJECT_STATUS_RUNTIME_ARG_NAME,
        project.status,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME,
        project.claim_status,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME,
        project.users_length,
    );

    write_project_field(
        project.id.clone(),
        PROJECT_CAPACITY_USD_RUNTIME_ARG_NAME,
        project.capacity_usd,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_SCHEDULES_RUNTIME_ARG_NAME,
        project.schedules,
    );

    write_project_field(
        project.id.clone(),
        PROJECT_REWARD_MULTIPLY_RUNTIME_ARG_NAME,
        project.reward_multiply,
    );

    write_project_field(
        project.id.clone(),
        PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME,
        project.unlocked_token_amount,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_USERS_RUNTIME_ARG_NAME,
        project.users,
    );
}

pub(crate) fn read_project(_id: &str) -> String {
    let project_id: String = read_project_field(_id, PROJECT_ID_RUNTIME_ARG_NAME);
    let project_name: String = read_project_field(_id, PROJECT_NAME_RUNTIME_ARG_NAME);
    let project_sale_start_time: i64 =
        read_project_field(_id, PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME);
    let project_sale_end_time: i64 =
        read_project_field(_id, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME);
    let project_open_time: i64 = read_project_field(_id, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME);
    let project_private: bool = read_project_field(_id, PROJECT_PRIVATE_RUNTIME_ARG_NAME);
    let project_token_symbol: String =
        read_project_field(_id, PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME);
    let project_token_price: U256 =
        read_project_field(_id, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME);
    let project_token_total_supply: U256 =
        read_project_field(_id, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME);
    let treasury_wallet: AccountHash = read_project_field(_id, TREASURY_WALLET_RUNTIME_ARG_NAME);
    let project_token_address: ContractHash =
        read_project_field(_id, PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME);
    let status: Status = read_project_field(_id, PROJECT_STATUS_RUNTIME_ARG_NAME);
    let users_length: U256 = read_project_field(_id, PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME);
    let claim_status_key: Key = read_project_field(_id, PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME);
    let capacity_usd: U256 = read_project_field(_id, PROJECT_CAPACITY_USD_RUNTIME_ARG_NAME);
    let schedules: Vec<Schedule> = read_project_field(_id, PROJECT_SCHEDULES_RUNTIME_ARG_NAME);
    let reward_multiply: U256 = read_project_field(_id, PROJECT_REWARD_MULTIPLY_RUNTIME_ARG_NAME);
    let locked_token_amount: U256 = runtime::call_contract(
        project_token_address,
        BALANCE_OF_ENTRY_POINT_NAME,
        runtime_args! {
            ADDRESS_RUNTIME_ARG_NAME => detail::get_caller_address().unwrap_or_revert()
        },
    );
    let unlocked_token_amount: U256 =
        read_project_field(_id, PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME);
    let users: Key = read_project_field(_id, PROJECT_USERS_RUNTIME_ARG_NAME);

    let project = Project::new(
        &project_id,
        &project_name,
        project_private,
        project_sale_start_time,
        project_sale_end_time,
        project_open_time,
        treasury_wallet,
        project_token_address,
        project_token_price,
        project_token_symbol,
        project_token_total_supply,
        capacity_usd,
        locked_token_amount,
        unlocked_token_amount,
        status,
        claim_status_key,
        reward_multiply,
        users,
        users_length,
        schedules,
    );
    project.serialize()
}

pub(crate) fn only_active_project(_id: &str) {
    let projects_uref = projects::get_projects_uref();
    projects::only_exist_project(projects_uref, _id.to_string());
    let status: Status = read_project_field(_id, PROJECT_STATUS_RUNTIME_ARG_NAME);
    match status {
        Status::Going => (),
        _ => runtime::revert(Error::PermissionDenied),
    }
}

pub(crate) fn only_sale_time(_id: &str) -> (BlockTime, BlockTime) {
    let projects_uref = projects::get_projects_uref();
    projects::only_exist_project(projects_uref, _id.to_string());
    let project_sale_start_time: i64 =
        read_project_field(_id, PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME);
    let project_sale_end_time: i64 =
        read_project_field(_id, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME);
    // Current time is after 1970
    let sale_end_block_time = BlockTime::new(project_sale_end_time.try_into().unwrap());

    let current_block_time: BlockTime = runtime::get_blocktime();
    // if current_block_time.gt(&sale_end_block_time) {
    //     runtime::revert(Error::PermissionDenied);
    // }
    (sale_end_block_time, current_block_time)
}
