#![feature(map_first_last)]

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_erc20::constants::{
        AMOUNT_RUNTIME_ARG_NAME, APPROVE_ENTRY_POINT_NAME, DECIMALS_RUNTIME_ARG_NAME,
        NAME_RUNTIME_ARG_NAME, SPENDER_RUNTIME_ARG_NAME, SYMBOL_RUNTIME_ARG_NAME,
        TOTAL_SUPPLY_RUNTIME_ARG_NAME,
    };

    use casper_execution_engine::core::engine_state::ExecuteRequest;
    use casper_types::{
        account::AccountHash, bytesrepr::FromBytes, runtime_args, CLTyped, ContractHash,
        ContractPackageHash, Key, PublicKey, RuntimeArgs, SecretKey, U256, U512,
    };

    const MY_ACCOUNT: [u8; 32] = [7u8; 32];
    // Define `KEY` constant to match that in the contract.

    const IDO_CONTRACT_WASM: &str = "casper_ido.wasm";
    const PRE_INVEST_CONTRACT_WASM: &str = "pre_invest.wasm";

    const CONTRACT_KEY_NAME: &str = "casper_ido";
    const ERC20_TOKEN_CONTRACT_KEY_NAME: &str = "erc20_token_contract";
    const OWNER_KEY_NAME: &str = "owner";
    const CONTRACT_HASH_KEY_NAME: &str = "casper_ido_contract_hash";
    const ERC20_TOKEN_CONTRACT_WASM: &str = "erc20_token.wasm";
    const ERC20_TEST_CALL_CONTRACT_WASM: &str = "erc20_test_call.wasm";
    const OWNER_RUNTIME_ARG_NAME: &str = "owner";
    const TRANSFER_OWNERSHIP_ENRTY_NAME: &str = "transfer_ownership";
    const CREATE_PROJECT_ENTRY_NAME: &str = "add_project";
    const PROJECT_ID_RUNTIME_ARG_NAME: &str = "id";
    const PROJECT_NAME_RUNTIME_ARG_NAME: &str = "name";
    const PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME: &str = "sale_start_time";
    const PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME: &str = "sale_end_time";
    const PROJECT_OPEN_TIME_RUNTIME_ARG_NAME: &str = "open_time";
    const PROJECT_PRIVATE_RUNTIME_ARG_NAME: &str = "private";
    const PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME: &str = "token_price";

    const TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "treasury_wallet";

    const PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME: &str = "token_address";

    const SET_PROJECT_STATUS_ENTRY_NAME: &str = "set_project_status";
    const PROJECT_STATUS_RUNTIME_ARG_NAME: &str = "status";
    const SET_CSPR_PRICE_ENTRY_NAME: &str = "set_cspr_price";
    const CSPR_PRICE_RUNTIME_ARG_NAME: &str = "cspr_price";
    const PROJECT_SCHEDULES_RUNTIME_ARG_NAME: &str = "schedules";
    const MERKLE_ROOT_RUNTIME_ARG_NAME: &str = "merkle_root";
    const SET_MERKLE_ROOT_ENTRY_NAME: &str = "set_merkle_root";
    const CLAIM_ENTRY_NAME: &str = "claim";
    const PROJECT_TOKEN_CAPACITY_RUNTIME_ARG_NAME: &str = "token_capacity";
    const MERKLE_ROOT_KEY_NAME: &str = "merkle_root";
    const PROOF_RUNTIME_ARG_NAME: &str = "proof";
    const ERC20_TEST_CALL_KEY: &str = "erc20_test_call";
    const RESULT_KEY_NAME: &str = "result";
    const CHECK_BALANCE_OF_ENTRYPOINT: &str = "check_balance_of";
    const ARG_TOKEN_CONTRACT: &str = "token_contract";
    const ARG_ADDRESS: &str = "address";
    const SCHEDULE_ID_RUNTIME_ARG_NAME: &str = "schedule_id";
    const IDO_CONTRACT_HASH_KEY_RUNTIME_ARG_NAME: &str = "ido_contract_hash";
    const SET_PURSE_ENTRY_NAME: &str = "set_purse";
    const DEFAULT_ACCOUNT_ADDR_STRING: &str =
        "account-hash-58b891759929bd4ed5a9cce20b9d6e3c96a66c21386bed96040e17dd07b79fa7";
    const SET_MULTIPLE_TIERS_ENTRY_NAME: &str = "set_multiple_tiers";
    const SET_TIER_ENTRY_NAME: &str = "set_tier";
    const MULTIPLE_TIERS_RUNTIME_ARG_NAME: &str = "tiers";
    const TIER_RUNTIME_ARG_NAME: &str = "tier";

    #[derive(Copy, Clone)]
    struct TestContext {
        ido_contract_package: ContractPackageHash,
        ido_contract: ContractHash,
        erc20_token_contract: ContractHash,
    }

    fn get_test_result<T: FromBytes + CLTyped>(
        builder: &mut InMemoryWasmTestBuilder,
        contract_package_hash: ContractPackageHash,
    ) -> T {
        let contract_package = builder
            .get_contract_package(contract_package_hash)
            .expect("should have contract package");
        let enabled_versions = contract_package.enabled_versions();
        let (_version, contract_hash) = enabled_versions
            .iter()
            .rev()
            .next()
            .expect("should have latest version");

        builder.get_value(*contract_hash, RESULT_KEY_NAME)
    }

    fn erc20_check_balance_of(
        builder: &mut InMemoryWasmTestBuilder,
        erc20_contract_hash: &ContractHash,
        address: Key,
    ) -> U256 {
        let account = builder
            .get_account(*DEFAULT_ACCOUNT_ADDR)
            .expect("should have account");

        let erc20_test_contract_hash = account
            .named_keys()
            .get(ERC20_TEST_CALL_KEY)
            .and_then(|key| key.into_hash())
            .map(ContractPackageHash::new)
            .expect("should have test contract hash");

        let check_balance_args = runtime_args! {
            ARG_TOKEN_CONTRACT => *erc20_contract_hash,
            ARG_ADDRESS => address,
        };
        let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            erc20_test_contract_hash,
            None,
            CHECK_BALANCE_OF_ENTRYPOINT,
            check_balance_args,
        )
        .build();
        builder.exec(exec_request).expect_success().commit();

        get_test_result(builder, erc20_test_contract_hash)
    }

    fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
        let mut builder = InMemoryWasmTestBuilder::default();

        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

        // let hash_addr: HashAddr = *DEFAULT_ACCOUNT_ADDR.value();

        let install_ido_contract = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            IDO_CONTRACT_WASM,
            runtime_args! {},
        )
        .build();

        builder.exec(install_ido_contract).expect_success().commit();

        let install_erc20_token_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            ERC20_TOKEN_CONTRACT_WASM,
            runtime_args! {
                NAME_RUNTIME_ARG_NAME => String::from("test token"),
                SYMBOL_RUNTIME_ARG_NAME => String::from("TTT"),
                DECIMALS_RUNTIME_ARG_NAME => 18u8,
                TOTAL_SUPPLY_RUNTIME_ARG_NAME => U256::from(5000u32).checked_mul(U256::exp10(18)).unwrap(),
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

    fn account2() -> String {
        AccountHash::new([42; 32]).to_formatted_string()
    }

    fn account3() -> String {
        AccountHash::new([43; 32]).to_formatted_string()
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

    fn get_proof() -> Vec<(String, u8)> {
        vec![
            (
                "8680c6f98b4a17b9e4d2ed3c182c6d43e38dbffe1a346240a368d294407addad".to_string(),
                0u8,
            ),
            (
                "fc55bf77a246f2ba0e20011cd147a2511e8f99ad80e9fbeff656c2ba8b36e311".to_string(),
                1u8,
            ),
        ]
    }

    fn make_pre_invest_request(context: TestContext) -> ExecuteRequest {
        ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            PRE_INVEST_CONTRACT_WASM,
            runtime_args! {
                IDO_CONTRACT_HASH_KEY_RUNTIME_ARG_NAME => Key::from(context.ido_contract),
                PROJECT_ID_RUNTIME_ARG_NAME => "swappery",
                AMOUNT_RUNTIME_ARG_NAME => U512::from(1).checked_mul(U512::exp10(9)).unwrap(),
                PROOF_RUNTIME_ARG_NAME => get_proof()
            },
        )
        .build()
    }

    fn make_set_purse_request(context: TestContext) -> ExecuteRequest {
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract,
            SET_PURSE_ENTRY_NAME,
            runtime_args! {},
        )
        .build()
    }

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
            (1651071253130i64, U256::from(50000)),
            (1651071253130i64, U256::from(50000)),
        ];
        let erc20_contracthash = context.erc20_token_contract;

        println!("{:?}", *DEFAULT_ACCOUNT_ADDR);

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
                PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME => U256::from(1u32).checked_mul(U256::exp10(18 - 2)).unwrap(),
                PROJECT_TOKEN_CAPACITY_RUNTIME_ARG_NAME => U256::from(2000u32).checked_mul(U256::exp10(18)).unwrap(),
                TREASURY_WALLET_RUNTIME_ARG_NAME => DEFAULT_ACCOUNT_ADDR_STRING,
                PROJECT_SCHEDULES_RUNTIME_ARG_NAME => schedules,
            },
        )
        .build()
    }

    fn make_set_cspr_price_request(context: TestContext) -> ExecuteRequest {
        ExecuteRequestBuilder::contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract,
            SET_CSPR_PRICE_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => "swappery",
                CSPR_PRICE_RUNTIME_ARG_NAME => U256::from(500).checked_mul(U256::exp10(18 - 3)).unwrap(), // assert(USD Token decimals is 18)
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

    fn make_claim_req(project_id: &str, schedule_id: u8, context: TestContext) -> ExecuteRequest {
        ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            CLAIM_ENTRY_NAME,
            runtime_args! {
                PROJECT_ID_RUNTIME_ARG_NAME => project_id,
                SCHEDULE_ID_RUNTIME_ARG_NAME => schedule_id
            },
        )
        .build()
    }

    fn make_set_multiple_tiers_req(context: TestContext) -> ExecuteRequest {
        let tiers: Vec<(String, U256)> = vec![
            (
                DEFAULT_ACCOUNT_ADDR_STRING.to_string(),
                U256::from(100).checked_mul(U256::exp10(18)).unwrap(),
            ),
            (
                account2(),
                U256::from(100).checked_mul(U256::exp10(18)).unwrap(),
            ),
            (
                account3(),
                U256::from(100).checked_mul(U256::exp10(18)).unwrap(),
            ),
        ];
        ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            SET_MULTIPLE_TIERS_ENTRY_NAME,
            runtime_args! {
                MULTIPLE_TIERS_RUNTIME_ARG_NAME => tiers,
            },
        )
        .build()
    }

    fn make_set_tier_req(context: TestContext) -> ExecuteRequest {
        let tier: (String, U256) = (
            DEFAULT_ACCOUNT_ADDR_STRING.to_string(),
            U256::from(100).checked_mul(U256::exp10(18)).unwrap(),
        );
        ExecuteRequestBuilder::versioned_contract_call_by_hash(
            *DEFAULT_ACCOUNT_ADDR,
            context.ido_contract_package,
            None,
            SET_TIER_ENTRY_NAME,
            runtime_args! {
                TIER_RUNTIME_ARG_NAME => tier,
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
            U256::from(2000u32).checked_mul(U256::exp10(18)).unwrap(),
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
        let result_of_query: AccountHash = builder.get_value(context.ido_contract, OWNER_KEY_NAME);
        assert_eq!(
            result_of_query,
            AccountHash::from_formatted_str(&account2()).unwrap()
        ); // *DEFAULT_ACCOUNT_ADDR
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
    fn should_add_project() {
        let (mut builder, context) = setup();
        let erc20_approve_req = make_erc20_approve_request(
            &context.erc20_token_contract,
            Key::from(context.ido_contract_package),
            U256::from(2000u32).checked_mul(U256::exp10(18)).unwrap(),
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
            U256::from(2000u32).checked_mul(U256::exp10(18)).unwrap(),
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

        let erc20_approve_req = make_erc20_approve_request(
            &context.erc20_token_contract,
            Key::from(context.ido_contract_package),
            U256::from(2000u32).checked_mul(U256::exp10(18)).unwrap(),
        );
        builder.exec(erc20_approve_req).expect_success().commit();

        // First create project
        builder
            .exec(make_add_project_req(context))
            .expect_success()
            .commit();

        builder
            .exec(make_set_cspr_price_request(context))
            .expect_success()
            .commit();

        builder
            .exec(make_set_purse_request(context))
            .expect_success()
            .commit();

        builder
            .exec(make_pre_invest_request(context))
            .expect_success()
            .commit();
    }

    #[test]
    fn should_claim() {
        let (mut builder, context) = setup();

        let root = "3024df7bec4b2cb43ff4204a79799db70764c32f3badca73299573fe7f582324".to_string();

        builder
            .exec(make_set_merkle_root_request(context, root.clone()))
            .expect_success()
            .commit();

        let balance = erc20_check_balance_of(
            &mut builder,
            &context.erc20_token_contract,
            Key::from(*DEFAULT_ACCOUNT_ADDR),
        );
        assert_eq!(
            balance,
            U256::from(5000u32).checked_mul(U256::exp10(18)).unwrap()
        );

        let erc20_approve_req = make_erc20_approve_request(
            &context.erc20_token_contract,
            Key::from(context.ido_contract_package),
            U256::from(2000u32).checked_mul(U256::exp10(18)).unwrap(),
        );
        builder.exec(erc20_approve_req).expect_success().commit();

        builder
            .exec(make_add_project_req(context))
            .expect_success()
            .commit();

        let balance = erc20_check_balance_of(
            &mut builder,
            &context.erc20_token_contract,
            Key::from(*DEFAULT_ACCOUNT_ADDR),
        );
        assert_eq!(
            balance,
            U256::from(3000u32).checked_mul(U256::exp10(18)).unwrap()
        );

        builder
            .exec(make_set_cspr_price_request(context))
            .expect_success()
            .commit();

        builder
            .exec(make_set_purse_request(context))
            .expect_success()
            .commit();

        builder
            .exec(make_pre_invest_request(context))
            .expect_success()
            .commit();

        builder
            .exec(make_claim_req("swappery", 0, context))
            .expect_success()
            .commit();
        builder
            .exec(make_claim_req("swappery", 0, context))
            .expect_failure()
            .commit();
        builder
            .exec(make_claim_req("swappery", 1, context))
            .expect_success()
            .commit();

        let balance = erc20_check_balance_of(
            &mut builder,
            &context.erc20_token_contract,
            Key::from(*DEFAULT_ACCOUNT_ADDR),
        );

        assert_eq!(
            balance,
            U256::from(3050u32).checked_mul(U256::exp10(18)).unwrap()
        );
    }

    #[test]
    fn should_set_multiple_tiers() {
        let (mut builder, context) = setup();
        builder
            .exec(make_set_multiple_tiers_req(context))
            .expect_success()
            .commit();
    }

    #[test]
    fn should_set_tier() {
        let (mut builder, context) = setup();
        builder
            .exec(make_set_tier_req(context))
            .expect_success()
            .commit();
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
