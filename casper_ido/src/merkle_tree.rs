//! Implementation of merkle_tree.
use core::convert::{TryFrom, TryInto};

use alloc::vec::Vec;
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_erc20::Address;
use casper_types::URef;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleProof};

use crate::{constants::MERKLE_ROOT_KEY_NAME, detail, error::Error};

#[inline]
pub(crate) fn merkle_tree_root_uref() -> URef {
    detail::get_uref(MERKLE_ROOT_KEY_NAME)
}

/// Reads a merkle_tree from a specified [`URef`].
pub(crate) fn read_merkle_tree_root_from(uref: URef) -> Vec<u8> {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

/// Writes a merkle_tree to a specific [`URef`].
pub(crate) fn write_merkle_tree_root_to(uref: URef, value: Vec<u8>) {
    storage::write(uref, value);
}

pub(crate) fn verify_whitelist(proof_bytes: Vec<u8>, index_to_prove: usize, leaves_len: usize) {
    let merkle_tree_root_uref: URef = merkle_tree_root_uref();
    let merkle_tree_root_vec: Vec<u8> = read_merkle_tree_root_from(merkle_tree_root_uref);
    let merkle_tree_root = merkle_tree_root_vec.try_into().unwrap();
    let proof = MerkleProof::<Sha256>::try_from(proof_bytes).unwrap();

    let caller: Address = detail::get_immediate_caller_address().unwrap_or_revert();
    let caller_bytes = caller.as_account_hash().unwrap().as_bytes();
    let leaves_to_prove = [Sha256::hash(caller_bytes)];
    let mut indices_to_prove: Vec<usize> = Vec::new();
    indices_to_prove.push(index_to_prove);
    let verify_result = proof.verify(
        merkle_tree_root,
        &indices_to_prove,
        &leaves_to_prove,
        leaves_len,
    );
    if !verify_result {
        runtime::revert(Error::PermissionDenied)
    }
}
