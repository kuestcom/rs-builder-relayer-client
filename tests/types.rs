mod common;

use common::{ADDRESS, DERIVED_WALLET};
use kuest_builder_relayer_client::{RelayerTransaction, TransactionType};

#[test]
fn relayer_transaction_deserialization_validates_transaction_type() {
    let transaction = serde_json::from_value::<RelayerTransaction>(serde_json::json!({
        "transactionID": "txn-1",
        "transactionHash": "0xabc",
        "from": ADDRESS,
        "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
        "walletAddress": DERIVED_WALLET,
        "data": "0x",
        "nonce": "0",
        "value": "0",
        "state": "STATE_NEW",
        "failureReason": null,
        "type": "WALLET",
        "metadata": null,
        "createdAt": "2026-05-24T18:00:00Z",
        "updatedAt": "2026-05-24T18:00:00Z"
    }))
    .expect("valid transaction type should deserialize");

    assert_eq!(transaction.transaction_type, TransactionType::Wallet);
}

#[test]
fn relayer_transaction_deserialization_rejects_unknown_transaction_type() {
    let error = serde_json::from_value::<RelayerTransaction>(serde_json::json!({
        "transactionID": "txn-1",
        "transactionHash": "0xabc",
        "from": ADDRESS,
        "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
        "walletAddress": DERIVED_WALLET,
        "data": "0x",
        "nonce": "0",
        "value": "0",
        "state": "STATE_NEW",
        "failureReason": null,
        "type": "UNEXPECTED",
        "metadata": null,
        "createdAt": "2026-05-24T18:00:00Z",
        "updatedAt": "2026-05-24T18:00:00Z"
    }))
    .expect_err("unknown wire value should fail");

    assert!(error.to_string().contains("unknown variant"));
}
