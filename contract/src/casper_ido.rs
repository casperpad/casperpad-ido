use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use casper_contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{account::AccountHash, ApiError, ContractHash, URef, U256};
use contract_utils::{ContractContext, ContractStorage};

use crate::{
    auction_purse, create_auction_purse,
    data::{FeeDeNominator, TreasuryWallet},
    enums::{Address, BiddingToken},
    event::{self, CasperIdoEvent},
    libs::{
        conversion::{u256_to_512, u512_to_u256},
        merkle_tree,
    },
    structs::{Auction, Tiers, Time},
    Auctions, Error, IERC20,
};

pub trait CasperIdo<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(&mut self, default_treasury_wallet: Address) {
        Auctions::init();
        merkle_tree::init();
        TreasuryWallet::init(default_treasury_wallet);
        FeeDeNominator::init(U256::exp10(4));
    }

    /// Create auction
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

    /// Create order, caller must be whitelisted and can creat in sale time.
    fn create_order(
        &mut self,
        caller: AccountHash,
        auction_id: String,
        proof: Vec<(String, u8)>,
        token: ContractHash,
        amount: U256,
    ) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);

        // Check caller is whitelisted
        let leaf = caller.to_string();
        merkle_tree::verify(auction.merkle_root.clone(), leaf, proof);

        // Check current time is between sale time
        auction.assert_auction_time();

        // Check payment is right
        let order_amount_in_usd = match auction.bidding_token.clone() {
            BiddingToken::Native { price: _ } => {
                runtime::revert(Error::InvalidPayToken);
            }
            BiddingToken::ERC20s { tokens_with_price } => {
                let paytoken_price = tokens_with_price
                    .get(&token)
                    .unwrap_or_revert_with(Error::InvalidPayToken);
                IERC20::new(token).transfer_from(
                    Address::from(caller),
                    Address::from(auction.creator),
                    amount,
                );
                amount.checked_mul(*paytoken_price).unwrap_or_revert()
            }
        };

        // Check order amount is less than tier
        let exist_order_amount = *auction.orders.get(&caller).unwrap_or(&U256::zero());

        let unchecked_new_order_amount =
            order_amount_in_usd.checked_add(exist_order_amount).unwrap();

        let caller_tier = *auction
            .tiers
            .get(&caller)
            .unwrap_or_revert_with(Error::TierNotSetted);

        if caller_tier.lt(&unchecked_new_order_amount) {
            runtime::revert(Error::OutOfTier);
        }

        auction.orders.insert(caller, unchecked_new_order_amount);
        Auctions::instance().set(&auction_id, auction);
    }

    fn create_order_cspr(
        &mut self,
        caller: AccountHash,
        auction_id: String,
        proof: Vec<(String, u8)>,
        deposit_purse: URef,
    ) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);

        // Check caller is whitelisted
        let leaf = caller.to_string();
        merkle_tree::verify(auction.merkle_root.clone(), leaf, proof);

        // Check current time is between sale time
        // TODO Check
        auction.assert_auction_time();

        // Check payment is right
        let order_amount_in_usd = match auction.bidding_token.clone() {
            BiddingToken::Native { price } => {
                let purse_balance = system::get_purse_balance(deposit_purse).unwrap_or_default();
                system::transfer_from_purse_to_account(
                    deposit_purse,
                    auction.creator,
                    purse_balance,
                    None,
                )
                .unwrap();
                u512_to_u256(&purse_balance)
                    .unwrap_or_revert()
                    .checked_mul(price.unwrap_or_revert())
                    .unwrap_or_revert()
                    .checked_div(U256::exp10(9))
                    .unwrap_or_revert()
            }
            BiddingToken::ERC20s {
                tokens_with_price: _,
            } => {
                runtime::revert(Error::InvalidPayToken);
            }
        };

        // Check order amount is less than tier
        let exist_order_amount = *auction.orders.get(&caller).unwrap_or(&U256::zero());

        let unchecked_new_order_amount =
            order_amount_in_usd.checked_add(exist_order_amount).unwrap();

        let caller_tier = *auction
            .tiers
            .get(&caller)
            .unwrap_or_revert_with(Error::TierNotSetted);

        if caller_tier.lt(&unchecked_new_order_amount) {
            runtime::revert(Error::OutOfTier);
        }

        auction.orders.insert(caller, unchecked_new_order_amount);
        Auctions::instance().set(&auction_id, auction);
    }

    /// Cancel order, currently support CSPR
    fn cancel_order(&mut self, caller: AccountHash, auction_id: String) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        auction.assert_before_auction_time();

        match auction.orders.get(&caller) {
            Some(order_amount) => {
                match auction.bidding_token {
                    BiddingToken::Native { price: _ } => {
                        let auction_purese = auction_purse(&auction_id);
                        system::transfer_from_purse_to_account(
                            auction_purese,
                            runtime::get_caller(),
                            u256_to_512(order_amount).unwrap(),
                            None,
                        )
                        .unwrap();
                    }
                    BiddingToken::ERC20s {
                        tokens_with_price: _,
                    } => {
                        runtime::revert(Error::PermissionDenied);
                    }
                }
                auction.orders.remove(&caller);
                Auctions::instance().set(&auction_id, auction);
            }
            None => runtime::revert(Error::NotExistOrder),
        }
    }

    /// Whitelisted user can claim after schedule time
    fn claim(&mut self, caller: AccountHash, auction_id: String, schedule_time: Time) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);

        let current_block_time = runtime::get_blocktime();

        if schedule_time.lt(&u64::from(current_block_time)) {
            runtime::revert(Error::InvalidTime);
        }

        let order_amount = *auction
            .orders
            .get(&caller)
            .unwrap_or_revert_with(Error::NotExistOrder);

        if auction.claims.get(&(caller, schedule_time)).is_some() {
            runtime::revert(Error::AlreadyClaimed);
        }
        let schedule_percent = *auction
            .schedules
            .get(&schedule_time)
            .unwrap_or_revert_with(Error::InvalidSchedule);
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

    /// Set CSPR price for given auction, if ERC20 token is used for payment abort, should set to only admin call
    fn set_cspr_price(&mut self, auction_id: String, price: U256) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        // Only auction creator
        if auction.creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }
        // Can set CSPR price before sale time
        auction.assert_before_auction_time();

        match auction.bidding_token {
            BiddingToken::Native { price: _ } => {
                auction.bidding_token = BiddingToken::Native { price: Some(price) };
            }
            _ => {
                runtime::revert(ApiError::InvalidArgument);
            }
        }
        Auctions::instance().set(&auction_id, auction);
    }

    fn set_tiers(&mut self, auction_id: String, tiers: &mut Tiers) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        if auction.creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }
        auction.tiers.append(tiers);
        Auctions::instance().set(&auction_id, auction);
    }

    /// Set merkle_root for given aution, should set to only admin call
    fn set_merkle_root(&mut self, auction_id: String, merkle_root: String) {
        let mut auction = Auctions::instance()
            .get(&auction_id)
            .unwrap_or_revert_with(Error::NotExistAuction);
        if auction.creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }

        auction.merkle_root = Some(merkle_root);
        Auctions::instance().set(&auction_id, auction);
    }

    fn set_fee_denominator(&self, fee_denominator: U256) {
        FeeDeNominator::instance().set_fee_denominator(fee_denominator);
    }

    fn get_fee_denominator(&self) -> U256 {
        FeeDeNominator::instance().get_fee_denominator()
    }

    /// Set treasury wallet for fee, should set to only admin call
    fn set_treasury_wallet(&self, treasury_wallet: Address) {
        TreasuryWallet::instance().set_treasury_wallet(treasury_wallet);
    }

    fn get_treasury_wallet(&self) -> Address {
        TreasuryWallet::instance().get_treasury_wallet()
    }

    fn emit(&mut self, event: CasperIdoEvent) {
        event::emit(&event);
    }
}
