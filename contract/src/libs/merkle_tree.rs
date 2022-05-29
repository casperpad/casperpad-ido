#![allow(dead_code)]
//! Implementation of merkle_tree.
use core::convert::TryInto;

use alloc::string::String;

use alloc::vec::Vec;

use casper_contract::contract_api::runtime;
use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};

use casper_types::{ApiError, Key, URef};
use tiny_keccak::Hasher;

const MERKLE_ROOT_KEY_NAME: &str = "merkle_root";

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
    let key = runtime::get_key(MERKLE_ROOT_KEY_NAME).unwrap_or({
        let key: Key = storage::new_uref("").into();
        runtime::put_key(MERKLE_ROOT_KEY_NAME, key);
        key
    });

    key.try_into().unwrap_or_revert()
}

/// Reads a merkle_tree from a specified [`URef`].
pub(crate) fn read_merkle_tree_root_from(uref: URef) -> String {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

/// Writes a merkle_tree to a specific [`URef`].
pub(crate) fn write_merkle_tree_root_to(uref: URef, value: String) {
    storage::write(uref, value);
}

/// Verify leaf is in the tree
pub(crate) fn verify(root: Option<String>, leaf: String, proof: Vec<(String, u8)>) {
    let converted_proof: Vec<(Vec<u8>, Position)> = proof
        .iter()
        .map(|proof| {
            (
                hex::decode(proof.0.clone()).unwrap(),
                Position::from_u8(proof.1),
            )
        })
        .collect();

    let leaf_bytes: &[u8] = leaf.as_bytes();

    let mut keccak = tiny_keccak::Keccak::v256();
    let mut result: [u8; 32] = Default::default();
    keccak.update(leaf_bytes);
    keccak.finalize(&mut result);

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

    let root_to_check = match root {
        Some(root) => root,
        _ => {
            let uref = merkle_tree_root_uref();
            read_merkle_tree_root_from(uref)
        }
    };
    if !computed_root.eq(&root_to_check) {
        runtime::revert(ApiError::PermissionDenied);
    }
}

pub(crate) fn init() {
    let _ = merkle_tree_root_uref();
}
