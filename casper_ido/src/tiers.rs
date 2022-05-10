//! Implementation of tiers.
use alloc::{string::String, vec::Vec};

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{account::AccountHash, bytesrepr::ToBytes, URef, U256};

use crate::{constants::TIERS_KEY_NAME, detail};

/// Creates a dictionary item key for a dictionary item.
#[inline]
fn make_dictionary_item_key(user: AccountHash, project_id: String) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut project_id.to_bytes().unwrap_or_revert());
    preimage.append(&mut project_id.to_bytes().unwrap_or_revert());
    hex::encode(&preimage)
}

pub(crate) fn get_tiers_uref() -> URef {
    detail::get_uref(TIERS_KEY_NAME)
}

/// Writes tier
pub(crate) fn write_tier_to(tiers_uref: URef, user: AccountHash, project_id: String, amount: U256) {
    let dictionary_item_key = make_dictionary_item_key(user, project_id);
    storage::dictionary_put(tiers_uref, &dictionary_item_key, amount);
}

/// Reads tier of user, project pair
pub(crate) fn read_tier_from(tiers_uref: URef, user: AccountHash, project_id: String) -> U256 {
    let dictionary_item_key = make_dictionary_item_key(user, project_id);

    storage::dictionary_get(tiers_uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap_or_default()
}

pub(crate) fn write_multiple_tiers(tiers: Vec<(String, String, U256)>) {
    let converted_tiers: Vec<(AccountHash, String, U256)> = tiers
        .iter()
        .map(|tier| {
            (
                AccountHash::from_formatted_str(&tier.0).unwrap(),
                tier.1.clone(),
                tier.2,
            )
        })
        .collect();
    let tiers_uref = get_tiers_uref();
    for tier in converted_tiers {
        write_tier_to(tiers_uref, tier.0, tier.1, tier.2);
    }
}

pub(crate) fn write_tier(tier: (String, String, U256)) {
    let converted_tier = (
        AccountHash::from_formatted_str(&tier.0).unwrap(),
        tier.1.clone(),
        tier.2,
    );

    let tiers_uref = get_tiers_uref();
    write_tier_to(
        tiers_uref,
        converted_tier.0,
        converted_tier.1,
        converted_tier.2,
    );
}
