use alloc::{collections::BTreeMap, string::String};
use casper_contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{account::AccountHash, ContractHash, ContractPackageHash, Key, URef, U256};
use contract_utils::{ContractContext, ContractStorage};

use crate::{
    data::{
        set_creator, set_schedules, Claims, Orders, _get_max_order_amount, _get_min_order_amount,
        _get_sold_amount, _get_total_participants, _get_treasury_wallet, _set_max_order_amount,
        _set_min_order_amount, _set_sold_amount, _set_total_participants, _set_treasury_wallet,
        get_auction_end_time, get_auction_start_time, get_auction_token,
        get_auction_token_capacity, get_auction_token_price, get_creator, get_pay_token,
        get_schedules, set_auction_end_time, set_auction_start_time, set_auction_token,
        set_auction_token_capacity, set_auction_token_price, set_pay_token,
    },
    enums::Address,
    event::{self, CasperIdoEvent},
    libs::conversion::u512_to_u256,
    structs::{Schedules, Time},
    Error, IERC20,
};

pub trait CasperIdo<Storage: ContractStorage>: ContractContext<Storage> {
    fn init(
        &mut self,
        auction_start_time: Time,
        auction_end_time: Time,
        auction_token_price: U256,
        auction_token_capacity: U256,
        pay_token: Option<ContractHash>,
        schedules: Schedules,
        treasury_wallet: AccountHash,
        min_order_amount: U256,
        max_order_amount: U256,
    ) {
        set_creator(runtime::get_caller());
        set_auction_start_time(auction_start_time);
        set_auction_end_time(auction_end_time);

        set_auction_token(ContractHash::new([0u8; 32]));
        set_auction_token_price(auction_token_price);
        set_auction_token_capacity(auction_token_capacity);
        set_pay_token(pay_token);
        set_schedules(schedules);
        _set_total_participants(0);
        _set_sold_amount(U256::from(0));
        _set_treasury_wallet(treasury_wallet);
        _set_min_order_amount(min_order_amount);
        _set_max_order_amount(max_order_amount);

        Orders::init();
        Claims::init();
    }

    fn contract_package_hash(&self) -> ContractPackageHash {
        let hash_addr = self.self_addr().into_hash().unwrap();
        ContractPackageHash::from(hash_addr)
    }

    fn set_auction_token(&mut self, auction_token: ContractHash, auction_token_capacity: U256) {
        self._asert_null_auction_token();
        set_auction_token_capacity(auction_token_capacity);
        let auction_creator = get_creator();
        IERC20::new(auction_token).transfer_from(
            Address::from(auction_creator),
            Address::from(self.contract_package_hash()),
            auction_token_capacity,
        );
        set_auction_token(auction_token);
    }

    /// Create order, caller must be whitelisted and can create in sale time.
    fn create_order(&mut self, caller: AccountHash, amount: U256) {
        // Check current time is between sale time
        self._assert_auction_time();

        let pay_token = self.pay_token();

        // Check payment is right
        let order_amount = match pay_token {
            Some(token) => {
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

        // Check order amount is less than max
        let exist_order_amount = Orders::instance()
            .get(&Key::from(caller))
            .unwrap_or(U256::zero());

        let unchecked_new_order_amount = order_amount.checked_add(exist_order_amount).unwrap();

        if unchecked_new_order_amount.lt(&self.min_order_amount())
            || unchecked_new_order_amount.gt(&self.max_order_amount())
        {
            runtime::revert(Error::InvalidOrderAmount);
        }

        if exist_order_amount.eq(&U256::zero()) {
            self.increase_sold_amount_and_participants(order_amount);
        } else {
            self._increase_sold_amount(order_amount);
        }
        Orders::instance().set(&Key::from(caller), unchecked_new_order_amount);
    }

    fn create_order_cspr(&mut self, caller: AccountHash, deposit_purse: URef) {
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

        if unchecked_new_order_amount.lt(&self.min_order_amount())
            || unchecked_new_order_amount.gt(&self.max_order_amount())
        {
            runtime::revert(Error::InvalidOrderAmount);
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

    fn add_orders(&mut self, orders: BTreeMap<String, U256>) {
        orders.iter().enumerate().for_each(|order| {
            let user_order = order.1;
            let account = AccountHash::from_formatted_str(user_order.0).unwrap();
            let order_amount = *user_order.1;
            let exist_order_amount = Orders::instance()
                .get(&Key::from(account))
                .unwrap_or(U256::zero());
            let unchecked_new_order_amount = exist_order_amount.checked_add(order_amount).unwrap();

            if exist_order_amount.eq(&U256::zero()) {
                self.increase_sold_amount_and_participants(order_amount);
            } else {
                self._increase_sold_amount(order_amount);
            }

            Orders::instance().set(&Key::from(account), unchecked_new_order_amount);
        });
    }

    fn change_auction_token_price(&mut self, price: U256) {
        self._assert_before_auction_time();
        set_auction_token_price(price);
    }

    fn change_time_schedules(
        &mut self,
        auction_start_time: Time,
        auction_end_time: Time,
        schedules: Schedules,
    ) {
        set_auction_start_time(auction_start_time);
        set_auction_end_time(auction_end_time);
        set_schedules(schedules);
    }

    fn set_min_order_amount(&mut self, amount: U256) {
        _set_min_order_amount(amount);
    }

    fn set_max_order_amount(&mut self, amount: U256) {
        _set_max_order_amount(amount);
    }

    fn min_order_amount(&self) -> U256 {
        _get_min_order_amount()
    }

    fn max_order_amount(&self) -> U256 {
        _get_max_order_amount()
    }

    fn set_treasury_wallet(&mut self, treasury_wallet: AccountHash) {
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

    fn _asert_null_auction_token(&self) {
        let auction_token = self.auction_token();
        if auction_token.ne(&ContractHash::new([0u8; 32])) {
            runtime::revert(Error::AlreadySettedToken);
        }
    }

    fn _assert_before_first_shedule_time(&self) {
        let time = Time::from(runtime::get_blocktime());
        let schedules = get_schedules();
        let first_time = schedules.keys().min().unwrap();

        if time.gt(&first_time) {
            runtime::revert(Error::InvalidTime);
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
