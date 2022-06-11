use alloc::{collections::BTreeMap, string::String, vec::Vec};
use casper_contract::contract_api::runtime;
use casper_types::{
    account::AccountHash,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, ContractHash, U256,
};

use crate::{enums::BiddingToken, Error};

pub type Time = u64;

/// order amount unit is USD
pub type Orders = BTreeMap<AccountHash, U256>;

pub type Claims = BTreeMap<(AccountHash, Time), bool>;

pub type Schedules = BTreeMap<Time, U256>;

#[derive(Debug, Clone)]
pub struct Auction {
    pub id: String,
    pub info: String,
    pub creator: AccountHash,
    pub auction_created_time: Time,
    pub auction_start_time: Time,
    pub auction_end_time: Time,
    pub project_open_time: Time,
    pub auction_token: ContractHash,
    pub auction_token_price: U256,
    pub auction_token_capacity: U256,
    pub bidding_token: BiddingToken,
    pub fee_numerator: u8,
    pub orders: Orders,
    pub claims: Claims,
    pub schedules: Schedules,
    pub merkle_root: Option<String>, // if None use default merkle_root
}

impl CLTyped for Auction {
    fn cl_type() -> casper_types::CLType {
        CLType::Any
    }
}

impl ToBytes for Auction {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        buffer.extend(self.id.to_bytes()?);
        buffer.extend(self.info.to_bytes()?);
        buffer.extend(self.creator.to_bytes()?);
        buffer.extend(self.auction_created_time.to_bytes()?);
        buffer.extend(self.auction_start_time.to_bytes()?);
        buffer.extend(self.auction_end_time.to_bytes()?);
        buffer.extend(self.project_open_time.to_bytes()?);
        buffer.extend(self.auction_token.to_bytes()?);
        buffer.extend(self.auction_token_price.to_bytes()?);
        buffer.extend(self.auction_token_capacity.to_bytes()?);
        buffer.extend(self.bidding_token.to_bytes()?);
        buffer.extend(self.fee_numerator.to_bytes()?);
        buffer.extend(self.orders.to_bytes()?);
        buffer.extend(self.claims.to_bytes()?);
        buffer.extend(self.schedules.to_bytes()?);
        buffer.extend(self.merkle_root.to_bytes()?);

        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        self.id.serialized_length()
            + self.info.serialized_length()
            + self.creator.serialized_length()
            + self.auction_created_time.serialized_length()
            + self.auction_start_time.serialized_length()
            + self.auction_end_time.serialized_length()
            + self.project_open_time.serialized_length()
            + self.auction_token.serialized_length()
            + self.auction_token_price.serialized_length()
            + self.auction_token_capacity.serialized_length()
            + self.bidding_token.serialized_length()
            + self.fee_numerator.serialized_length()
            + self.orders.serialized_length()
            + self.claims.serialized_length()
            + self.schedules.serialized_length()
            + self.merkle_root.serialized_length()
    }
}

impl FromBytes for Auction {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (id, bytes) = String::from_bytes(bytes)?;
        let (info, bytes) = String::from_bytes(bytes)?;
        let (creator, bytes) = AccountHash::from_bytes(bytes)?;
        let (auction_created_time, bytes) = Time::from_bytes(bytes)?;
        let (auction_start_time, bytes) = Time::from_bytes(bytes)?;
        let (auction_end_time, bytes) = Time::from_bytes(bytes)?;
        let (project_open_time, bytes) = Time::from_bytes(bytes)?;
        let (auction_token, bytes) = ContractHash::from_bytes(bytes)?;
        let (auction_token_price, bytes) = U256::from_bytes(bytes)?;
        let (auction_token_capacity, bytes) = U256::from_bytes(bytes)?;
        let (bidding_token, bytes) = BiddingToken::from_bytes(bytes)?;
        let (fee_numerator, bytes) = u8::from_bytes(bytes)?;
        let (orders, bytes) = Orders::from_bytes(bytes)?;
        let (claims, bytes) = Claims::from_bytes(bytes)?;
        let (schedules, bytes) = Schedules::from_bytes(bytes)?;
        let (merkle_root, bytes) = Option::<String>::from_bytes(bytes)?;

        Ok((
            Self {
                id,
                info,
                auction_created_time,
                auction_start_time,
                auction_end_time,
                project_open_time,
                auction_token,
                auction_token_price,
                auction_token_capacity,
                bidding_token,
                creator,
                fee_numerator,
                orders,
                claims,
                schedules,
                merkle_root,
            },
            bytes,
        ))
    }
}

impl Auction {
    pub fn assert_before_auction_time(&self) {
        let current_block_time = runtime::get_blocktime();
        if !self.is_before_auction_time(u64::from(current_block_time)) {
            runtime::revert(Error::InvalidTime);
        }
    }

    pub fn assert_auction_time(&self) {
        let current_block_time = runtime::get_blocktime();
        if !self.is_auction_time(u64::from(current_block_time)) {
            runtime::revert(Error::InvalidTime);
        }
    }

    fn is_before_auction_time(&self, time: Time) -> bool {
        time.lt(&self.auction_start_time)
    }

    fn is_auction_time(&self, time: Time) -> bool {
        time.gt(&self.auction_start_time) && time.lt(&self.auction_end_time)
    }
}
