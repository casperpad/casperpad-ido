use casper_types::{
    bytesrepr::ToBytes,
    bytesrepr::{self, FromBytes},
    CLType, CLTyped, U256,
};

use alloc::{string::String, vec::Vec};

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    Upcoming,
    Going,
    Completed,
    Paused,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenInfo {
    pub token_price: U256,
    pub token_symbol: String,
    pub total_supply: U256,
}

// struct ScheduleInfo {}

mod my_date_format {
    use alloc::{format, string::String};
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub private: bool,
    #[serde(with = "my_date_format")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "my_date_format")]
    pub end_time: DateTime<Utc>,
    pub status: Status,
    pub token_info: TokenInfo,
    pub whitelisted_users_length: U256,
    // whitelisted_users: URef,
}

// The struct `Project` can be treated as CLType
impl CLTyped for Project {
    fn cl_type() -> CLType {
        CLType::ByteArray(10u32)
    }
}

// Serialize for Project
impl ToBytes for Project {
    #[inline(always)]
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let serialized = serde_json::to_string(&self).unwrap();
        Ok(serialized.as_bytes().to_vec())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        let serialized = serde_json::to_string(&self).unwrap();
        serialized.as_bytes().len()
    }

    fn into_bytes(self) -> Result<Vec<u8>, casper_types::bytesrepr::Error>
    where
        Self: Sized,
    {
        self.to_bytes()
    }
}

// Deserialize for Project
impl FromBytes for Project {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (string, remainder) = String::from_bytes(bytes).unwrap();
        let project: Project = serde_json::from_str(&string).unwrap();
        Ok((project, remainder))
    }
}

impl Project {
    pub fn new(
        id: &str,
        name: &str,
        private: bool,
        start_time: u32,
        end_time: u32,
        token_info: TokenInfo,
    ) -> Self {
        Self {
            id: String::from(id),
            name: String::from(name),
            private,
            start_time: Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, start_time),
            end_time: Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, end_time),
            status: Status::Upcoming,
            whitelisted_users_length: U256::from(0),
            token_info, // whitelisted_users:
        }
    }
}
