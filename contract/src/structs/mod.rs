use alloc::collections::BTreeMap;
use casper_types::U256;

pub type Time = u64;

pub type Schedules = BTreeMap<Time, U256>;
