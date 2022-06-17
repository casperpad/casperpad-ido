use alloc::{
    collections::BTreeMap,
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
        Orders, _get_merkle_root, _get_sold_amount, _get_total_participants, _get_treasury_wallet,
        _set_merkle_root, _set_sold_amount, _set_total_participants, _set_treasury_wallet,
        get_auction_end_time, get_auction_start_time, get_auction_token,
        get_auction_token_capacity, get_auction_token_price, get_creator, get_pay_token,
        get_schedules, set_auction_end_time, set_auction_start_time, set_auction_token,
        set_auction_token_capacity, set_auction_token_price, set_pay_token,
    },
    enums::Address,
    event::{self, CasperIdoEvent},
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
        pay_token: Option<ContractHash>,
        schedules: Schedules,
        treasury_wallet: AccountHash,
    ) {
        set_info(info);
        set_creator(runtime::get_caller());
        set_auction_start_time(auction_start_time);
        set_auction_end_time(auction_end_time);
        set_launch_time(launch_time);
        if auction_token.is_some() {
            set_auction_token(auction_token.unwrap());
        } else {
            set_auction_token(ContractHash::new([0u8; 32]));
        }
        set_auction_token_price(auction_token_price);
        set_auction_token_capacity(auction_token_capacity);
        set_pay_token(pay_token);
        set_schedules(schedules);
        set_factory_contract(factory_contract);
        _set_merkle_root("".to_string());
        _set_total_participants(0);
        _set_sold_amount(U256::from(0));
        _set_treasury_wallet(treasury_wallet);
        Orders::init();
        Claims::init();
    }

    fn contract_package_hash(&self) -> ContractPackageHash {
        let hash_addr = self.self_addr().into_hash().unwrap();
        ContractPackageHash::from(hash_addr)
    }

    fn set_auction_token(&mut self, auction_token: ContractHash) {
        self._assert_caller_is_creator();
        self._assert_before_auction_time();
        let auction_creator = get_creator();
        IERC20::new(auction_token).transfer_from(
            Address::from(auction_creator),
            Address::from(self.contract_package_hash()),
            self.auction_token_capacity(),
        );
        set_auction_token(auction_token);
    }

    /// Create order, caller must be whitelisted and can create in sale time.
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
        merkle_tree::verify(self.merkle_root(), leaf, proof);

        // Check current time is between sale time
        self._assert_auction_time();

        let pay_token = self.pay_token();

        // Check payment is right
        let order_amount = match pay_token {
            Some(_) => {
                IERC20::new(token).transfer_from(
                    Address::from(caller),
                    Address::from(self.treasury_wallet()),
                    amount,
                );
                amount
            }
            None => {
                runtime::revert(Error::InvalidPayToken);
            }
        };

        // Check order amount is less than tier
        let exist_order_amount = Orders::instance()
            .get(&Key::from(caller))
            .unwrap_or(U256::zero());

        let unchecked_new_order_amount = order_amount.checked_add(exist_order_amount).unwrap();

        if tier.lt(&unchecked_new_order_amount) {
            runtime::revert(Error::OutOfTier);
        }

        if exist_order_amount.eq(&U256::zero()) {
            self.increase_sold_amount_and_participants(order_amount);
        } else {
            self._increase_sold_amount(order_amount);
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
        merkle_tree::verify(self.merkle_root(), leaf, proof);

        // Check current time is between auction time
        self._assert_auction_time();

        let pay_token = self.pay_token();

        // Check payment is right
        let order_amount = match pay_token {
            Some(_) => {
                runtime::revert(Error::InvalidPayToken);
            }
            None => {
                let purse_balance = system::get_purse_balance(deposit_purse).unwrap_or_revert();

                system::transfer_from_purse_to_account(
                    deposit_purse,
                    self.treasury_wallet(),
                    purse_balance,
                    None,
                )
                .unwrap_or_revert();
                u512_to_u256(&purse_balance).unwrap_or_revert()
            }
        };

        // Check order amount is less than tier
        let exist_order_amount = Orders::instance()
            .get(&Key::from(caller))
            .unwrap_or(U256::zero());

        let unchecked_new_order_amount = order_amount.checked_add(exist_order_amount).unwrap();

        if tier.lt(&unchecked_new_order_amount) {
            runtime::revert(Error::OutOfTier);
        }
        if exist_order_amount.eq(&U256::zero()) {
            self.increase_sold_amount_and_participants(order_amount);
        } else {
            self._increase_sold_amount(order_amount);
        }
        Orders::instance().set(&Key::from(caller), unchecked_new_order_amount);
    }

    /// Whitelisted user can claim after schedule time
    fn claim(&mut self, caller: AccountHash, schedule_time: Time) {
        // Can claim after schedule
        let current_block_time = runtime::get_blocktime();
        if !schedule_time.lt(&u64::from(current_block_time)) {
            runtime::revert(Error::InvalidTime);
        }

        let order_amount = Orders::instance()
            .get(&Key::from(caller))
            .unwrap_or_revert_with(Error::NotExistOrder);

        if Claims::instance()
            .get(&Key::from(caller), schedule_time)
            .is_some()
        {
            runtime::revert(Error::AlreadyClaimed);
        }

        let auction_token_instance = IERC20::new(self.auction_token());
        let transfer_amount = {
            let schedule_percent = *get_schedules()
                .get(&schedule_time)
                .unwrap_or_revert_with(Error::InvalidSchedule);
            let auction_token_decimals = auction_token_instance.decimals();
            let auction_token_price = get_auction_token_price();
            order_amount
                .checked_mul(schedule_percent)
                .unwrap_or_revert()
                .checked_div(U256::exp10(4))
                .unwrap_or_revert()
                .checked_mul(U256::exp10(auction_token_decimals.into()))
                .unwrap_or_revert()
                .checked_div(auction_token_price)
                .unwrap_or_revert()
        };
        auction_token_instance.transfer(Address::from(caller), transfer_amount);
        Claims::instance().set(&Key::from(caller), schedule_time, true);
    }

    /// Set merkle_root , only admin call
    fn set_merkle_root(&mut self, merkle_root: String) {
        self._assert_caller_is_creator();
        _set_merkle_root(merkle_root);
    }

    fn add_orders(&mut self, orders: BTreeMap<String, U256>) {
        self._assert_caller_is_creator();

        orders.iter().enumerate().for_each(|order| {
            let user_order = order.1;
            let account = AccountHash::from_formatted_str(user_order.0).unwrap();
            let order_amount = *user_order.1;
            let exist_order_amount = Orders::instance()
                .get(&Key::from(account))
                .unwrap_or(U256::zero());
            let unchecked_new_order_amount = exist_order_amount.checked_add(order_amount).unwrap();

            Orders::instance().set(&Key::from(account), unchecked_new_order_amount);
            set_key("result", *user_order.1);
        });
    }

    fn change_time_schedules(
        &mut self,
        auction_start_time: Time,
        auction_end_time: Time,
        launch_time: Time,
        schedules: Schedules,
    ) {
        self._assert_before_auction_time();
        self._assert_caller_is_creator();
        set_auction_start_time(auction_start_time);
        set_auction_end_time(auction_end_time);
        set_launch_time(launch_time);
        set_schedules(schedules);
    }

    fn set_treasury_wallet(&mut self, treasury_wallet: AccountHash) {
        self._assert_caller_is_creator();
        _set_treasury_wallet(treasury_wallet);
    }

    fn treasury_wallet(&self) -> AccountHash {
        _get_treasury_wallet()
    }

    fn pay_token(&self) -> Option<ContractHash> {
        get_pay_token()
    }

    fn creator(&self) -> AccountHash {
        get_creator()
    }

    fn auction_token(&self) -> ContractHash {
        get_auction_token()
    }

    fn auction_token_price(&self) -> U256 {
        get_auction_token_price()
    }

    fn merkle_root(&self) -> String {
        _get_merkle_root()
    }

    fn increase_sold_amount_and_participants(&self, amount: U256) {
        self._increase_sold_amount(amount);
        self._increase_total_participants();
    }

    fn _increase_sold_amount(&self, amount: U256) {
        self.set_sold_amount(self.sold_amount().checked_add(amount).unwrap());
    }

    fn set_sold_amount(&self, amount: U256) {
        _set_sold_amount(amount);
    }

    fn sold_amount(&self) -> U256 {
        _get_sold_amount()
    }

    fn _increase_total_participants(&self) {
        self.set_total_participants(self.total_participants() + 1);
    }

    fn set_total_participants(&self, total_participants: u64) {
        _set_total_participants(total_participants);
    }

    fn total_participants(&self) -> u64 {
        _get_total_participants()
    }

    fn auction_token_capacity(&self) -> U256 {
        get_auction_token_capacity()
    }

    fn _assert_caller_is_creator(&self) {
        let auction_creator = get_creator();
        if auction_creator != runtime::get_caller() {
            runtime::revert(ApiError::PermissionDenied);
        }
    }

    fn _assert_before_auction_time(&self) {
        let time = Time::from(runtime::get_blocktime());
        let auction_start_time = get_auction_start_time();
        if time.gt(&auction_start_time) {
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
