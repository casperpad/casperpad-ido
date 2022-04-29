#![feature(map_first_last)]

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_erc20::Address;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };

    use casper_types::{
        account::AccountHash, runtime_args, ContractHash, ContractPackageHash, PublicKey,
        RuntimeArgs, SecretKey, U256,
    };

    const MY_ACCOUNT: [u8; 32] = [7u8; 32];
    // Define `KEY` constant to match that in the contract.

    const CONTRACT_KEY_NAME: &str = "casper_ido";
    const ERC20_TEST_CONTRACT_KEY_NAME: &str = "test_contract";
    const ERC20_TEST_CALL_KEY: &str = "erc20_test_call";
    const OWNER_KEY_NAME: &str = "owner";
    const CONTRACT_HASH_KEY_NAME: &str = "casper_ido_contract_hash";
    const IDO_CONTRACT_WASM: &str = "casper_ido.wasm";
    const ERC20_TEST_CONTRACT_WASM: &str = "erc20_test.wasm";
    const ERC20_TEST_CALL_CONTRACT_WASM: &str = "erc20_test_call.wasm";
    const OWNER_RUNTIME_ARG_NAME: &str = "owner";
    const TRANSFER_OWNERSHIP_ENRTY_NAME: &str = "transfer_ownership";
    const DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "default_treasury_wallet";
    const CREATE_PROJECT_ENTRY_NAME: &str = "add_project";
    const GET_PROJECT_INFO_ENTRY_NAME: &str = "get_project_info_by_id";
    const PROJECT_ID_RUNTIME_ARG_NAME: &str = "id";
    const PROJECT_NAME_RUNTIME_ARG_NAME: &str = "name";
    const PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME: &str = "sale_start_time";
    const PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME: &str = "sale_end_time";
    const PROJECT_OPEN_TIME_RUNTIME_ARG_NAME: &str = "open_time";
    const PROJECT_PRIVATE_RUNTIME_ARG_NAME: &str = "private";
    const PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME: &str = "token_symbol";
    const PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME: &str = "token_total_supply";
    const PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME: &str = "token_price";
    const ADD_INVEST_ENTRY_NAME: &str = "add_invest";
    const CSPR_AMOUNT_RUNTIME_ARG_NAME: &str = "cspr_amount";
    const GET_INVEST_INFO_ENTRY_NAME: &str = "get_invest_info";
    const RESULT_KEY_NAME: &str = "result";
    const TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "treasury_wallet";
    const SET_DEFAULT_TREASURY_WALLET_ENTRY_NAME: &str = "set_default_treasury_wallet";
    const GET_DEFAULT_TREASURY_WALLET_ENTRY_NAME: &str = "get_default_treasury_wallet";
    const PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME: &str = "token_address";
    #[derive(Copy, Clone)]
    struct TestContext {
        ido_contract_package: ContractPackageHash,
        ido_contract: ContractHash,
        erc20_test_contract: ContractHash,
        erc20_test_call_contract: ContractHash,
    }

    fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
        let mut builder = InMemoryWasmTestBuilder::default();

        builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST);

        let install_ido_contract = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            IDO_CONTRACT_WASM,
            runtime_args! {
                DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME => *DEFAULT_ACCOUNT_ADDR
            },
        )
        .build();

        builder.exec(install_ido_contract).expect_success().commit();

        let install_erc20_test_contract = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            ERC20_TEST_CONTRACT_WASM,
            runtime_args! {},
        )
        .build();

        builder
            .exec(install_erc20_test_contract)
            .expect_success()
            .commit();

        let install_erc20_test_call_contract = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            ERC20_TEST_CALL_CONTRACT_WASM,
            runtime_args! {},
        )
        .build();

        builder
            .exec(install_erc20_test_call_contract)
            .expect_success()
            .commit();

        let account = builder
            .get_account(*DEFAULT_ACCOUNT_ADDR)
            .expect("should have account");

        let ido_contract_package = account
            .named_keys()
            .get(CONTRACT_KEY_NAME)
            .and_then(|key| key.into_hash())
            .map(ContractPackageHash::new)
            .expect("should have contract package hash");

        let ido_contract = account
            .named_keys()
            .get(CONTRACT_HASH_KEY_NAME)
            .and_then(|key| key.into_hash())
            .map(ContractHash::new)
            .expect("should have contract hash");

        let erc20_test_contract = account
            .named_keys()
            .get(ERC20_TEST_CONTRACT_KEY_NAME)
            .and_then(|key| key.into_hash())
            .map(ContractHash::new)
            .expect("should have contract hash");
        let erc20_test_call_contract = account
            .named_keys()
            .get(ERC20_TEST_CALL_KEY)
            .and_then(|key| key.into_hash())
            .map(ContractHash::new)
            .expect("should have contract hash");

        let test_context = TestContext {
            ido_contract_package,
            ido_contract,
            erc20_test_contract,
            erc20_test_call_contract,
        };
        (builder, test_context)
    }

    fn account2() -> AccountHash {
        AccountHash::new([42; 32])
    }

    #[test]
    fn should_install_ido_contract() {
        setup();
    }

    #[test]
    fn should_transfer_contract_owner() {
        let (mut builder, context) = setup();
        let transfer_ownership_req = ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            TRANSFER_OWNERSHIP_ENRTY_NAME,
            runtime_args! {
                OWNER_RUNTIME_ARG_NAME => account2()
            },
        )
        .build();
        builder
            .exec(transfer_ownership_req)
            .expect_success()
            .commit();
        let result_of_query: Address = builder.get_value(context.ido_contract, OWNER_KEY_NAME);
        assert_eq!(result_of_query, Address::from(account2())); // *DEFAULT_ACCOUNT_ADDR
        let transfer_ownership_req2 = ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            TRANSFER_OWNERSHIP_ENRTY_NAME,
            runtime_args! {
                OWNER_RUNTIME_ARG_NAME => account2()
            },
        )
        .build();
        builder
            .exec(transfer_ownership_req2)
            .expect_failure()
            .commit();
    }

    #[test]
    fn should_set_default_treasury_wallet() {
        let (mut builder, context) = setup();
        let get_default_treasury_wallet_req =
            ExecuteRequestBuilder::versioned_contract_call_by_hash(
                *DEFAULT_ACCOUNT_ADDR,
                context.ido_contract_package,
                None,
                GET_DEFAULT_TREASURY_WALLET_ENTRY_NAME,
                runtime_args! {},
            )
            .build();
        builder
            .exec(get_default_treasury_wallet_req)
            .expect_success()
            .commit();
        let result_of_query: Address = builder.get_value(context.ido_contract, RESULT_KEY_NAME);
        assert_eq!(result_of_query, Address::from(*DEFAULT_ACCOUNT_ADDR)); // *DEFAULT_ACCOUNT_ADDR

        let set_default_treasury_wallet_req =
            ExecuteRequestBuilder::versioned_contract_call_by_hash(
                *DEFAULT_ACCOUNT_ADDR,
                context.ido_contract_package,
                None,
                SET_DEFAULT_TREASURY_WALLET_ENTRY_NAME,
                runtime_args! {
                    DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME => Address::from(account2())
                },
            )
            .build();
        builder
            .exec(set_default_treasury_wallet_req)
            .expect_success()
            .commit();

        let get_default_treasury_wallet_req =
            ExecuteRequestBuilder::versioned_contract_call_by_hash(
                *DEFAULT_ACCOUNT_ADDR,
                context.ido_contract_package,
                None,
                GET_DEFAULT_TREASURY_WALLET_ENTRY_NAME,
                runtime_args! {},
            )
            .build();
        builder
            .exec(get_default_treasury_wallet_req)
            .expect_success()
            .commit();
        let result_of_query: Address = builder.get_value(context.ido_contract, RESULT_KEY_NAME);
        assert_eq!(result_of_query, Address::from(account2())); // *DEFAULT_ACCOUNT_ADDR
    }

    #[test]
    fn should_add_project() {
        let (mut builder, context) = setup();
        print!("{}", context.erc20_test_contract);
        let add_project_req = ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            CREATE_PROJECT_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => "swappery",
                PROJECT_NAME_RUNTIME_ARG_NAME => "The first dex on Casper network.",
                PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME => 1651071253130i64,
                PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME => 1651071253130i64,
                PROJECT_OPEN_TIME_RUNTIME_ARG_NAME => 1651071253130i64,
                PROJECT_PRIVATE_RUNTIME_ARG_NAME => false,
                PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME => "SWPR",
                PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME => 10u32,
                PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME => 1000000u32,
                TREASURY_WALLET_RUNTIME_ARG_NAME => *DEFAULT_ACCOUNT_ADDR,
                PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME => context.erc20_test_contract
            },
        )
        .build();

        builder.exec(add_project_req).expect_success().commit();

        let get_project_req = ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            GET_PROJECT_INFO_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => "swappery",
            },
        )
        .build();
        builder.exec(get_project_req).expect_success().commit();

        let result: String = builder.get_value(context.ido_contract, RESULT_KEY_NAME);
        assert_ne!(result, "");
    }

    #[test]
    fn should_add_invest() {
        let (mut builder, context) = setup();
        let add_invest_req = ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            ADD_INVEST_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => "swappery",
                CSPR_AMOUNT_RUNTIME_ARG_NAME => U256::from(10)
            },
        )
        .build();

        builder.exec(add_invest_req).expect_success().commit();

        let get_invest_info_req = ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            GET_INVEST_INFO_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => "swappery",
            },
        )
        .build();
        builder.exec(get_invest_info_req).expect_success().commit();

        let result: U256 = builder.get_value(context.ido_contract, RESULT_KEY_NAME);
        assert_eq!(result, U256::from(10));
    }

    #[test]
    fn should_error_on_missing_runtime_arg() {
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);

        let session_code = PathBuf::from(IDO_CONTRACT_WASM);
        let session_args = RuntimeArgs::new();

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[account_addr])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .with_session_code(session_code, session_args)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder.exec(execute_request).commit().expect_failure();
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
