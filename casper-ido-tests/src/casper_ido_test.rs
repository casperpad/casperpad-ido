use casper_ido::{
    enums::BiddingToken,
    structs::{Schedules, Tiers, Time},
};
use casper_types::{account::AccountHash, ContractHash, U256};
use test_env::TestEnv;

use crate::casper_ido_instance::CasperIdoInstance;

fn deploy() -> (TestEnv, CasperIdoInstance, AccountHash) {
    let env = TestEnv::new();
    let owner = env.next_user();
    let casper_ido_instance = CasperIdoInstance::new(&env, "NAME", owner);
    (env, casper_ido_instance, owner)
}

#[test]
fn test_deploy() {
    let _ = deploy();
}

#[test]
fn test_create_auction() {
    let (env, casper_ido_instance, owner) = deploy();
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
    let merkle_root: Option<String> = None;
    let mut tiers: Tiers = Tiers::new();

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

    let auction = casper_ido_instance.get_auction(id).unwrap();
    println!("{:?}", auction);
    assert!(false);
}
