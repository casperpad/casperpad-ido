use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use casper_ido_contract::{
    enums::{Address, BiddingToken},
    structs::{Schedules, Time},
};
use casper_types::{
    account::AccountHash, runtime_args, ContractHash, PublicKey, RuntimeArgs, SecretKey, U256, U512,
};
use test_env::{utils::DeploySource, TestEnv};

use crate::{
    casper_ido_instance::CasperIdoInstance,
    erc20_instance::ERC20Instance,
    factory_contract_instance::{self, FactoryContractInstance},
};

const PRE_CREATE_ORDER_WASM: &str = "pre_create_order.wasm";

struct TestContext {
    casper_ido_instance: CasperIdoInstance,
    erc20_instance: ERC20Instance,
    factory_contract_instance: FactoryContractInstance,
}

fn deploy() -> (TestEnv, TestContext, AccountHash) {
    let env = TestEnv::new();
    let owner = env.next_user();
    let ali = env.next_user();

    let erc20_instance = ERC20Instance::new(
        &env,
        "Test_Token",
        owner,
        "ACME",
        18,
        U256::from(5000u32).checked_mul(U256::exp10(18)).unwrap(),
    );

    let factory_contract_instance = FactoryContractInstance::new(
        &env,
        "ido_factory",
        owner,
        ali.to_formatted_string(),
        U256::exp10(4),
    );

    let info =
        "{\n  \"name\":\"The Swappery\",\n  \"info\":\"The Coolest DEX on Casper Network\"\n}";
    let auction_start_time = Time::from(1653728791085u64);
    let auction_end_time = Time::from(1653728791085u64);
    let launch_time = Time::from(1653728791085u64);

    let auction_token = Some(erc20_instance.contract_hash().to_formatted_string());
    let auction_token_price = U256::zero();
    let auction_token_capacity = U256::from(5000u32).checked_mul(U256::exp10(18)).unwrap();

    let schedules: Schedules = Schedules::new();
    let bidding_token: BiddingToken = BiddingToken::Native { price: None };
    let casper_ido_instance = CasperIdoInstance::new(
        &env,
        factory_contract_instance
            .contract_hash()
            .to_formatted_string(),
        "casper_ido",
        owner,
        info,
        auction_start_time,
        auction_end_time,
        launch_time,
        auction_token,
        auction_token_price,
        auction_token_capacity,
        bidding_token,
        schedules,
    );

    let test_context = TestContext {
        erc20_instance,
        factory_contract_instance,
        casper_ido_instance,
    };
    (env, test_context, owner)
}

fn whitlisted_users() -> Vec<AccountHash> {
    let mut users = Vec::new();
    let env = TestEnv::new();
    users.push(env.next_user());
    users.push(env.next_user());
    users.push(env.next_user());
    users.push(env.next_user());
    users
}

fn get_proof() -> Vec<(String, u8)> {
    // proof for first user
    vec![
        (
            "7850a400fe6148d5c4f1de52d470b1dcb92a148bcdbb50b21d8350ec07a72d4a".to_string(),
            1u8,
        ),
        (
            "3375b9932eca058ddf0b16ce35ea7c8f14ab672e8eb95259ada456b0aefeb8a9".to_string(),
            1u8,
        ),
    ]
}

#[test]
fn test_factory_contract_deploy() {
    let env = TestEnv::new();
    let owner = env.next_user();
    let ali = env.next_user();
    let factory_contract_instance = FactoryContractInstance::new(
        &env,
        "ido_factory_contract",
        owner,
        ali.to_formatted_string(),
        U256::exp10(4),
    );
    factory_contract_instance.add_auction(
        owner,
        factory_contract_instance
            .contract_hash()
            .to_formatted_string(),
    );
}

#[test]
fn test_deploy() {
    let _ = deploy();
}

#[test]
fn list_whitelisted_users() {
    let users = whitlisted_users();
    let result: Vec<String> = users.iter().map(|user| user.to_string()).collect();
    println!("{:?}", result);
    // assert!(false);
}

#[ignore]
#[test]
fn should_set_treasury_wallet() {
    let (env, test_context, owner) = deploy();
    let factory_contract = test_context.factory_contract_instance;
    let treasury_wallet = env.next_user();
    factory_contract.set_treasury_wallet(owner, treasury_wallet.to_formatted_string());
    let stored_treasury_wallet = factory_contract.get_treasury_wallet();
    assert_eq!(treasury_wallet, stored_treasury_wallet)
}
