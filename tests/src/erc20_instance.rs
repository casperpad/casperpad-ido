use casper_types::{account::AccountHash, runtime_args, RuntimeArgs, U256};
use test_env::{TestContract, TestEnv};

pub struct ERC20Instance(TestContract);

impl ERC20Instance {
    pub fn new(env: &TestEnv, contract_name: &str, sender: AccountHash) -> ERC20Instance {
        ERC20Instance(TestContract::new(
            env,
            "erc20_token.wasm",
            contract_name,
            sender,
            runtime_args! {
              "name" => String::from("test token"),
              "symbol" => String::from("TTT"),
              "decimals" => 18u8,
              "total_supply" => U256::from(5000u32).checked_mul(U256::exp10(18)).unwrap(),
            },
        ))
    }
}
