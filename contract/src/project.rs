use casper_types::Key;

use alloc::string::String;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    Upcoming,
    Going,
    Completed,
    Paused,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenInfo {
    pub token_price: u32,
    pub token_symbol: String,
    pub total_supply: u32,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub private: bool,
    #[serde(with = "my_date_format")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "my_date_format")]
    pub end_time: DateTime<Utc>,
    pub token_info: TokenInfo,
    pub status: Key,
    pub claim_status: Key,
    pub users_length: Key,
}

impl Project {
    pub fn new(
        id: &str,
        name: &str,
        private: bool,
        start_time: i64,
        end_time: i64,
        token_info: TokenInfo,
        status: Key,
        claim_status: Key,
        users_length: Key,
    ) -> Self {
        Self {
            id: String::from(id),
            name: String::from(name),
            private,
            start_time: Utc.timestamp_millis(start_time),
            end_time: Utc.timestamp_millis(end_time),
            status,
            claim_status,
            token_info,
            users_length,
        }
    }
    pub fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    pub fn deserialize(value: String) -> Project {
        let deserialized: Project = serde_json::from_str(&value).unwrap();
        deserialized
    }
}
