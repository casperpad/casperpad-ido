#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

use alloc::{string::String, vec::Vec};
use casper_contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, ContractHash, RuntimeArgs, U512};

#[no_mangle]
pub extern "C" fn call() {
    let contract_hash: ContractHash = {
        let contract_hash_string: String = runtime::get_named_arg("ido_contract_hash");
        ContractHash::from_formatted_str(&contract_hash_string).unwrap()
    };
    let auction_id: String = runtime::get_named_arg("auction_id");
    let proof: Vec<(String, u8)> = runtime::get_named_arg("proof");
    let amount: U512 = runtime::get_named_arg("amount");

    let deposit_purse = system::create_purse();
    let account_purse = account::get_main_purse();
    system::transfer_from_purse_to_purse(account_purse, deposit_purse, amount, None)
        .unwrap_or_revert();
    let _ = runtime::call_contract::<()>(
        contract_hash,
        "create_order_cspr",
        runtime_args! {
          "auction_id" => auction_id,
          "proof" => proof,
          "deposit_purse" => deposit_purse,
        },
    );
}