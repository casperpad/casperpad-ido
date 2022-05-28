use casper_types::{account::AccountHash, ContractHash, U256};

use crate::{
    enums::BiddingToken,
    structs::{Schedules, Time},
};

pub enum CasperIdoEvent {
    AuctionCreated {
        id: String,
        info: String,
        creator: AccountHash,
        auction_created_time: Time,
        auction_start_time: Time,
        auction_end_time: Time,
        project_open_time: Time,
        auction_token: ContractHash,
        auction_token_price: U256,
        auction_token_capacity: U256,
        bidding_token: BiddingToken,
        schedules: Schedules,
    },
}

pub(crate) fn emit(event: &CasperIdoEvent) {}
