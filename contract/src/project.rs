use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};

use casper_types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, Key, URef, U256,
};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME, PROJECT_ID_RUNTIME_ARG_NAME,
        PROJECT_NAME_RUNTIME_ARG_NAME, PROJECT_OPEN_TIME_RUNTIME_ARG_NAME,
        PROJECT_PRIVATE_RUNTIME_ARG_NAME, PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME,
        PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME,
        PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME,
        PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME, TREASURY_WALLET_RUNTIME_ARG_NAME,
    },
    projects,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    Upcoming,
    Going,
    Completed,
    Paused,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub private: bool,
    pub sale_start_time: i64,
    pub sale_end_time: i64,
    pub open_time: i64,
    pub token_price: u32,
    pub token_symbol: String,
    pub total_supply: u32,
    pub treasury_wallet: AccountHash,
    pub status: Status,
    pub claim_status: Key,
    pub users_length: U256,
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
        token_price: u32,
        token_symbol: String,
        total_supply: u32,
        status: Status,
        claim_status: Key,
        users_length: U256,
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
            token_price,
            token_symbol,
            total_supply,
            users_length,
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

/// Creates a dictionary item key for an (owner, spender) pair.
fn make_dictionary_item_key(field: String) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut field.to_bytes().unwrap_or_revert());

    let key_bytes = runtime::blake2b(&preimage);
    hex::encode(&key_bytes)
}

/// Writes an invest for owner and spender for a specific amount.
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
        PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME,
        project.claim_status,
    );
    write_project_field(
        project.id.clone(),
        PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME,
        project.users_length,
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
    let project_token_price: u32 =
        read_project_field(_id, PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME);
    let project_token_total_supply: u32 =
        read_project_field(_id, PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME);
    let treasury_wallet: AccountHash = read_project_field(_id, TREASURY_WALLET_RUNTIME_ARG_NAME);

    let status = Status::Upcoming;

    let users_length = U256::from(0i32);
    let claim_status_key: Key = read_project_field(_id, PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME);
    let project = Project::new(
        &project_id,
        &project_name,
        project_private,
        project_sale_start_time,
        project_sale_end_time,
        project_open_time,
        treasury_wallet,
        project_token_price,
        project_token_symbol,
        project_token_total_supply,
        status,
        claim_status_key,
        users_length,
    );
    project.serialize()
}
