use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{ContractHash, U256};
use contract_utils::{ContractContext, ContractStorage};

use crate::{
    create_auction_purse,
    enums::BiddingToken,
    event::{self, CasperIdoEvent},
    libs::merkle_tree,
    structs::{Auction, Claims, Orders, Schedules, Tiers, Time},
    Auctions, Error,
};

pub trait CasperIdo<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(&mut self) {
        Auctions::init();
        merkle_tree::init();
    }

    fn create_auction(&mut self) {
        let id: String = runtime::get_named_arg("id");
        let info: String = runtime::get_named_arg("info");
        let creator = runtime::get_caller();
        let auction_created_time = Time::from(runtime::get_blocktime());
        let auction_start_time: Time = runtime::get_named_arg("auction_start_time");
        let auction_end_time: Time = runtime::get_named_arg("auction_end_time");
        let project_open_time: Time = runtime::get_named_arg("project_open_time");
        let auction_token = {
            let auction_token_string: String = runtime::get_named_arg("auction_token");
            ContractHash::from_formatted_str(&auction_token_string).unwrap()
        };
        let auction_token_price: U256 = runtime::get_named_arg("auction_token_price");
        let auction_token_capacity: U256 = runtime::get_named_arg("auction_token_capacity");
        let bidding_token: BiddingToken = runtime::get_named_arg("bidding_token");
        let fee_numerator: u8 = runtime::get_named_arg("fee_numerator");
        let schedules: Schedules = runtime::get_named_arg("schedules");
        let merkle_root: Option<String> = runtime::get_named_arg("merkle_root");
        let tiers: Tiers = runtime::get_named_arg("tiers");

        match Auctions::instance().get(&id) {
            Some(_exist_auction) => {
                runtime::revert(Error::AlreadyExistAuction);
            }
            None => {
                create_auction_purse(&id);
                Auctions::instance().set(
                    &id,
                    Auction {
                        id: id.clone(),
                        info: info.clone(),
                        creator,
                        auction_created_time,
                        auction_start_time,
                        auction_end_time,
                        project_open_time,
                        auction_token,
                        auction_token_price,
                        auction_token_capacity,
                        bidding_token: bidding_token.clone(),
                        fee_numerator,
                        orders: Orders::new(),
                        claims: Claims::new(),
                        schedules: schedules.clone(),
                        merkle_root,
                        tiers,
                    },
                );
                self.emit(CasperIdoEvent::AuctionCreated {
                    id: id.clone(),
                    info: info.clone(),
                    creator,
                    auction_created_time,
                    auction_start_time,
                    auction_end_time,
                    project_open_time,
                    auction_token,
                    auction_token_price,
                    auction_token_capacity,
                    bidding_token: bidding_token.clone(),
                    schedules: schedules.clone(),
                })
            }
        }
    }

    fn create_order(&mut self) {
        let caller = runtime::get_caller();
        let auction_id: String = runtime::get_named_arg("auction_id");
        let proof: Vec<(String, u8)> = runtime::get_named_arg("proof");
        let amount: U256 = runtime::get_named_arg("amount");

        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        let leaf = caller.to_string();
        merkle_tree::verify(auction.merkle_root.clone(), leaf, proof);
        let exist_order_amount = {
            let balance = auction.orders.get(&caller);
            match balance {
                Some(x) => *x,
                None => U256::default(),
            }
        };
        match auction.tiers.get(&caller) {
            Some(tier) => {
                if tier.lt(&amount.checked_add(exist_order_amount).unwrap()) {
                    runtime::revert(Error::OutOfTier);
                }
            }
            None => {
                runtime::revert(Error::TierNotSetted);
            }
        }

        auction
            .orders
            .insert(caller, amount.checked_add(exist_order_amount).unwrap());
        Auctions::instance().set(&auction_id, auction);
    }

    fn set_cspr_price(&mut self) {}

    fn set_multiple_tiers(&mut self) {}
    fn set_tier(&mut self) {}

    fn set_merkle_root(&mut self) {}

    fn emit(&mut self, event: CasperIdoEvent) {
        event::emit(&event);
    }
}
