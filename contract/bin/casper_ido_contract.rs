#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{
    boxed::Box,
    collections::BTreeSet,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_ido_contract::{
    enums::{Address, BiddingToken},
    libs::address_utils,
    structs::{Auction, Claims, Orders, Schedules, Tiers, Time},
    CasperIdo, IERC20,
};

use casper_types::{
    account::AccountHash, runtime_args, CLType, CLTyped, ContractHash, ContractPackageHash,
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Group, Key, Parameter, RuntimeArgs,
    URef, U256,
};
use contract_utils::{AdminControl, ContractContext, OnChainContractStorage, ReentrancyGuard};

#[derive(Default)]
struct CasperIdoContract(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for CasperIdoContract {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl CasperIdo<OnChainContractStorage> for CasperIdoContract {}
impl AdminControl<OnChainContractStorage> for CasperIdoContract {}
impl ReentrancyGuard<OnChainContractStorage> for CasperIdoContract {}

impl CasperIdoContract {
    fn constructor(&mut self, default_treasury_wallet: Address) {
        CasperIdo::init(self, default_treasury_wallet);
        AdminControl::init(self);
        ReentrancyGuard::init(self);
    }
}

#[no_mangle]
pub extern "C" fn constructor() {
    let default_treasury_wallet: Address = runtime::get_named_arg("default_treasury_wallet");
    CasperIdoContract::default().constructor(default_treasury_wallet);
    let default_admin = runtime::get_caller();
    CasperIdoContract::default().add_admin_without_checked(Key::from(default_admin));
}

#[no_mangle]
pub extern "C" fn create_auction() {
    CasperIdoContract::default().assert_caller_is_admin();

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

    let auction_token_instance = IERC20::new(auction_token);

    // // Send Fee to treasury wallet
    let fee_denominator = CasperIdoContract::default().get_fee_denominator();
    let fee_amount = auction_token_capacity
        .checked_mul(U256::from(fee_numerator))
        .unwrap_or_revert()
        .checked_div(fee_denominator)
        .unwrap_or_revert();
    let treasury_wallet = CasperIdoContract::default().get_treasury_wallet();
    auction_token_instance.transfer_from(Address::from(creator), treasury_wallet, fee_amount);

    // Set auction_token_capacity - fee_amount to new auction_token_capacity
    let contract_package_hash = address_utils::get_caller_address().unwrap_or_revert();
    let auction_token_capacity = auction_token_capacity
        .checked_sub(fee_amount)
        .unwrap_or_revert();
    auction_token_instance.transfer_from(
        Address::from(creator),
        contract_package_hash,
        auction_token_capacity,
    );

    let auction = Auction {
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
    };
    CasperIdoContract::default().create_auction(id, auction);
}

#[no_mangle]
pub extern "C" fn create_order() {
    let caller = runtime::get_caller();
    let auction_id: String = runtime::get_named_arg("auction_id");
    let proof: Vec<(String, u8)> = runtime::get_named_arg("proof");
    let amount: U256 = runtime::get_named_arg("amount");

    CasperIdoContract::default().create_order(caller, auction_id, proof, amount);
}

#[no_mangle]
pub extern "C" fn cancel_order() {
    let caller = runtime::get_caller();
    let auction_id: String = runtime::get_named_arg("auction_id");
    CasperIdoContract::default().cancel_order(caller, auction_id)
}

#[no_mangle]
pub extern "C" fn claim() {
    let caller = runtime::get_caller();
    let auction_id: String = runtime::get_named_arg("auction_id");
    let schedule_time: Time = runtime::get_named_arg("schedule_time");

    CasperIdoContract::default().set_reentrancy();
    CasperIdoContract::default().claim(caller, auction_id, schedule_time);
    CasperIdoContract::default().clear_reentrancy();
}

#[no_mangle]
pub extern "C" fn set_cspr_price() {
    let auction_id: String = runtime::get_named_arg("auction_id");
    let price: U256 = runtime::get_named_arg("price");
    CasperIdoContract::default().set_cspr_price(auction_id, price);
}

#[no_mangle]
pub extern "C" fn set_tiers() {
    let auction_id: String = runtime::get_named_arg("auction_id");
    let mut tiers: Tiers = runtime::get_named_arg("tiers");
    CasperIdoContract::default().set_tiers(auction_id, &mut tiers);
}

#[no_mangle]
pub extern "C" fn set_merkle_root() {
    let auction_id: String = runtime::get_named_arg("auction_id");
    let merkle_root: String = runtime::get_named_arg("merkle_root");
    CasperIdoContract::default().set_merkle_root(auction_id, merkle_root);
}

#[no_mangle]
pub extern "C" fn set_fee_denominator() {
    let fee_denominator: U256 = runtime::get_named_arg("fee_denominator");
    CasperIdoContract::default().set_fee_denominator(fee_denominator);
}

#[no_mangle]
pub extern "C" fn set_treasury_wallet() {
    CasperIdoContract::default().assert_caller_is_admin();
    let treasury_wallet: Address = runtime::get_named_arg("treasury_wallet");
    CasperIdoContract::default().set_treasury_wallet(treasury_wallet);
}

#[no_mangle]
pub extern "C" fn add_admin() {
    let admin: AccountHash = {
        let admin_string: String = runtime::get_named_arg("admin");
        AccountHash::from_formatted_str(&admin_string).unwrap()
    };
    CasperIdoContract::default().add_admin(Key::from(admin));
}

#[no_mangle]
pub extern "C" fn remove_admin() {
    let admin: AccountHash = {
        let admin_string: String = runtime::get_named_arg("admin");
        AccountHash::from_formatted_str(&admin_string).unwrap()
    };
    CasperIdoContract::default().disable_admin(Key::from(admin));
}

#[no_mangle]
pub extern "C" fn call() {
    let contract_name: String = runtime::get_named_arg("contract_name");
    let default_treasury_wallet: Address = runtime::get_named_arg("default_treasury_wallet");

    storage::new_uref(default_treasury_wallet); // TODO Check why not working on init
    let (contract_hash, _) = storage::new_contract(
        get_entry_points(),
        None,
        Some(String::from(format!(
            "{}_contract_package_hash",
            contract_name
        ))),
        None,
    );

    let package_hash: ContractPackageHash = ContractPackageHash::new(
        runtime::get_key(&format!("{}_contract_package_hash", contract_name))
            .unwrap_or_revert()
            .into_hash()
            .unwrap_or_revert(),
    );
    let constructor_access: URef =
        storage::create_contract_user_group(package_hash, "constructor", 1, Default::default())
            .unwrap_or_revert()
            .pop()
            .unwrap_or_revert();

    let constructor_args = runtime_args! {
        "default_treasury_wallet" => default_treasury_wallet
    };
    let _: () = runtime::call_contract(contract_hash, "constructor", constructor_args);

    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(package_hash, "constructor", urefs)
        .unwrap_or_revert();

    runtime::put_key(
        &format!("{}_contract_hash", contract_name),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{}_contract_hash_wrapped", contract_name),
        storage::new_uref(contract_hash).into(),
    );
}

fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![Parameter::new(
            "default_treasury_wallet".to_string(),
            Address::cl_type(),
        )],
        CLType::Unit,
        EntryPointAccess::Groups(vec![Group::new("constructor")]),
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "create_auction",
        vec![
            Parameter::new("id".to_string(), CLType::String),
            Parameter::new("info".to_string(), CLType::String),
            Parameter::new("auction_start_time".to_string(), CLType::U64),
            Parameter::new("auction_end_time".to_string(), CLType::U64),
            Parameter::new("project_open_time".to_string(), CLType::U64),
            Parameter::new("auction_token".to_string(), CLType::String),
            Parameter::new("auction_token_price".to_string(), CLType::U256),
            Parameter::new("bidding_token".to_string(), BiddingToken::cl_type()),
            Parameter::new("fee_numerator".to_string(), CLType::U8),
            Parameter::new("schedules".to_string(), Schedules::cl_type()),
            Parameter::new(
                "merkle_root".to_string(),
                CLType::Option(Box::new(CLType::String)),
            ),
            Parameter::new("tiers".to_string(), Tiers::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "create_order",
        vec![
            Parameter::new("auction_id".to_string(), CLType::String),
            Parameter::new(
                "proof".to_string(),
                CLType::List(Box::new(CLType::Tuple2([
                    Box::new(CLType::String),
                    Box::new(CLType::U8),
                ]))),
            ),
            Parameter::new("amount".to_string(), CLType::U256),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "cancel_order",
        vec![Parameter::new("auction_id".to_string(), CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "claim",
        vec![
            Parameter::new("auction_id".to_string(), CLType::String),
            Parameter::new("schedule_time".to_string(), CLType::U64),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_cspr_price",
        vec![
            Parameter::new("auction_id".to_string(), CLType::String),
            Parameter::new("price".to_string(), CLType::U256),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_tiers",
        vec![
            Parameter::new("auction_id".to_string(), CLType::String),
            Parameter::new("tiers".to_string(), Tiers::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_merkle_root",
        vec![
            Parameter::new("auction_id".to_string(), CLType::String),
            Parameter::new("merkle_root".to_string(), CLType::String),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_fee_denominator",
        vec![Parameter::new("fee_denominator".to_string(), CLType::U256)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "set_treasury_wallet",
        vec![Parameter::new(
            "treasury_wallet".to_string(),
            Address::cl_type(),
        )],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "add_admin",
        vec![Parameter::new("admin".to_string(), CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        "remove_admin",
        vec![Parameter::new("admin".to_string(), CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}
