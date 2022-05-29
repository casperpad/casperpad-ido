use casper_ido_contract::{
    enums::BiddingToken,
    structs::{Schedules, Tiers, Time},
};
use casper_types::{account::AccountHash, ContractHash, U256};
use test_env::TestEnv;

use crate::{casper_ido_instance::CasperIdoInstance, erc20_instance::ERC20Instance};

struct TestContext {
    casper_ido_instance: CasperIdoInstance,
    erc20_instance: ERC20Instance,
}

fn deploy() -> (TestEnv, TestContext, AccountHash) {
    let env = TestEnv::new();
    let owner = env.next_user();
    let casper_ido_instance = CasperIdoInstance::new(&env, "NAME", owner);
    let erc20_instance = ERC20Instance::new(&env, "NAME", owner);
    let test_context = TestContext {
        casper_ido_instance,
        erc20_instance,
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

#[test]
fn test_create_auction() {
    let (env, test_context, owner) = deploy();
    let casper_ido_instance = test_context.casper_ido_instance;
    let id = "swappery";
    let info =
        "{\n  \"name\":\"The Swappery\",\n  \"info\":\"The Coolest DEX on Casper Network\"\n}";
    let auction_start_time = Time::from(1653728791085u64);
    let auction_end_time = Time::from(1653728791085u64);
    let project_open_time = Time::from(1653728791085u64);
    let auction_token = ContractHash::new([1u8; 32]).to_formatted_string();
    let auction_token_price = U256::zero();
    let auction_token_capacity = U256::zero();
    let bidding_token: BiddingToken = BiddingToken::Native { price: None };
    let fee_numerator: u8 = 15u8;
    let schedules: Schedules = Schedules::new();
    let merkle_root: Option<String> =
        Some("4946762002a6613343a97a66739a836f2c3bca1fd7004824f43a5e9b187e51f0".to_string());
    let mut tiers: Tiers = Tiers::new();
    tiers.insert(owner, U256::from(2000));
    tiers.insert(env.next_user(), U256::zero());
    tiers.insert(env.next_user(), U256::one());
    tiers.insert(env.next_user(), U256::from(4));
    tiers.insert(env.next_user(), U256::from(5));
    casper_ido_instance.create_auction(
        owner,
        id,
        info,
        auction_start_time,
        auction_end_time,
        project_open_time,
        &auction_token,
        auction_token_price,
        auction_token_capacity,
        bidding_token,
        fee_numerator,
        schedules,
        merkle_root,
        tiers,
    );
}

#[ignore]
#[test]
fn should_create_order() {
    let (env, test_context, owner) = deploy();
    let proof = get_proof();
    let amount = U256::from(1000);
    let id = "swappery";
    let info =
        "{\n  \"name\":\"The Swappery\",\n  \"info\":\"The Coolest DEX on Casper Network\"\n}";
    let auction_start_time = Time::from(1653728791085u64);
    let auction_end_time = Time::from(1653728791085u64);
    let project_open_time = Time::from(1653728791085u64);
    let auction_token = ContractHash::new([1u8; 32]).to_formatted_string();
    let auction_token_price = U256::zero();
    let auction_token_capacity = U256::zero();
    let bidding_token: BiddingToken = BiddingToken::Native { price: None };
    let fee_numerator: u8 = 15u8;
    let schedules: Schedules = Schedules::new();
    let merkle_root: Option<String> =
        Some("4946762002a6613343a97a66739a836f2c3bca1fd7004824f43a5e9b187e51f0".to_string());
    let mut tiers: Tiers = Tiers::new();

    tiers.insert(owner, U256::from(5000));
    tiers.insert(env.next_user(), U256::zero());
    tiers.insert(env.next_user(), U256::one());
    tiers.insert(env.next_user(), U256::from(4));
    tiers.insert(env.next_user(), U256::from(5));
    let casper_ido_instance = test_context.casper_ido_instance;
    casper_ido_instance.create_auction(
        owner,
        id,
        info,
        auction_start_time,
        auction_end_time,
        project_open_time,
        &auction_token,
        auction_token_price,
        auction_token_capacity,
        bidding_token,
        fee_numerator,
        schedules,
        merkle_root,
        tiers,
    );
    casper_ido_instance.create_order(owner, id, proof, amount);

    // let auction = casper_ido_instance.get_auction(id).unwrap();
    // println!("{:?}", auction);
    let time = casper_ido_instance.get_install_time();
    println!("{:?}", time);
    assert!(false);
}
