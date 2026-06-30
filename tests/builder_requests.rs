mod common;

use common::{
    ADDRESS, BATCH_WALLET, EXPECTED_BATCH_SIGNATURE, TEST_PRIVATE_KEY, deposit_wallet_call,
};
use kuest_builder_relayer_client::{
    DepositWalletTransactionArgs, PrivateKeySigner, build_deposit_wallet_batch_request,
    build_deposit_wallet_create_request, get_contract_config,
};
use std::str::FromStr as _;

#[test]
fn build_deposit_wallet_create_request_matches_current_clients() {
    let config = get_contract_config(137).expect("known chain");
    let request = build_deposit_wallet_create_request(ADDRESS, &config.deposit_wallet_contracts);

    assert_eq!(
        serde_json::to_value(request).expect("serialize"),
        serde_json::json!({
            "type": "WALLET-CREATE",
            "from": ADDRESS,
            "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
        })
    );
}

#[tokio::test]
async fn build_deposit_wallet_batch_request_matches_fixed_signature_vector() {
    let config = get_contract_config(137).expect("known chain");
    let signer = PrivateKeySigner::from_str(TEST_PRIVATE_KEY).expect("valid signer");
    let request = build_deposit_wallet_batch_request(
        &signer,
        &DepositWalletTransactionArgs {
            from: ADDRESS.to_owned(),
            chain_id: 137,
            wallet_address: BATCH_WALLET.to_owned(),
            nonce: "0".to_owned(),
            deadline: "1234567890".to_owned(),
            calls: vec![deposit_wallet_call()],
        },
        &config.deposit_wallet_contracts,
    )
    .await
    .expect("request builds");

    assert_eq!(request.signature, EXPECTED_BATCH_SIGNATURE);
    assert_eq!(
        serde_json::to_value(request).expect("serialize"),
        serde_json::json!({
                "type": "WALLET",
                "from": ADDRESS,
                "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
                "nonce": "0",
                "signature": EXPECTED_BATCH_SIGNATURE,
                "depositWalletParams": {
                "depositWallet": BATCH_WALLET,
                "deadline": "1234567890",
                "calls": [{
                    "target": "0x0000000000000000000000000000000000000001",
                    "value": "0",
                    "data": "0x095ea7b30000000000000000000000000000000000000000000000000000000000000002ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                }],
            },
        })
    );
}
