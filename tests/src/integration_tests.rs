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
        RuntimeArgs, SecretKey,
    };

    const MY_ACCOUNT: [u8; 32] = [7u8; 32];
    // Define `KEY` constant to match that in the contract.

    const CONTRACT_KEY_NAME: &str = "casper_ido";
    const OWNER_KEY_NAME: &str = "owner";
    const CONTRACT_HASH_KEY_NAME: &str = "casper_ido_contract_hash";
    const CONTRACT_WASM: &str = "contract.wasm";
    const OWNER_RUNTIME_ARG_NAME: &str = "owner";
    const TRANSFER_OWNERSHIP_ENRTY_NAME: &str = "transfer_ownership";
    const DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "default_treasury_wallet";
    #[derive(Copy, Clone)]
    struct TestContext {
        ido_contract_package: ContractPackageHash,
        ido_contract: ContractHash,
    }

    fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST);

        let install_contract = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            CONTRACT_WASM,
            runtime_args! {
                DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME => *DEFAULT_ACCOUNT_ADDR
            },
        )
        .build();

        builder.exec(install_contract).expect_success().commit();

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

        let test_context = TestContext {
            ido_contract_package,
            ido_contract,
        };
        (builder, test_context)
    }

    fn account2() -> AccountHash {
        const MY_ACCOUNT: [u8; 32] = [7u8; 32];
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);

        // Create an AccountHash from a public key.
        AccountHash::from(&public_key)
    }

    #[test]
    fn should_install_contract() {
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
    fn should_error_on_missing_runtime_arg() {
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);

        let session_code = PathBuf::from(CONTRACT_WASM);
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
