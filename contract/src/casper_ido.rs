use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{account::AccountHash, ApiError, U256};
use contract_utils::{set_key, ContractContext, ContractStorage};

use crate::{
    create_auction_purse,
    data::{FeeDeNominator, TreasuryWallet},
    enums::{Address, BiddingToken},
    event::{self, CasperIdoEvent},
    libs::merkle_tree,
    structs::{Auction, Tiers, Time},
    Auctions, Error, IERC20,
};

pub trait CasperIdo<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(&mut self, default_treasury_wallet: Address) {
        Auctions::init();
        merkle_tree::init();
        TreasuryWallet::init(default_treasury_wallet);
        FeeDeNominator::init(U256::exp10(5));

        set_key("install_time", u64::from(runtime::get_blocktime()));
    }

    fn create_auction(&mut self, id: String, auction: Auction) {
        match Auctions::instance().get(&id) {
            Some(_exist_auction) => {
                runtime::revert(Error::AlreadyExistAuction);
            }
            None => {
                create_auction_purse(&id);
                Auctions::instance().set(&id, auction.clone());
                // self.emit(CasperIdoEvent::AuctionCreated { aution })
            }
        }
    }

    fn create_order(
        &mut self,
        caller: AccountHash,
        auction_id: String,
        proof: Vec<(String, u8)>,
        amount: U256,
    ) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        let current_block_time = runtime::get_blocktime();
        if !auction.is_auction_time(u64::from(current_block_time)) {
            runtime::revert(Error::NotValidTime);
        }
        let leaf = caller.to_string();

        merkle_tree::verify(auction.merkle_root.clone(), leaf, proof);

        // TODO SALE TIME ASSERT
        if !auction.is_auction_time(u64::from(runtime::get_blocktime())) {
            runtime::revert(Error::NotValidTime);
        };

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

    fn cancel_order(&mut self, caller: AccountHash, auction_id: String) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);

        match auction.orders.get(&caller) {
            Some(_order_amount) => {
                auction.orders.remove(&caller);
            }
            None => runtime::revert(Error::NotExistOrder),
        }
    }

    fn claim(&mut self, caller: AccountHash, auction_id: String, schedule_time: Time) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);

        if auction.claims.get(&(caller, schedule_time)).is_some() {
            runtime::revert(Error::AlreadyClaimed);
        }

        // TODO after sale time

        let order_amount = *auction
            .orders
            .get(&caller)
            .unwrap_or_revert_with(Error::NotExistOrder);
        let schedule_percent = *auction
            .schedules
            .get(&schedule_time)
            .unwrap_or_revert_with(Error::NotValidSchedule);
        let percent_denominator = U256::exp10(5);
        let transfer_amount_in_usd = order_amount
            .checked_add(schedule_percent)
            .unwrap_or_revert()
            .checked_div(percent_denominator)
            .unwrap_or_revert();
        let auction_token_instance = IERC20::new(auction.auction_token);
        let transfer_amount = {
            let auction_token_decimals = auction_token_instance.decimals();
            let auction_token_price_in_usd = auction.auction_token_price;
            transfer_amount_in_usd
                .checked_div(auction_token_price_in_usd)
                .unwrap_or_revert()
                .checked_mul(U256::from(auction_token_decimals))
                .unwrap_or_revert()
        };
        auction_token_instance.transfer(Address::from(caller), transfer_amount);
        auction.claims.insert((caller, schedule_time), true);
        Auctions::instance().set(&auction_id, auction);
    }

    fn set_cspr_price(&mut self) {
        let auction_id: String = runtime::get_named_arg("auction_id");
        let new_price: U256 = runtime::get_named_arg("price");

        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        // Only auction creator
        if auction.creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }
        match auction.bidding_token {
            BiddingToken::Native { price: _ } => {
                auction.bidding_token = BiddingToken::Native {
                    price: Some(new_price),
                };
            }
            _ => {
                runtime::revert(ApiError::InvalidArgument);
            }
        }
        Auctions::instance().set(&auction_id, auction);
    }

    fn set_tiers(&mut self) {
        let auction_id: String = runtime::get_named_arg("auction_id");
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        if auction.creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }
        let mut tiers: Tiers = runtime::get_named_arg("tiers");
        auction.tiers.append(&mut tiers);
        Auctions::instance().set(&auction_id, auction);
    }

    fn set_merkle_root(&mut self) {
        let auction_id: String = runtime::get_named_arg("auction_id");
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        if auction.creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }
        let merkle_root: String = runtime::get_named_arg("merkle_root");
        auction.merkle_root = Some(merkle_root);
        Auctions::instance().set(&auction_id, auction);
    }

    fn get_fee_denominator(&self) -> U256 {
        FeeDeNominator::instance().get_fee_denominator()
    }

    fn get_treasury_wallet(&self) -> Address {
        TreasuryWallet::instance().get_treasury_wallet()
    }

    fn emit(&mut self, event: CasperIdoEvent) {
        event::emit(&event);
    }
}
