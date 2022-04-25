use casper_types::{
    bytesrepr::ToBytes,
    bytesrepr::{self, FromBytes},
    CLType, CLTyped, Key, URef, U256,
};

enum Status {
    Upcoming,
    Going,
    Completed,
    Paused,
    Cancelled,
}

struct TokenInfo {
    token_price: U256,
    total_supply: U256,
}

struct ScheduleInfo {}

struct Project {
    id: String,
    name: String,
    start_time: U256,
    end_time: U256,

    status: Status,
    whitelisted_users_length: U256,
    whitelisted_users: URef,
}

// The struct `Project` can me treated as CLType
impl CLTyped for Project {
    fn cl_type() -> CLType {
        CLType::ByteArray(10u32)
    }
}

// Serialize for Project
impl ToBytes for Project {
    #[inline(always)]
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let bytes = self.id.as_bytes();
        Ok(bytes.to_vec())
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.id.serialized_length()
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
        let project = Project::new(string.as_str());
        Ok((project, remainder))
    }
}

impl Project {
    fn new(id: &str) -> Self {
        Self {
            id: String::from(id),
        }
    }
}
