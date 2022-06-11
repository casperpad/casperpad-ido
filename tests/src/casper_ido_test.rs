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
        owner.to_formatted_string(),
        U256::exp10(4),
    );

    let info =
        "{\n  \"name\":\"The Swappery\",\n  \"info\":\"The Coolest DEX on Casper Network\"\n}";
    let since_the_epoch: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let auction_start_time = Time::from(since_the_epoch);
    let auction_end_time = Time::from(since_the_epoch + 500000);
    let launch_time = Time::from(since_the_epoch + 1000000);

    let auction_token = Some(erc20_instance.contract_hash().to_formatted_string());
    let auction_token_price = U256::zero();
    let auction_token_capacity = U256::from(5000u32).checked_mul(U256::exp10(18)).unwrap();

    let mut schedules: Schedules = Schedules::new();
    schedules.insert(since_the_epoch + 666666, U256::from(1250));
    schedules.insert(since_the_epoch + 777777, U256::from(8750));
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
    let mut accounts = Vec::new();
    for i in 0..10u8 {
        let secret_key: SecretKey = SecretKey::ed25519_from_bytes([i; 32]).unwrap();
        let public_key: PublicKey = (&secret_key).into();
        let account_hash = AccountHash::from(&public_key);
        println!("{:?}", account_hash.to_string());
        accounts.push(account_hash);
    }
    accounts
}

fn get_proof() -> Vec<(String, u8)> {
    // proof for seconde env user
    // left :0 , right:1
    vec![
        (
            "8cb9c5592ec280a04113339fd485f4c9624d54a57f5bfd8a624c48b87cf3a4d0".to_string(),
            1u8,
        ),
        (
            "aadf43a695e899258595d7ca69de2369369c7add74a07e728121ef5b5971fb89".to_string(),
            0u8,
        ),
        (
            "6f7fbab2e9ca755cc58c3321a7fad3120fb3ba04aad9f7df6cba3d19536f7d6a".to_string(),
            1u8,
        ),
        (
            "85ec4e9e4eb79e2fa6f730ad6a311c2dae85f798054f1d4018a92ec7f511d50f".to_string(),
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
fn should_create_order() {
    let (env, test_context, owner) = deploy();
    let ido_contract = test_context.casper_ido_instance;

    let erc20 = test_context.erc20_instance;

    erc20.approve(
        owner,
        Address::from(ido_contract.contract_package_hash()),
        U256::from(5000u32).checked_mul(U256::exp10(18)).unwrap(),
    );

    ido_contract.set_auction_token(
        owner,
        erc20.contract_hash().to_formatted_string(),
        SystemTime::now()
            .checked_sub(Duration::from_secs(50000))
            .unwrap(),
    );

    ido_contract.set_merkle_root(
        owner,
        "32f7f9803d8e88954435659db24d6fdaa94ba46165fa1ce076b03f232273b3a5".to_string(),
    );

    ido_contract.set_cspr_price(
        owner,
        U256::from(1).checked_mul(U256::exp10(18 - 2)).unwrap(),
        SystemTime::now()
            .checked_sub(Duration::from_secs(20000))
            .unwrap(),
    );

    env.next_user();
    let user = env.next_user();
    let tier = U256::from(2u8).checked_mul(U256::exp10(18)).unwrap();
    let amount = U512::from(50u8).checked_mul(U512::exp10(9)).unwrap();

    let session_code = PathBuf::from(PRE_CREATE_ORDER_WASM);
    env.run_with_time(
        user,
        DeploySource::Code(session_code),
        runtime_args! {
            "ido_contract_hash" => ido_contract.contract_hash().to_formatted_string(),
            "tier" => tier,
            "proof" => get_proof(),
            "amount" => amount
        },
        SystemTime::now()
            .checked_add(Duration::from_secs(20000))
            .unwrap(),
    );

    let mut auction_schedules = ido_contract.schedules();

    println!("{:?}", auction_schedules);
    ido_contract.claim(
        user,
        *auction_schedules.first_entry().unwrap().key(),
        SystemTime::now()
            .checked_add(Duration::from_secs(7666660))
            .unwrap(),
    );
    let result: U256 = ido_contract.result();
    let balance = erc20.balance_of(Address::Account(user));
    println!("{:?},{:?}", result, balance);
    assert!(false);
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
