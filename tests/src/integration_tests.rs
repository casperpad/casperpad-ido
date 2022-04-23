#![feature(map_first_last)]
#[cfg(test)]

mod tests {
    use std::path::PathBuf;

    use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};

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

    const CONTRACT_HASH_KEY_NAME: &str = "casper_ido_contract_hash";
    const CONTRACT_KEY_NAME: &str = "ido_contract";
    const CONTRACT_WASM: &str = "contract.wasm";
    #[derive(Copy, Clone)]
    struct TestContext {
        // ido_package: ContractPackageHash,
        ido_contract: ContractHash,
    }

    fn setup() ->(InMemoryWasmTestBuilder, TestContext) {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST);

        let install_contract =
            ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, CONTRACT_WASM, runtime_args! {})
                .build();

        builder.exec(install_contract).expect_success().commit();

        let account = builder
            .get_account(*DEFAULT_ACCOUNT_ADDR)
            .expect("should have account");

        let ido_contract = account
            .named_keys()
            .get(CONTRACT_HASH_KEY_NAME)
            .and_then(|key| key.into_hash())
            .map(ContractHash::new)
            .expect("should have contract hash");

        let test_context = TestContext {
            ido_contract,
        };
        (builder, test_context)
    }

    #[test]
    fn should_install_contract() {
        setup();
        // dbg!(test_context.ido_package);
    }

    // #[test]
    // fn should_return_contract_name() {}

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
