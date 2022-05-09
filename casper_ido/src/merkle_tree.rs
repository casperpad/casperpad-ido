//! Implementation of merkle_tree.

use alloc::string::{String, ToString};

use alloc::vec::Vec;

use casper_contract::contract_api::runtime;
use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};

use casper_types::URef;
use tiny_keccak::Hasher;

use crate::{constants::MERKLE_ROOT_KEY_NAME, detail, error::Error};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Position {
    Left = 0,
    Right = 1,
}

impl Position {
    pub fn from_u8(value: u8) -> Position {
        match value {
            0 => Position::Left,
            1 => Position::Right,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

#[inline]
pub(crate) fn merkle_tree_root_uref() -> URef {
    detail::get_uref(MERKLE_ROOT_KEY_NAME)
}

/// Reads a merkle_tree from a specified [`URef`].
pub(crate) fn read_merkle_tree_root_from(uref: URef) -> String {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

/// Writes a merkle_tree to a specific [`URef`].
pub(crate) fn write_merkle_tree_root_to(uref: URef, value: String) {
    storage::write(uref, value);
}

pub(crate) fn verify_whitelist(proof: Vec<(String, u8)>) {
    let converted_proof: Vec<(Vec<u8>, Position)> = proof
        .iter()
        .map(|proof| {
            (
                hex::decode(proof.0.clone()).unwrap(),
                Position::from_u8(proof.1),
            )
        })
        .collect();

    let caller: String = detail::get_immediate_caller_address()
        .unwrap_or_revert()
        .as_account_hash()
        .unwrap_or_revert()
        .to_string();
    // let caller = "58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7".to_string();
    let caller_bytes: &[u8] = caller.as_bytes();

    let mut keccak = tiny_keccak::Keccak::v256();
    let mut result: [u8; 32] = Default::default();
    keccak.update(caller_bytes);
    keccak.finalize(&mut result);

    let uref = merkle_tree_root_uref();
    let root = read_merkle_tree_root_from(uref);

    let mut computed_hash: &[u8; 32] = &result;
    let mut result: [u8; 32] = Default::default();
    for proof_item in converted_proof {
        match proof_item.1 {
            Position::Right => {
                let mut keccak = tiny_keccak::Keccak::v256();
                keccak.update(computed_hash);
                keccak.update(&proof_item.0);
                keccak.finalize(&mut result);
            }
            Position::Left => {
                let mut keccak = tiny_keccak::Keccak::v256();
                keccak.update(&proof_item.0);
                keccak.update(computed_hash);
                keccak.finalize(&mut result);
            }
        }
        computed_hash = &result;
    }

    let computed_root = hex::encode(*computed_hash);

    if root != computed_root {
        runtime::revert(Error::NotWhiteListed);
    }
}
