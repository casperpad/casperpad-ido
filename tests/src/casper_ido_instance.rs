use casper_ido_contract::{
    enums::{Address, BiddingToken},
    structs::{Auction, Schedules, Tiers, Time},
};
use casper_types::{account::AccountHash, runtime_args, ContractPackageHash, RuntimeArgs, U256};
use test_env::{TestContract, TestEnv};

pub struct CasperIdoInstance(TestContract);

impl CasperIdoInstance {
    pub fn new(
        env: &TestEnv,
        contract_name: &str,
        sender: AccountHash,
        default_treasury_wallet: Address,
    ) -> CasperIdoInstance {
        CasperIdoInstance(TestContract::new(
            env,
            "casper_ido_contract.wasm",
            contract_name,
            sender,
            runtime_args! {
                "default_treasury_wallet" => default_treasury_wallet
            },
        ))
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

    pub fn contract_package_hash(&self) -> ContractPackageHash {
        self.0.contract_package_hash()
    }

    pub fn set_treasury_wallet(&self, sender: AccountHash, treasury_wallet: Address) {
        self.0.call_contract(
            sender,
            "set_treasury_wallet",
            runtime_args! {"treasury_wallet" => treasury_wallet},
        );
    }

    pub fn get_treasury_wallet(&self) -> Address {
        self.0.query_named_key("treasury_wallet".to_string())
    }

    pub fn get_fee_denominator(&self) -> U256 {
        self.0.query_named_key("fee_denominator".to_string())
    }

    pub fn create_order(
        &self,
        sender: AccountHash,
        auction_id: &str,
        proof: Vec<(String, u8)>,
        amount: U256,
    ) {
        self.0.call_contract(
            sender,
            "create_order",
            runtime_args! {
                "auction_id" => auction_id,
                "proof" => proof,
                "amount" => amount,

            },
        )
    }
}
