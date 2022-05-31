use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use casper_ido_contract::enums::Address;
use casper_types::{
    account::AccountHash, bytesrepr::ToBytes, runtime_args, ContractHash, RuntimeArgs, U256,
};
use test_env::{TestContract, TestEnv};

fn blake2b256(item_key_string: &[u8]) -> Box<[u8]> {
    let mut hasher = Blake2bVar::new(32).unwrap();
    hasher.update(item_key_string);
    hasher.finalize_boxed()
}

pub struct ERC20Instance(TestContract);

impl ERC20Instance {
    pub fn new(
        env: &TestEnv,
        contract_name: &str,
        sender: AccountHash,
        symbol: &str,
        decimals: u8,
        total_supply: U256,
    ) -> ERC20Instance {
        ERC20Instance(TestContract::new(
            env,
            "erc20_token.wasm",
            contract_name,
            sender,
            runtime_args! {
              "name" => String::from(contract_name),
              "symbol" => String::from(symbol),
              "decimals" => decimals,
              "total_supply" => total_supply,
            },
        ))
    }

    pub fn contract_hash(&self) -> ContractHash {
        self.0.contract_hash()
    }

    pub fn name(&self) -> String {
        self.0.query_named_key("name".to_string())
    }
    pub fn symbol(&self) -> String {
        self.0.query_named_key("symbol".to_string())
    }
    pub fn total_supply(&self) -> U256 {
        self.0.query_named_key("total_supply".to_string())
    }

    pub fn decimals(&self) -> u8 {
        self.0.query_named_key("decimals".to_string())
    }

    pub fn balance_of(&self, account: Address) -> Result<U256, String> {
        let item_key = base64::encode(&account.to_bytes().unwrap());

        self.0.query_dictionary_old("balances", item_key)
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, String> {
        let mut preimage = Vec::new();
        preimage.append(&mut owner.to_bytes().unwrap());
        preimage.append(&mut spender.to_bytes().unwrap());
        let key_bytes = blake2b256(&preimage);
        let allowance_item_key = hex::encode(&key_bytes);

        self.0
            .query_dictionary_old("allowances", allowance_item_key)
    }

    pub fn transfer_from(
        &self,
        sender: AccountHash,
        owner: Address,
        recipient: Address,
        amount: U256,
    ) {
        self.0.call_contract(
            sender,
            "transfer_from",
            runtime_args! {
              "owner" => owner,
              "recipient" => recipient,
              "amount" => amount,
            },
        )
    }

    pub fn approve(&self, sender: AccountHash, spender: Address, amount: U256) {
        self.0.call_contract(
            sender,
            "approve",
            runtime_args! {
                "spender" => spender,
                "amount" => amount
            },
        )
    }

    pub fn transfer(&self, sender: AccountHash, recipient: Address, amount: U256) {
        self.0.call_contract(
            sender,
            "transfer",
            runtime_args! {
                "recipient" => recipient,
                "amount" => amount
            },
        )
    }
}
