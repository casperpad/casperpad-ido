#![no_main]

use casper_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};

use casper_types::RuntimeArgs;
use casper_types::{runtime_args, ContractHash, HashAddr, Key, URef, U512};

const ADD_INVEST_ENTRY_NAME: &str = "add_invest";
const GET_PURSE_ENTRY_NAME: &str = "get_purse";
const AMOUNT_RUNTIME_ARG_NAME: &str = "amount";
const CSPR_AMOUNT_RUNTIME_ARG_NAME: &str = "cspr_amount";
const IDO_CONTRACT_HASH_KEY_RUNTIME_ARG_NAME: &str = "ido_contract_hash";
const PROJECT_ID_RUNTIME_ARG_NAME: &str = "id";
const PROOF_RUNTIME_ARG_NAME: &str = "proof";
#[no_mangle]
fn call() {
    let amount: U512 = runtime::get_named_arg(AMOUNT_RUNTIME_ARG_NAME);
    let project_id: String = runtime::get_named_arg(PROJECT_ID_RUNTIME_ARG_NAME);
    let proof: Vec<(String, u8)> = runtime::get_named_arg(PROOF_RUNTIME_ARG_NAME);
    let ido_contract_hash_key: Key = runtime::get_named_arg(IDO_CONTRACT_HASH_KEY_RUNTIME_ARG_NAME);

    let contract_hash_addr: HashAddr = ido_contract_hash_key.into_hash().unwrap_or_revert();
    let ido_contract_hash: ContractHash = ContractHash::new(contract_hash_addr);

    let sender_purse: URef = account::get_main_purse();

    let deposit_purse: URef =
        runtime::call_contract(ido_contract_hash, GET_PURSE_ENTRY_NAME, runtime_args! {});

    system::transfer_from_purse_to_purse(sender_purse, deposit_purse, amount, None)
        .unwrap_or_revert();

    runtime::call_contract::<()>(
        ido_contract_hash,
        ADD_INVEST_ENTRY_NAME,
        runtime_args! {
            PROJECT_ID_RUNTIME_ARG_NAME => project_id,
            CSPR_AMOUNT_RUNTIME_ARG_NAME => amount,
            PROOF_RUNTIME_ARG_NAME => proof
        },
    );
}
