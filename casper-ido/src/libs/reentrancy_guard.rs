#![allow(dead_code)]

use core::convert::TryInto;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    ApiError, CLType, CLTyped, Key, URef,
};

pub const REENTRANCY_GUARD_KEY_NAME: &str = "reentrancy_guard";

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum ReentrancyGuard {
    NotEntered = 0,
    Entered = 1,
}

impl CLTyped for ReentrancyGuard {
    fn cl_type() -> casper_types::CLType {
        CLType::U8
    }
}

impl ToBytes for ReentrancyGuard {
    fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, casper_types::bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        buffer.extend((*self as u8).to_bytes()?);
        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        (*self as u8).serialized_length()
    }

    fn into_bytes(self) -> Result<alloc::vec::Vec<u8>, bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
}

impl FromBytes for ReentrancyGuard {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (result, bytes) = u8::from_bytes(bytes).unwrap();
        Ok((ReentrancyGuard::from(result), bytes))
    }
}

impl ReentrancyGuard {
    pub fn from(value: u8) -> Self {
        match value {
            0 => ReentrancyGuard::NotEntered,
            1 => ReentrancyGuard::Entered,
            _ => panic!(),
        }
    }
}

impl Default for ReentrancyGuard {
    fn default() -> Self {
        ReentrancyGuard::NotEntered
    }
}

#[inline]
pub(crate) fn reentrancy_guard_uref() -> URef {
    let key = runtime::get_key(REENTRANCY_GUARD_KEY_NAME).unwrap_or({
        let key: Key = storage::new_uref(ReentrancyGuard::default()).into();
        runtime::put_key(REENTRANCY_GUARD_KEY_NAME, key);
        key
    });

    key.try_into().unwrap_or_revert()
}

pub(crate) fn read_reentrancy_guard() -> ReentrancyGuard {
    let uref = reentrancy_guard_uref();
    storage::read(uref).unwrap_or_revert().unwrap_or_default()
}

pub(crate) fn write_reentrancy_guard(value: ReentrancyGuard) {
    let uref = reentrancy_guard_uref();
    storage::write(uref, value);
}

pub(crate) fn assert_reentrancy() {
    let reentrancy = read_reentrancy_guard();
    if !reentrancy.eq(&ReentrancyGuard::NotEntered) {
        runtime::revert(ApiError::PermissionDenied);
    }
}

pub(crate) fn set_reentrancy() {
    assert_reentrancy();
    write_reentrancy_guard(ReentrancyGuard::Entered);
}

pub(crate) fn clear_reentrancy() {
    write_reentrancy_guard(ReentrancyGuard::NotEntered);
}
