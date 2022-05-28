use casper_ido::{
    enums::BiddingToken,
    structs::{Auction, Schedules, Tiers, Time},
};
use casper_types::{account::AccountHash, runtime_args, RuntimeArgs, U256};
use test_env::{TestContract, TestEnv};

pub struct CasperIdoInstance(TestContract);

impl CasperIdoInstance {
    pub fn new(env: &TestEnv, contract_name: &str, sender: AccountHash) -> CasperIdoInstance {
        CasperIdoInstance(TestContract::new(
            env,
            "casper_ido_contract.wasm",
            contract_name,
            sender,
            runtime_args! {},
        ))
    }

    pub fn constructor(&self, sender: AccountHash, default_merkle_root: &str) {
        self.0.call_contract(
            sender,
            "constructor",
            runtime_args! {
            "default_merkle_root" => default_merkle_root,},
        );
    }

    pub fn create_auction(
        &self,
        sender: AccountHash,
        id: &str,
        info: &str,
        auction_start_time: Time,
        auction_end_time: Time,
        project_open_time: Time,
        auction_token: &str,
        auction_token_price: U256,
        auction_token_capacity: U256,
        bidding_token: BiddingToken,
        fee_numerator: u8,
        schedules: Schedules,
        merkle_root: Option<String>,
        tiers: Tiers,
    ) {
        self.0.call_contract(
            sender,
            "create_auction",
            runtime_args! {
                "id" => id,
                "info" => info,
                "auction_start_time" => auction_start_time,
                "auction_end_time" => auction_end_time,
                "project_open_time" => project_open_time,
                "auction_token" => auction_token,
                "auction_token_price" => auction_token_price,
                "auction_token_capacity" => auction_token_capacity,
                "bidding_token" => bidding_token,
                "fee_numerator" => fee_numerator,
                "schedules" => schedules,
                "merkle_root" => merkle_root,
                "tiers" => tiers,
            },
        )
    }

    pub fn get_auction(&self, auction_id: &str) -> Option<Auction> {
        self.0.query_dictionary("auctions", auction_id.to_string())
    }
}
