use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use casper_ido_contract::{
    enums::{Address, BiddingToken},
    structs::{Schedules, Tiers, Time},
};
use casper_types::{account::AccountHash, runtime_args, ContractHash, RuntimeArgs, U256, U512};
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

    let _ = factory_contract_instance.get_treasury_wallet();

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

    // After remove below comment Error occured
    // let _ = factory_contract_instance.get_treasury_wallet();

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
fn test_deploy2() {
    let env = TestEnv::new();
    let owner = env.next_user();
    let ali = env.next_user();
    let factory_contract_instance = FactoryContractInstance::new(
        &env,
        "ido_factory",
        owner,
        ali.to_formatted_string(),
        U256::exp10(4),
    );
    factory_contract_instance
        .add_auction(owner, ContractHash::new([1u8; 32]).to_formatted_string());
    let result = factory_contract_instance.get_treasury_wallet();
    println!("{:?}", result);
    // assert!(false);
}

#[test]
fn test_deploy3() {
    let (env, test_context, owner) = deploy();
    println!("{:?}", env.get_account(owner).unwrap().named_keys());
    assert!(false);
    // let ido = test_context.casper_ido_instance;

    // ido.set_cspr_price(owner, U256::one());
    // let result: U256 = ido.result();

    // let treasury_wallet = env.next_user().to_formatted_string();
    // test_context
    //     .factory_contract_instance
    //     .set_treasury_wallet(owner, treasury_wallet);
    // test_context
    //     .factory_contract_instance
    //     .set_fee_denominator(owner, U256::zero());

    // let result: ContractHash = test_context.casper_ido_instance.result();
    // let result2: ContractHash = test_context.casper_ido_instance.result2();
}

#[test]
fn list_whitelisted_users() {
    let users = whitlisted_users();
    let result: Vec<String> = users.iter().map(|user| user.to_string()).collect();
    println!("{:?}", result);
    // assert!(false);
}

// #[test]
// fn should_create_order() {
//     let (env, test_context, owner) = deploy();
//     let casper_ido_instance = test_context.casper_ido_instance;
//     let erc20_instance = test_context.erc20_instance;

//     let id = "swappery";
//     let info =
//         "{\n  \"name\":\"The Swappery\",\n  \"info\":\"The Coolest DEX on Casper Network\"\n}";

//     let since_the_epoch: u64 = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .expect("Time went backwards")
//         .as_secs();

//     let auction_start_time = Time::from(since_the_epoch.checked_add(5000000).unwrap());
//     let auction_end_time = Time::from(since_the_epoch.checked_add(10000000).unwrap());
//     let project_open_time = Time::from(1653728791085u64);

//     let auction_token = erc20_instance.contract_hash().to_formatted_string();
//     let auction_token_price = U256::zero();
//     let auction_token_capacity = U256::from(5000u32).checked_mul(U256::exp10(18)).unwrap();
//     erc20_instance.approve(
//         owner,
//         Address::from(casper_ido_instance.contract_package_hash()),
//         auction_token_capacity,
//     );
//     let allowance = erc20_instance
//         .allowance(
//             Address::from(owner),
//             Address::from(casper_ido_instance.contract_package_hash()),
//         )
//         .unwrap();
//     // Should approve amount equal
//     assert_eq!(allowance, auction_token_capacity);
//     let bidding_token: BiddingToken = BiddingToken::Native { price: None };
//     let fee_numerator: u8 = 255u8;
//     let schedules: Schedules = Schedules::new();
//     let merkle_root: Option<String> =
//         Some("4946762002a6613343a97a66739a836f2c3bca1fd7004824f43a5e9b187e51f0".to_string());
//     let mut tiers: Tiers = Tiers::new();
//     tiers.insert(
//         owner,
//         U256::from(1000).checked_mul(U256::exp10(18)).unwrap(),
//     );
//     tiers.insert(env.next_user(), U256::zero());
//     tiers.insert(env.next_user(), U256::one());
//     tiers.insert(env.next_user(), U256::from(4));
//     let bob = env.next_user();
//     tiers.insert(bob, U256::from(10).checked_mul(U256::exp10(18)).unwrap());

//     casper_ido_instance.create_auction(
//         owner,
//         id,
//         info,
//         auction_start_time,
//         auction_end_time,
//         project_open_time,
//         &auction_token,
//         auction_token_price,
//         auction_token_capacity,
//         bidding_token,
//         fee_numerator,
//         schedules,
//         merkle_root,
//         tiers,
//     );

//     casper_ido_instance.set_cspr_price(
//         owner,
//         id,
//         U256::from(1).checked_mul(U256::exp10(18 - 2)).unwrap(),
//     );

//     let proof = get_proof();
//     let amount = U512::from(1_000_u64).checked_mul(U512::exp10(9)).unwrap();

//     let session_code = PathBuf::from(PRE_CREATE_ORDER_WASM);
//     env.run_with_time(
//         owner,
//         DeploySource::Code(session_code),
//         runtime_args! {
//             "ido_contract_hash" => casper_ido_instance.contract_hash().to_formatted_string(),
//             "auction_id" => id,
//             "proof" => proof.clone(),
//             "token" => Option::<String>::None,
//             "amount" => amount
//         },
//         SystemTime::now()
//             .checked_add(Duration::from_secs(7000000))
//             .unwrap(),
//     );

//     // let amount = U512::from(70000).checked_mul(U512::exp10(9)).unwrap();

//     // let session_code = PathBuf::from(PRE_CREATE_ORDER_WASM);
//     // env.run_with_time(
//     //     bob,
//     //     DeploySource::Code(session_code),
//     //     runtime_args! {
//     //         "ido_contract_hash" => casper_ido_instance.contract_hash().to_formatted_string(),
//     //         "auction_id" => id,
//     //         "proof" => proof,
//     //         "token" => Option::<String>::None,
//     //         "amount" => amount
//     //     },
//     //     SystemTime::now()
//     //         .checked_add(Duration::from_secs(8000000))
//     //         .unwrap(),
//     // );

//     let auction = casper_ido_instance.get_auction(id);

//     println!("{:?}", auction);
//     // assert!(false);
// }

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
