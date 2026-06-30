#![allow(dead_code)]

use kuest_builder_relayer_client::{
    BuilderApiKeyCreds, BuilderConfig, DepositWalletCall, RelayClient,
};

pub const TEST_PRIVATE_KEY: &str =
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
pub const ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
pub const DERIVED_WALLET: &str = "0xF3ab66D34F0B14C9a4f8564Ec8baaBBf51ad0Fd6";
pub const BATCH_WALLET: &str = "0xa2927E7834648F1C03b4961CeeA4597292e3c025";
pub const TOKEN: &str = "0x0000000000000000000000000000000000000001";
pub const APPROVE_CALLDATA: &str = "0x095ea7b30000000000000000000000000000000000000000000000000000000000000002ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
pub const EXPECTED_BATCH_SIGNATURE: &str = "0x7827946c566e7860f6c5f2e641587ed6928989c8618e463a00dd56832e7300023b7436c67a2ea82d6d506b1a5eda3e27526e9e2ffaad52128d75c47c2e9d1fac1b";

pub fn builder_config() -> BuilderConfig {
    BuilderConfig::local(BuilderApiKeyCreds::new(
        "019894b9-cb40-79c4-b2bd-6aecb6f8c6c5",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "1816e5ed89518467ffa78c65a2d6a62d240f6fd6d159cba7b2c4dc510800f75a",
    ))
    .expect("valid builder config")
}

pub fn deposit_wallet_call() -> DepositWalletCall {
    DepositWalletCall {
        target: TOKEN.to_owned(),
        value: "0".to_owned(),
        data: APPROVE_CALLDATA.to_owned(),
    }
}

pub fn client(relayer_url: &str, chain_id: u64) -> RelayClient {
    RelayClient::new_with_private_key(
        relayer_url,
        chain_id,
        TEST_PRIVATE_KEY,
        Some(builder_config()),
    )
    .expect("valid client")
}
