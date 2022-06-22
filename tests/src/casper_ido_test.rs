use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use alloc::collections::BTreeMap;

use casper_ido_contract::{
    enums::Address,
    structs::{Schedules, Time},
};
use casper_types::{
    account::AccountHash, runtime_args, PublicKey, RuntimeArgs, SecretKey, U256, U512,
};
use test_env::{utils::DeploySource, TestEnv};

use crate::{casper_ido_instance::CasperIdoInstance, erc20_instance::ERC20Instance};

const PRE_CREATE_ORDER_WASM: &str = "pre_create_order.wasm";

struct TestContext {
    casper_ido_instance: CasperIdoInstance,
    erc20_instance: ERC20Instance,
}

fn deploy() -> (TestEnv, TestContext, AccountHash) {
    let env = TestEnv::new();
    let owner = env.next_user();

    let erc20_instance = ERC20Instance::new(
        &env,
        "Test_Token",
        owner,
        "ACME",
        9,
        U256::from(5000u32).checked_mul(U256::exp10(9)).unwrap(),
    );

    let since_the_epoch: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let auction_start_time = Time::from(since_the_epoch);
    let auction_end_time = Time::from(since_the_epoch + 500000);
    let auction_token_price = U256::from(2).checked_mul(U256::exp10(9)).unwrap();
    let auction_token_capacity = U256::from(5000u32).checked_mul(U256::exp10(9)).unwrap();

    let mut schedules: Schedules = Schedules::new();
    schedules.insert(since_the_epoch + 666666, U256::from(4000));
    schedules.insert(since_the_epoch + 777777, U256::from(6000));
    let pay_token: Option<String> = None;
    let treasury_wallet = AccountHash::new([3u8; 32]).to_formatted_string();
    let casper_ido_instance = CasperIdoInstance::new(
        &env,
        "casper_ido",
        owner,
        auction_start_time,
        auction_end_time,
        auction_token_price,
        auction_token_capacity,
        pay_token,
        schedules,
        treasury_wallet,
    );

    let test_context = TestContext {
        erc20_instance,
        casper_ido_instance,
    };
    (env, test_context, owner)
}

fn _whitlisted_users() -> Vec<AccountHash> {
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
fn test_deploy() {
    let _ = deploy();
}

#[test]
fn should_create_order_and_claim() {
    let (env, test_context, owner) = deploy();
    let ido_contract = test_context.casper_ido_instance;

    // Set Auction token
    let erc20 = test_context.erc20_instance;

    erc20.approve(
        owner,
        Address::from(ido_contract.contract_package_hash()),
        U256::from(5000u32).checked_mul(U256::exp10(9)).unwrap(),
    );

    ido_contract.set_auction_token(
        owner,
        erc20.contract_hash().to_formatted_string(),
        SystemTime::now()
            .checked_sub(Duration::from_secs(50000))
            .unwrap(),
    );

    // Set merkle root
    ido_contract.set_merkle_root(
        owner,
        "32f7f9803d8e88954435659db24d6fdaa94ba46165fa1ce076b03f232273b3a5".to_string(),
    );

    env.next_user();
    let user = env.next_user();
    let tier = U256::from(2u8).checked_mul(U256::exp10(18)).unwrap();
    let amount = U512::from(50u8).checked_mul(U512::exp10(9)).unwrap();

    let new_treasury_wallet = AccountHash::new([4u8; 32]);
    ido_contract.set_treasury_wallet(owner, new_treasury_wallet.to_formatted_string());

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

    ido_contract.claim(
        user,
        *auction_schedules.first_entry().unwrap().key(),
        SystemTime::now()
            .checked_add(Duration::from_secs(7666660))
            .unwrap(),
    );
    let treasury_wallet_balance = env.account_purse_balance(new_treasury_wallet);
    assert!(amount.eq(&treasury_wallet_balance));
    let _ = erc20.balance_of(Address::Account(user)).unwrap();
}

#[test]
fn should_create_order_and_claim_erc20() {
    let env = TestEnv::new();
    let owner = env.next_user();

    let erc20_instance = ERC20Instance::new(
        &env,
        "Test_Token",
        owner,
        "ACME",
        9,
        U256::from(5000u32).checked_mul(U256::exp10(9)).unwrap(),
    );

    let since_the_epoch: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let auction_start_time = Time::from(since_the_epoch);
    let auction_end_time = Time::from(since_the_epoch + 500000);
    let auction_token_price = U256::from(2).checked_mul(U256::exp10(9)).unwrap();
    let auction_token_capacity = U256::from(5000u32).checked_mul(U256::exp10(9)).unwrap();

    let mut schedules: Schedules = Schedules::new();
    schedules.insert(since_the_epoch + 666666, U256::from(4000));
    schedules.insert(since_the_epoch + 777777, U256::from(6000));

    let pay_token = ERC20Instance::new(
        &env,
        "USDT",
        owner,
        "USDT",
        9,
        U256::from(5000u32).checked_mul(U256::exp10(9)).unwrap(),
    );

    let pay_token_str: Option<String> = Some(pay_token.contract_hash().to_formatted_string());
    let treasury_wallet = AccountHash::new([3u8; 32]).to_formatted_string();
    let casper_ido_instance = CasperIdoInstance::new(
        &env,
        "casper_ido",
        owner,
        auction_start_time,
        auction_end_time,
        auction_token_price,
        auction_token_capacity,
        pay_token_str,
        schedules,
        treasury_wallet,
    );
    let ido_contract = casper_ido_instance;

    // Set Auction token
    let erc20 = erc20_instance;

    erc20.approve(
        owner,
        Address::from(ido_contract.contract_package_hash()),
        U256::from(5000u32).checked_mul(U256::exp10(9)).unwrap(),
    );

    ido_contract.set_auction_token(
        owner,
        erc20.contract_hash().to_formatted_string(),
        SystemTime::now()
            .checked_sub(Duration::from_secs(50000))
            .unwrap(),
    );

    // Set merkle root
    ido_contract.set_merkle_root(
        owner,
        "32f7f9803d8e88954435659db24d6fdaa94ba46165fa1ce076b03f232273b3a5".to_string(),
    );

    let new_treasury_wallet = AccountHash::new([4u8; 32]);
    ido_contract.set_treasury_wallet(owner, new_treasury_wallet.to_formatted_string());

    env.next_user();
    let user = env.next_user();
    let tier = U256::from(2u8).checked_mul(U256::exp10(18)).unwrap();
    let amount = U256::from(50u8).checked_mul(U256::exp10(9)).unwrap();

    pay_token.transfer(owner, Address::from(user), amount);

    pay_token.approve(
        user,
        Address::Contract(ido_contract.contract_package_hash()),
        amount,
    );

    ido_contract.create_order(
        user,
        tier,
        get_proof(),
        amount,
        SystemTime::now()
            .checked_add(Duration::from_secs(20000))
            .unwrap(),
    );

    let mut auction_schedules = ido_contract.schedules();

    ido_contract.claim(
        user,
        *auction_schedules.first_entry().unwrap().key(),
        SystemTime::now()
            .checked_add(Duration::from_secs(7666660))
            .unwrap(),
    );
    let treasury_wallet_balance = pay_token
        .balance_of(Address::from(new_treasury_wallet))
        .unwrap();
    assert!(amount.eq(&treasury_wallet_balance));
    let _ = erc20.balance_of(Address::Account(user)).unwrap();
}

#[test]
fn should_add_orders() {
    let (env, test_context, owner) = deploy();
    let ido_contract = test_context.casper_ido_instance;
    let mut orders: BTreeMap<String, U256> = BTreeMap::new();
    let ali = env.next_user();
    let bob = env.next_user();
    orders.insert(ali.to_formatted_string(), U256::one());
    orders.insert(bob.to_formatted_string(), U256::one());
    orders.insert(env.next_user().to_formatted_string(), U256::one());
    orders.insert(env.next_user().to_formatted_string(), U256::one());
    ido_contract.add_orders(owner, orders);
}
