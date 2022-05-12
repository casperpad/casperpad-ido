use core::{convert::TryInto, fmt::Debug};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use casper_types::{
    account::AccountHash,
    bytesrepr::{self, FromBytes, ToBytes},
    BlockTime, CLType, CLTyped, ContractHash, URef, U256,
};

use crate::{
    constants::{
        CSPR_PRICE_RUNTIME_ARG_NAME, PROJECT_ID_RUNTIME_ARG_NAME, PROJECT_NAME_RUNTIME_ARG_NAME,
        PROJECT_OPEN_TIME_RUNTIME_ARG_NAME, PROJECT_PRIVATE_RUNTIME_ARG_NAME,
        PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME, PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME,
        PROJECT_SCHEDULES_RUNTIME_ARG_NAME, PROJECT_STATUS_RUNTIME_ARG_NAME,
        PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME, PROJECT_TOKEN_CAPACITY_RUNTIME_ARG_NAME,
        PROJECT_TOKEN_DECIMALS_RUNTIME_ARG_NAME, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
        PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME,
        PROJECT_TOTAL_INVESTS_AMOUNT_RUNTIME_ARG_NAME,
        PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME, PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME,
        TREASURY_WALLET_RUNTIME_ARG_NAME,
    },
    detail,
    error::Error,
    projects,
};
// #[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[derive(Debug, Clone, Copy)]
pub enum Status {
    Pending = 1,
    Approved = 2,
    Upcoming = 3,
    Going = 4,
    Completed = 5,
    Paused = 6,
    Cancelled = 7,
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

#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub private: bool,
    pub sale_start_time: i64,
    pub sale_end_time: i64,
    pub open_time: i64,
    pub token_address: ContractHash,
    pub token_price: U256,
    pub token_decimals: u8,
    pub token_symbol: String,
    pub total_supply: U256,
    pub token_capacity: U256,
    pub unlocked_token_amount: U256,
    pub treasury_wallet: AccountHash,
    pub status: Status,
    pub users_length: U256,
    pub schedules: Vec<(i64, U256)>,
    pub cspr_price: U256,
    pub total_invests_amount: U256,
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
        token_decimals: u8,
        token_symbol: String,
        total_supply: U256,
        token_capacity: U256,
        unlocked_token_amount: U256,
        status: Status,
        users_length: U256,
        schedules: Vec<(i64, U256)>,
        cspr_price: U256,
        total_invests_amount: U256,
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
            token_address,
            token_price,
            token_decimals,
            token_symbol,
            total_supply,
            users_length,
            schedules,
            token_capacity,
            unlocked_token_amount,
            cspr_price,
            total_invests_amount,
        }
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

    base64::encode(&preimage)
}

/// Writes project field.
pub(crate) fn write_project_field<T: CLTyped + ToBytes>(project_id: String, field: &str, value: T) {
    let uref = project_dictionary_uref(project_id);
    let dictionary_item_key = make_dictionary_item_key(field.to_string());
    storage::dictionary_put(uref, &dictionary_item_key, value);
}

/// Reads project field.
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

/// Writes project to the projects dictionary
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
        PROJECT_TOKEN_DECIMALS_RUNTIME_ARG_NAME,
        project.token_decimals,
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
        PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME,
        project.users_length,
    );

    write_project_field(
        project.id.clone(),
        PROJECT_SCHEDULES_RUNTIME_ARG_NAME,
        project.schedules,
    );

    write_project_field(
        project.id.clone(),
        PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME,
        project.unlocked_token_amount,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_TOKEN_CAPACITY_RUNTIME_ARG_NAME,
        project.token_capacity,
    );
    write_project_field(
        project.id.clone(),
        CSPR_PRICE_RUNTIME_ARG_NAME,
        project.cspr_price,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_TOTAL_INVESTS_AMOUNT_RUNTIME_ARG_NAME,
        project.total_invests_amount,
    );
}

/// Before Sale start admin must set cspr price for estimate vest amount.
pub(crate) fn only_valid_cspr_price(_id: &str) {
    let projects_uref = projects::get_projects_uref();
    projects::only_exist_project(projects_uref, _id.to_string());
    let status: U256 = read_project_field(_id, CSPR_PRICE_RUNTIME_ARG_NAME);
    if status.eq(&U256::zero()) {
        runtime::revert(Error::InvalidCSPRPrice);
    }
}

pub(crate) fn only_approved_project(_id: &str) {
    let projects_uref = projects::get_projects_uref();
    projects::only_exist_project(projects_uref, _id.to_string());
    let status: Status = read_project_field(_id, PROJECT_STATUS_RUNTIME_ARG_NAME);
    match status {
        Status::Approved => (),
        _ => runtime::revert(Error::PermissionDenied),
    }
}

/// Users can vest during the vest time.
pub(crate) fn only_sale_time(_id: &str) -> BlockTime {
    let projects_uref = projects::get_projects_uref();
    projects::only_exist_project(projects_uref, _id.to_string());
    only_valid_cspr_price(_id);
    let project_sale_start_time: i64 =
        read_project_field(_id, PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME);
    let project_sale_end_time: i64 =
        read_project_field(_id, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME);
    // Current time is after 1970
    let sale_start_block_time = BlockTime::new(project_sale_start_time.try_into().unwrap());
    let sale_end_block_time = BlockTime::new(project_sale_end_time.try_into().unwrap());

    let current_block_time: BlockTime = runtime::get_blocktime();

    detail::store_result(u64::from(current_block_time));

    if current_block_time.lt(&sale_start_block_time) {
        runtime::revert(Error::SaleNotStarted);
    }
    if current_block_time.gt(&sale_end_block_time) {
        runtime::revert(Error::SaleEnded);
    }
    current_block_time
}

/// Users can claim after specific time.
pub(crate) fn only_after_time(time: i64) {
    let current_block_time: BlockTime = runtime::get_blocktime();
    let block_time = BlockTime::new(time.try_into().unwrap());
    if current_block_time.gt(&block_time) {
        runtime::revert(Error::NotValidTime);
    }
}
