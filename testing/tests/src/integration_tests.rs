#![feature(map_first_last)]

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_erc20::{
        constants::{AMOUNT_RUNTIME_ARG_NAME, APPROVE_ENTRY_POINT_NAME, SPENDER_RUNTIME_ARG_NAME},
        Address,
    };

    use casper_execution_engine::core::engine_state::ExecuteRequest;
    use casper_types::{
        account::AccountHash, runtime_args, ContractHash, ContractPackageHash, Key, PublicKey,
        RuntimeArgs, SecretKey, U256,
    };

    const MY_ACCOUNT: [u8; 32] = [7u8; 32];
    // Define `KEY` constant to match that in the contract.

    const CONTRACT_KEY_NAME: &str = "casper_ido";
    const ERC20_TOKEN_CONTRACT_KEY_NAME: &str = "erc20_token_contract";
    const OWNER_KEY_NAME: &str = "owner";
    const CONTRACT_HASH_KEY_NAME: &str = "casper_ido_contract_hash";
    const IDO_CONTRACT_WASM: &str = "casper_ido.wasm";
    const PRE_INVEST_CONTRACT_WASM: &str = "pre_invest.wasm";
    const ERC20_TOKEN_CONTRACT_WASM: &str = "erc20_token.wasm";
    const ERC20_TEST_CALL_CONTRACT_WASM: &str = "erc20_test_call.wasm";
    const OWNER_RUNTIME_ARG_NAME: &str = "owner";
    const TRANSFER_OWNERSHIP_ENRTY_NAME: &str = "transfer_ownership";
    const DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "default_treasury_wallet";
    const CREATE_PROJECT_ENTRY_NAME: &str = "add_project";
    const PROJECT_ID_RUNTIME_ARG_NAME: &str = "id";
    const PROJECT_NAME_RUNTIME_ARG_NAME: &str = "name";
    const PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME: &str = "sale_start_time";
    const PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME: &str = "sale_end_time";
    const PROJECT_OPEN_TIME_RUNTIME_ARG_NAME: &str = "open_time";
    const PROJECT_PRIVATE_RUNTIME_ARG_NAME: &str = "private";
    const PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME: &str = "token_price";
    const ADD_INVEST_ENTRY_NAME: &str = "add_invest";
    const CSPR_AMOUNT_RUNTIME_ARG_NAME: &str = "cspr_amount";
    const TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "treasury_wallet";
    const SET_DEFAULT_TREASURY_WALLET_ENTRY_NAME: &str = "set_default_treasury_wallet";
    const PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME: &str = "token_address";

    const SET_PROJECT_STATUS_ENTRY_NAME: &str = "set_project_status";
    const PROJECT_STATUS_RUNTIME_ARG_NAME: &str = "status";

    const PROJECT_SCHEDULES_RUNTIME_ARG_NAME: &str = "schedules";
    const MERKLE_ROOT_RUNTIME_ARG_NAME: &str = "merkle_root";
    const SET_MERKLE_ROOT_ENTRY_NAME: &str = "set_merkle_root";
    const CLAIM_ENTRY_NAME: &str = "claim";
    const PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME: &str = "locked_token_amount";
    const MERKLE_ROOT_KEY_NAME: &str = "merkle_root";
    const PROOF_RUNTIME_ARG_NAME: &str = "proof";

    #[derive(Copy, Clone)]
    struct TestContext {
        ido_contract_package: ContractPackageHash,
        ido_contract: ContractHash,
        erc20_token_contract: ContractHash,
    }

    fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
        let mut builder = InMemoryWasmTestBuilder::default();

        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        // let hash_addr: HashAddr = *DEFAULT_ACCOUNT_ADDR.value();

        let install_ido_contract = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            IDO_CONTRACT_WASM,
            runtime_args! {
                DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME => Key::from(*DEFAULT_ACCOUNT_ADDR),
            },
        )
        .build();

        builder.exec(install_ido_contract).expect_success().commit();

        let install_erc20_token_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            ERC20_TOKEN_CONTRACT_WASM,
            runtime_args! {
                "name" => String::from("test token"),
                "symbol" => String::from("TTT"),
                "decimals" => 18u8,
                "total_supply" => U256::from(5000u32),
            },
        )
        .build();

        builder
            .exec(install_erc20_token_request)
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

        let erc20_token_contract = account
            .named_keys()
            .get(ERC20_TOKEN_CONTRACT_KEY_NAME)
            .and_then(|key| key.into_hash())
            .map(ContractHash::new)
            .expect("should have contract hash");

        let test_context = TestContext {
            ido_contract_package,
            ido_contract,
            erc20_token_contract,
        };
        (builder, test_context)
    }

    fn account2() -> AccountHash {
        AccountHash::new([42; 32])
    }
    fn account3() -> AccountHash {
        AccountHash::new([43; 32])
    }

    fn make_set_merkle_root_request(context: TestContext, root: String) -> ExecuteRequest {
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract,
            SET_MERKLE_ROOT_ENTRY_NAME,
            runtime_args! {
                MERKLE_ROOT_RUNTIME_ARG_NAME => root,
            },
        )
        .build()
    }

    fn make_invest_request(context: TestContext) -> ExecuteRequest {
        let proof = vec![
            (
                "8680c6f98b4a17b9e4d2ed3c182c6d43e38dbffe1a346240a368d294407addad".to_string(),
                0u8,
            ),
            (
                "fc55bf77a246f2ba0e20011cd147a2511e8f99ad80e9fbeff656c2ba8b36e311".to_string(),
                1u8,
            ),
        ];
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract,
            ADD_INVEST_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => "swappery",
                CSPR_AMOUNT_RUNTIME_ARG_NAME => U256::from(100),
                PROOF_RUNTIME_ARG_NAME => proof
            },
        )
        .build()
    }

    // fn make_pre_invest_request(context: TestContext) -> ExecuteRequest {
    //     ExecuteRequestBuilder::standard(
    //         *DEFAULT_ACCOUNT_ADDR,
    //         PRE_INVEST_CONTRACT_WASM,
    //         runtime_args! {
    //             IDO_CONTRACT_HASH_KEY_RUNTIME_ARG_NAME => Key::from(context.ido_contract),
    //             PROJECT_ID_RUNTIME_ARG_NAME => "swappery",
    //             CSPR_AMOUNT_RUNTIME_ARG_NAME => U512::from(30)
    //         },
    //     )
    //     .build()
    // }

    fn make_erc20_approve_request(
        erc20_token: &ContractHash,
        spender: Key,
        amount: U256,
    ) -> ExecuteRequest {
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            *erc20_token,
            APPROVE_ENTRY_POINT_NAME,
            runtime_args! {
                SPENDER_RUNTIME_ARG_NAME => spender,
                AMOUNT_RUNTIME_ARG_NAME => amount,
            },
        )
        .build()
    }

    fn make_add_project_req(context: TestContext) -> ExecuteRequest {
        let schedules = vec![
            (1651071253130i64, U256::from(20u32)),
            (1651071253130i64, U256::from(80u32)),
        ];
        let erc20_contracthash = context.erc20_token_contract;

        ExecuteRequestBuilder::versioned_contract_call_by_hash(
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
                PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME => Key::from(erc20_contracthash),
                PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME => U256::from(10u32),
                PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME => U256::from(2000u32),
                TREASURY_WALLET_RUNTIME_ARG_NAME => *DEFAULT_ACCOUNT_ADDR,
                PROJECT_SCHEDULES_RUNTIME_ARG_NAME => schedules,
            },
        )
        .build()
    }

    fn make_set_project_status_req(
        project_id: &str,
        status: u32,
        context: TestContext,
    ) -> ExecuteRequest {
        ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            SET_PROJECT_STATUS_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => project_id,
                PROJECT_STATUS_RUNTIME_ARG_NAME => status,

            },
        )
        .build()
    }

    fn make_claim_req(project_id: &str, context: TestContext) -> ExecuteRequest {
        ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            CLAIM_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => project_id,

            },
        )
        .build()
    }

    #[test]
    fn should_install_ido_contract() {
        setup();
    }

    #[test]
    fn should_set_merkle_root() {
        let root = "3024df7bec4b2cb43ff4204a79799db70764c32f3badca73299573fe7f582324".to_string();
        let (mut builder, context) = setup();
        builder
            .exec(make_set_merkle_root_request(context, root.clone()))
            .expect_success()
            .commit();
        let result: String = builder.get_value(context.ido_contract, MERKLE_ROOT_KEY_NAME);
        // assert_eq!(result, U256::from(10));
        assert_eq!(result, root);
    }

    #[test]
    fn should_approve_erc20_to_ido_contract() {
        let (mut builder, context) = setup();
        let erc20_approve_req = make_erc20_approve_request(
            &context.erc20_token_contract,
            Key::from(context.ido_contract_package),
            U256::from(2000),
        );
        builder.exec(erc20_approve_req).expect_success().commit();
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
                OWNER_RUNTIME_ARG_NAME => account3()
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

        let set_default_treasury_wallet_req =
            ExecuteRequestBuilder::versioned_contract_call_by_hash(
                *DEFAULT_ACCOUNT_ADDR,
                context.ido_contract_package,
                None,
                SET_DEFAULT_TREASURY_WALLET_ENTRY_NAME,
                runtime_args! {
                    DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME => account2()
                },
            )
            .build();
        builder
            .exec(set_default_treasury_wallet_req)
            .expect_success()
            .commit();
    }

    #[test]
    fn should_add_project() {
        let (mut builder, context) = setup();
        let erc20_approve_req = make_erc20_approve_request(
            &context.erc20_token_contract,
            Key::from(context.ido_contract_package),
            U256::from(2000),
        );
        builder.exec(erc20_approve_req).expect_success().commit();

        let add_project_req = make_add_project_req(context);
        builder.exec(add_project_req).expect_success().commit();
    }

    #[test]
    fn should_set_project_status() {
        let (mut builder, context) = setup();
        let erc20_approve_req = make_erc20_approve_request(
            &context.erc20_token_contract,
            Key::from(context.ido_contract_package),
            U256::from(2000),
        );
        builder.exec(erc20_approve_req).expect_success().commit();

        let add_project_req = make_add_project_req(context);
        builder.exec(add_project_req).expect_success().commit();

        let set_project_status_req = make_set_project_status_req("swappery", 1u32, context);
        builder
            .exec(set_project_status_req)
            .expect_success()
            .commit();
    }

    #[test]
    fn should_add_invest() {
        let (mut builder, context) = setup();

        let root = "3024df7bec4b2cb43ff4204a79799db70764c32f3badca73299573fe7f582324".to_string();

        builder
            .exec(make_set_merkle_root_request(context, root.clone()))
            .expect_success()
            .commit();

        // // First create project
        // builder
        //     .exec(make_add_project_req(context))
        //     .expect_success()
        //     .commit();

        builder
            .exec(make_invest_request(context))
            .expect_success()
            .commit();
    }

    // #[test]
    // fn should_claim() {
    //     let (mut builder, context) = setup();
    //     let erc20_transfer_req = make_erc20_approve_request(
    //         Key::from(*DEFAULT_ACCOUNT_ADDR),
    //         &context.erc20_token_contract,
    //         Key::from(context.ido_contract_package),
    //         U256::from(2000u64),
    //     );
    //     builder.exec(erc20_transfer_req).expect_success().commit();
    //     let balance = erc20_check_balance_of(
    //         &mut builder,
    //         &context.erc20_token_contract,
    //         Key::from(context.ido_contract_package),
    //     );
    //     assert_eq!(balance, U256::from(2000u64));

    //     let balance = erc20_check_balance_of(
    //         &mut builder,
    //         &context.erc20_token_contract,
    //         Key::from(*DEFAULT_ACCOUNT_ADDR),
    //     );
    //     assert_eq!(balance, U256::from(3000u64));

    //     builder
    //         .exec(make_add_project_req(context))
    //         .expect_success()
    //         .commit();
    //     builder
    //         .exec(make_claim_req("swappery", context))
    //         .expect_success()
    //         .commit();

    //     let balance = erc20_check_balance_of(
    //         &mut builder,
    //         &context.erc20_token_contract,
    //         Key::from(*DEFAULT_ACCOUNT_ADDR),
    //     );

    //     assert_eq!(balance, U256::from(3500));
    // }

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
