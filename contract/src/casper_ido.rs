use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use casper_contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    account::AccountHash, ApiError, ContractHash, ContractPackageHash, Key, URef, U256,
};
use contract_utils::{set_key, ContractContext, ContractStorage};

use crate::{
    data::{
        set_creator, set_factory_contract, set_info, set_launch_time, set_schedules, Claims,
        Orders, _get_merkle_root, _set_merkle_root, get_auction_end_time, get_auction_start_time,
        get_auction_token, get_auction_token_capacity, get_auction_token_price, get_bidding_token,
        get_creator, get_factory_contract, get_schedules, set_auction_end_time,
        set_auction_start_time, set_auction_token, set_auction_token_capacity,
        set_auction_token_price, set_bidding_token,
    },
    enums::{Address, BiddingToken},
    event::{self, CasperIdoEvent},
    interfaces::IFactory,
    libs::{conversion::u512_to_u256, merkle_tree},
    structs::{Schedules, Time},
    Error, IERC20,
};

pub trait CasperIdo<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(
        &mut self,
        factory_contract: ContractHash,
        info: &str,
        auction_start_time: Time,
        auction_end_time: Time,
        launch_time: Time,
        auction_token: Option<ContractHash>,
        auction_token_price: U256,
        auction_token_capacity: U256,
        bidding_token: BiddingToken,
        schedules: Schedules,
    ) {
        set_info(info);
        set_creator(runtime::get_caller());
        set_auction_start_time(auction_start_time);
        set_auction_end_time(auction_end_time);
        set_launch_time(launch_time);
        if auction_token.is_some() {
            set_auction_token(auction_token.unwrap());
        }
        set_auction_token_price(auction_token_price);
        set_auction_token_capacity(auction_token_capacity);
        set_bidding_token(bidding_token);
        set_schedules(schedules);
        set_factory_contract(factory_contract);
        _set_merkle_root("".to_string());
        Orders::init();
        Claims::init();
    }

    fn contract_package_hash(&self) -> ContractPackageHash {
        let hash_addr = self.self_addr().into_hash().unwrap();
        ContractPackageHash::from(hash_addr)
    }

    fn set_auction_token(&mut self, auction_token: ContractHash) {
        self._assert_caller_is_admin();
        let auction_creator = get_creator();
        if auction_creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }
        self._assert_before_auction_time();
        IERC20::new(auction_token).transfer_from(
            Address::from(auction_creator),
            Address::from(self.contract_package_hash()),
            self.auction_token_capacity(),
        );
        set_auction_token(auction_token);
    }

    /// Create order, caller must be whitelisted and can creat in sale time.
    fn create_order(
        &mut self,
        caller: AccountHash,
        tier: U256,
        proof: Vec<(String, u8)>,
        token: ContractHash,
        amount: U256,
    ) {
        // Check caller is whitelisted
        let leaf = format!("{}_{:?}", caller.to_string(), tier);
        merkle_tree::verify(self.get_merkle_root(), leaf, proof);

        // Check current time is between sale time
        self._assert_auction_time();

        let bidding_token = get_bidding_token();

        // Check payment is right
        let order_amount_in_usd = match bidding_token.clone() {
            BiddingToken::Native { price: _ } => {
                runtime::revert(Error::InvalidPayToken);
            }
            BiddingToken::ERC20s { tokens_with_price } => {
                let paytoken_price = tokens_with_price
                    .get(&token)
                    .unwrap_or_revert_with(Error::InvalidPayToken);
                let auction_creator = get_creator();
                IERC20::new(token).transfer_from(
                    Address::from(caller),
                    Address::from(auction_creator),
                    amount,
                );
                amount.checked_mul(*paytoken_price).unwrap_or_revert()
            }
        };

        // Check order amount is less than tier
        let exist_order_amount = Orders::instance()
            .get(&Key::from(caller))
            .unwrap_or(U256::zero());

        let unchecked_new_order_amount =
            order_amount_in_usd.checked_add(exist_order_amount).unwrap();

        if tier.lt(&unchecked_new_order_amount) {
            runtime::revert(Error::OutOfTier);
        }

        Orders::instance().set(&Key::from(caller), unchecked_new_order_amount);
    }

    fn create_order_cspr(
        &mut self,
        caller: AccountHash,
        tier: U256,
        proof: Vec<(String, u8)>,
        deposit_purse: URef,
    ) {
        // Check caller is whitelisted
        let leaf = format!("{}_{:?}", caller.to_string(), tier);
        merkle_tree::verify(self.get_merkle_root(), leaf, proof);

        // Check current time is between auction time
        // self._assert_auction_time();

        let bidding_token = get_bidding_token();

        // Check payment is right
        let order_amount_in_usd = match bidding_token.clone() {
            BiddingToken::Native { price } => {
                let purse_balance = system::get_purse_balance(deposit_purse).unwrap_or_revert();
                let auction_creator = get_creator();
                system::transfer_from_purse_to_account(
                    deposit_purse,
                    auction_creator,
                    purse_balance,
                    None,
                )
                .unwrap_or_revert();
                u512_to_u256(&purse_balance)
                    .unwrap_or_revert()
                    .checked_mul(price.unwrap_or_revert_with(Error::InvalidCSPRPrice))
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
        let exist_order_amount = Orders::instance()
            .get(&Key::from(caller))
            .unwrap_or(U256::zero());

        let unchecked_new_order_amount =
            order_amount_in_usd.checked_add(exist_order_amount).unwrap();

        if tier.lt(&unchecked_new_order_amount) {
            runtime::revert(Error::OutOfTier);
        }

        Orders::instance().set(&Key::from(caller), unchecked_new_order_amount);
        let amount_stored = Orders::instance().get(&Key::from(caller));
        set_key("result", amount_stored);
    }

    /// Cancel order, experimental feature
    fn cancel_order(&mut self, caller: AccountHash) {
        let order_amount = Orders::instance()
            .get(&Key::from(caller))
            .unwrap_or_revert_with(Error::NotExistOrder);

        let bidding_token = get_bidding_token();

        match bidding_token {
            BiddingToken::Native { price: _ } => {
                // TODO REFUND
                // let auction_purese = auction_purse(&auction_id);
                // system::transfer_from_purse_to_account(
                //     auction_purese,
                //     runtime::get_caller(),
                //     u256_to_512(order_amount).unwrap(),
                //     None,
                // )
                // .unwrap();
            }
            BiddingToken::ERC20s {
                tokens_with_price: _,
            } => {
                runtime::revert(Error::PermissionDenied);
            }
        }
        Orders::instance().remove(&Key::from(caller));
    }

    /// Whitelisted user can claim after schedule time
    fn claim(&mut self, caller: AccountHash, schedule_time: Time) {
        let current_block_time = runtime::get_blocktime();

        // if schedule_time.lt(&u64::from(current_block_time)) {
        //     runtime::revert(Error::InvalidTime);
        // }

        let order_amount = Orders::instance()
            .get(&Key::from(caller))
            .unwrap_or_revert_with(Error::NotExistOrder);

        if Claims::instance()
            .get(&Key::from(caller), schedule_time)
            .is_some()
        {
            runtime::revert(Error::AlreadyClaimed);
        }

        let schedule_percent = *get_schedules()
            .get(&schedule_time)
            .unwrap_or_revert_with(Error::InvalidSchedule);

        let percent_denominator = U256::exp10(5);
        let transfer_amount_in_usd = order_amount
            .checked_mul(schedule_percent)
            .unwrap_or_revert()
            .checked_div(percent_denominator)
            .unwrap_or_revert();
        let auction_token_instance = IERC20::new(get_auction_token());
        let transfer_amount = {
            auction_token_instance.total_supply();
            // let auction_token_decimals = auction_token_instance.decimals();
            let auction_token_decimals = 18;

            // let auction_token_price_in_usd = get_auction_token_price();
            // transfer_amount_in_usd
            //     .checked_div(auction_token_price_in_usd)
            //     .unwrap_or_revert()
            //     .checked_mul(U256::from(auction_token_decimals))
            //     .unwrap_or_revert()
            U256::from(5)
        };
        set_key("result", transfer_amount_in_usd);
        auction_token_instance.transfer(Address::from(caller), transfer_amount);
        Claims::instance().set(&Key::from(caller), schedule_time, true);
    }

    /// Set CSPR price for given auction, if ERC20 token is used for payment abort, should set to only admin call
    fn set_cspr_price(&mut self, price: U256) {
        self._assert_caller_is_admin();

        // Can set CSPR price before sale time
        self._assert_before_auction_time();

        let auction_creator = get_creator();
        if auction_creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }

        let mut bidding_token = get_bidding_token();

        match bidding_token {
            BiddingToken::Native { price: _ } => {
                bidding_token = BiddingToken::Native { price: Some(price) };
            }
            _ => {
                runtime::revert(ApiError::InvalidArgument);
            }
        }
        set_bidding_token(bidding_token);
    }

    /// Set merkle_root , only admin call
    fn set_merkle_root(&mut self, merkle_root: String) {
        // self._assert_caller_is_admin();
        _set_merkle_root(merkle_root);
    }

    fn get_merkle_root(&self) -> String {
        _get_merkle_root()
    }

    fn auction_token_capacity(&self) -> U256 {
        get_auction_token_capacity()
    }

    fn _assert_caller_is_admin(&self) {
        // let factory_contract = get_factory_contract();
        // IFactory::new(factory_contract).assert_caller_is_admin(runtime::get_caller());
    }

    fn _assert_before_auction_time(&self) {
        let time = Time::from(runtime::get_blocktime());
        let auction_start_time = get_auction_start_time();
        if !time.lt(&auction_start_time) {
            runtime::revert(Error::InvalidTime);
        }
    }

    fn _assert_auction_time(&self) {
        let time = Time::from(runtime::get_blocktime());
        let auction_start_time = get_auction_start_time();
        let auction_end_time = get_auction_end_time();
        if !(time.gt(&auction_start_time) && time.lt(&auction_end_time)) {
            runtime::revert(Error::InvalidTime);
        }
    }

    fn emit(&mut self, event: CasperIdoEvent) {
        event::emit(&event);
    }
}
