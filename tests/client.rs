mod common;

use common::{
    ADDRESS, BATCH_WALLET, DERIVED_WALLET, EXPECTED_BATCH_SIGNATURE, builder_config, client,
    deposit_wallet_call,
};
use httpmock::prelude::*;
use kuest_builder_relayer_client::{Error, RelayClient, RelayerTransactionState, TransactionType};
use serde_json::Value;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn derive_aliases_match_current_clients() {
    let client = client("http://localhost:8080", 80002);
    assert_eq!(
        client.derive_deposit_wallet().expect("wallet derives"),
        DERIVED_WALLET
    );
    assert_eq!(
        client
            .derive_deposit_wallet_address()
            .expect("wallet derives"),
        DERIVED_WALLET
    );
    assert_eq!(
        client
            .get_expected_deposit_wallet()
            .expect("wallet derives"),
        DERIVED_WALLET
    );
}

#[tokio::test]
async fn get_nonce_uses_public_endpoint() {
    let server = MockServer::start();
    let nonce_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/nonce")
            .query_param("address", ADDRESS)
            .query_param("type", "WALLET");
        then.status(200)
            .json_body_obj(&serde_json::json!({ "nonce": "7" }));
    });

    let client = RelayClient::new(&server.base_url(), 137, None, None).expect("valid client");
    let payload = client.get_nonce(ADDRESS, None).await.expect("nonce loads");

    nonce_mock.assert();
    assert_eq!(payload.nonce, "7");
}

#[tokio::test]
async fn get_transaction_uses_public_endpoint() {
    let server = MockServer::start();
    let tx_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/transaction")
            .query_param("id", "txn-1");
        then.status(200).json_body_obj(&serde_json::json!([{
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
        }]));
    });

    let client = RelayClient::new(&server.base_url(), 137, None, None).expect("valid client");
    let payload = client
        .get_transaction("txn-1")
        .await
        .expect("transaction loads");

    tx_mock.assert();
    assert_eq!(payload.len(), 1);
    assert_eq!(payload[0].transaction_id, "txn-1");
    assert_eq!(payload[0].transaction_hash.as_deref(), Some("0xabc"));
    assert_eq!(payload[0].transaction_type, TransactionType::Wallet);
}

#[tokio::test]
async fn get_deployed_uses_public_endpoint() {
    let server = MockServer::start();
    let deployed_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/deployed")
            .query_param("address", DERIVED_WALLET);
        then.status(200)
            .json_body_obj(&serde_json::json!({ "deployed": true }));
    });

    let client = RelayClient::new(&server.base_url(), 137, None, None).expect("valid client");
    let deployed = client
        .get_deployed(DERIVED_WALLET)
        .await
        .expect("deployment loads");

    deployed_mock.assert();
    assert!(deployed);
}

#[tokio::test]
async fn get_transactions_requires_builder_credentials() {
    let client = RelayClient::new("http://localhost:8080", 137, None, None).expect("valid client");
    let error = client
        .get_transactions()
        .await
        .expect_err("missing builder config should fail");

    assert!(matches!(error, Error::BuilderCredentialsUnavailable));
}

#[tokio::test]
async fn get_transactions_sends_builder_auth_headers() {
    let server = MockServer::start();
    let headers_seen = Arc::new(Mutex::new(Vec::new()));
    let headers_seen_ref = Arc::clone(&headers_seen);
    let tx_mock = server.mock(|when, then| {
        when.method(GET).path("/transactions");
        then.respond_with(move |req: &HttpMockRequest| {
            let headers = req.headers();
            let mut seen = headers_seen_ref.lock().expect("lock");
            seen.push(
                headers
                    .get("KUEST_BUILDER_API_KEY")
                    .expect("api key header")
                    .to_str()
                    .expect("utf8")
                    .to_owned(),
            );
            assert!(headers.get("KUEST_BUILDER_SIGNATURE").is_some());
            assert!(headers.get("KUEST_BUILDER_PASSPHRASE").is_some());
            assert!(headers.get("KUEST_BUILDER_TIMESTAMP").is_some());

            HttpMockResponse::builder()
                .status(200)
                .header("content-type", "application/json")
                .body("[]")
                .build()
        });
    });

    let client = RelayClient::new(&server.base_url(), 137, None, Some(builder_config()))
        .expect("valid client");

    let transactions = client.get_transactions().await.expect("transactions load");

    tx_mock.assert();
    assert!(transactions.is_empty());
    assert_eq!(
        headers_seen.lock().expect("lock").as_slice(),
        ["019894b9-cb40-79c4-b2bd-6aecb6f8c6c5"]
    );
}

#[tokio::test]
async fn deploy_deposit_wallet_posts_wallet_create() {
    let server = MockServer::start();
    let captured = Arc::new(Mutex::new(None::<Value>));
    let captured_ref = Arc::clone(&captured);
    let submit_mock = server.mock(|when, then| {
        when.method(POST).path("/submit");
        then.respond_with(move |req: &HttpMockRequest| {
            *captured_ref.lock().expect("lock") =
                Some(serde_json::from_str(&req.body_string()).expect("valid json"));
            HttpMockResponse::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"transactionID":"test-txn","state":"STATE_NEW","transactionHash":"0xabc","hash":"0xabc"}"#,
                )
                .build()
        });
    });

    let response = client(&server.base_url(), 137)
        .deploy_deposit_wallet()
        .await
        .expect("deploy succeeds");

    submit_mock.assert();
    assert_eq!(
        captured.lock().expect("lock").clone().expect("body"),
        serde_json::json!({
            "type": "WALLET-CREATE",
            "from": ADDRESS,
            "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
        })
    );
    assert_eq!(response.transaction_id, "test-txn");
    assert_eq!(response.transaction_hash.as_deref(), Some("0xabc"));
    assert_eq!(response.hash.as_deref(), Some("0xabc"));
}

#[tokio::test]
async fn execute_deposit_wallet_batch_posts_wallet_request() {
    let server = MockServer::start();
    let nonce_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/nonce")
            .query_param("address", ADDRESS)
            .query_param("type", "WALLET");
        then.status(200)
            .json_body_obj(&serde_json::json!({ "nonce": "0" }));
    });
    let captured = Arc::new(Mutex::new(None::<Value>));
    let captured_ref = Arc::clone(&captured);
    let submit_mock = server.mock(|when, then| {
        when.method(POST).path("/submit");
        then.respond_with(move |req: &HttpMockRequest| {
            *captured_ref.lock().expect("lock") =
                Some(serde_json::from_str(&req.body_string()).expect("valid json"));
            HttpMockResponse::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"transactionID":"test-txn","state":"STATE_NEW","transactionHash":"0xabc","hash":"0xabc"}"#,
                )
                .build()
        });
    });

    let response = client(&server.base_url(), 137)
        .execute_deposit_wallet_batch(&[deposit_wallet_call()], BATCH_WALLET, "1234567890")
        .await
        .expect("execute succeeds");

    nonce_mock.assert();
    submit_mock.assert();
    assert_eq!(response.transaction_id, "test-txn");
    assert_eq!(
        captured.lock().expect("lock").clone().expect("body"),
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

#[tokio::test]
async fn execute_deposit_wallet_batch_public_uses_public_wallet_endpoint() {
    let server = MockServer::start();
    let nonce_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/nonce")
            .query_param("address", ADDRESS)
            .query_param("type", "WALLET");
        then.status(200)
            .json_body_obj(&serde_json::json!({ "nonce": "0" }));
    });
    let submit_mock = server.mock(|when, then| {
        when.method(POST).path("/submit/wallet");
        then.status(200).json_body_obj(&serde_json::json!({
            "transactionID": "test-txn",
            "state": "STATE_NEW",
            "transactionHash": "0xabc",
            "hash": "0xabc",
        }));
    });

    let response = client(&server.base_url(), 137)
        .execute_deposit_wallet_batch_public(&[deposit_wallet_call()], BATCH_WALLET, "1234567890")
        .await
        .expect("execute succeeds");

    nonce_mock.assert();
    submit_mock.assert();
    assert_eq!(response.transaction_id, "test-txn");
}

#[tokio::test]
async fn poll_until_state_returns_success_early() {
    let server = MockServer::start();
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_ref = Arc::clone(&call_count);
    let tx_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/transaction")
            .query_param("id", "txn-1");
        then.respond_with(move |_req: &HttpMockRequest| {
            let index = call_count_ref.fetch_add(1, Ordering::Relaxed);
            let state = if index == 0 {
                "STATE_NEW"
            } else {
                "STATE_CONFIRMED"
            };
            HttpMockResponse::builder()
                .status(200)
                .header("content-type", "application/json")
                .body(
                    serde_json::json!([{
                        "transactionID": "txn-1",
                        "transactionHash": "0xabc",
                        "from": ADDRESS,
                        "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
                        "walletAddress": BATCH_WALLET,
                        "data": "0x",
                        "nonce": "0",
                        "value": "0",
                        "state": state,
                        "failureReason": null,
                        "type": "WALLET",
                        "metadata": null,
                        "createdAt": "2026-05-24T18:00:00Z",
                        "updatedAt": "2026-05-24T18:00:00Z"
                    }])
                    .to_string(),
                )
                .build()
        });
    });

    let client = RelayClient::new(&server.base_url(), 137, None, None).expect("valid client");
    let transaction = client
        .poll_until_state(
            "txn-1",
            &[RelayerTransactionState::StateConfirmed],
            Some(RelayerTransactionState::StateFailed),
            Some(5),
            Some(std::time::Duration::from_millis(1_000)),
        )
        .await
        .expect("poll succeeds")
        .expect("transaction should resolve");

    tx_mock.assert_calls(2);
    assert_eq!(transaction.state, "STATE_CONFIRMED");
}

#[tokio::test]
async fn poll_until_state_returns_none_on_fail_state() {
    let server = MockServer::start();
    let tx_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/transaction")
            .query_param("id", "txn-1");
        then.status(200).json_body_obj(&serde_json::json!([{
            "transactionID": "txn-1",
            "transactionHash": "0xabc",
            "from": ADDRESS,
            "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
            "walletAddress": BATCH_WALLET,
            "data": "0x",
            "nonce": "0",
            "value": "0",
            "state": "STATE_FAILED",
            "failureReason": "boom",
            "type": "WALLET",
            "metadata": null,
            "createdAt": "2026-05-24T18:00:00Z",
            "updatedAt": "2026-05-24T18:00:00Z"
        }]));
    });

    let client = RelayClient::new(&server.base_url(), 137, None, None).expect("valid client");
    let transaction = client
        .poll_until_state(
            "txn-1",
            &[RelayerTransactionState::StateConfirmed],
            Some(RelayerTransactionState::StateFailed),
            Some(5),
            Some(std::time::Duration::from_millis(1_000)),
        )
        .await
        .expect("poll succeeds");

    tx_mock.assert();
    assert!(transaction.is_none());
}

#[tokio::test]
async fn poll_until_state_returns_none_on_invalid_state() {
    let server = MockServer::start();
    let tx_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/transaction")
            .query_param("id", "txn-1");
        then.status(200).json_body_obj(&serde_json::json!([{
            "transactionID": "txn-1",
            "transactionHash": "0xabc",
            "from": ADDRESS,
            "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
            "walletAddress": BATCH_WALLET,
            "data": "0x",
            "nonce": "0",
            "value": "0",
            "state": "STATE_INVALID",
            "failureReason": "bad payload",
            "type": "WALLET",
            "metadata": null,
            "createdAt": "2026-05-24T18:00:00Z",
            "updatedAt": "2026-05-24T18:00:00Z"
        }]));
    });

    let client = RelayClient::new(&server.base_url(), 137, None, None).expect("valid client");
    let transaction = client
        .poll_until_state(
            "txn-1",
            &[RelayerTransactionState::StateConfirmed],
            Some(RelayerTransactionState::StateFailed),
            Some(5),
            Some(std::time::Duration::from_millis(1_000)),
        )
        .await
        .expect("poll succeeds");

    tx_mock.assert();
    assert!(transaction.is_none());
}

#[tokio::test]
async fn poll_until_state_does_not_sleep_after_last_poll() {
    let server = MockServer::start();
    let tx_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/transaction")
            .query_param("id", "txn-1");
        then.status(200).json_body_obj(&serde_json::json!([]));
    });

    let client = RelayClient::new(&server.base_url(), 137, None, None).expect("valid client");
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(250),
        client.poll_until_state(
            "txn-1",
            &[RelayerTransactionState::StateConfirmed],
            Some(RelayerTransactionState::StateFailed),
            Some(1),
            Some(std::time::Duration::from_millis(1_000)),
        ),
    )
    .await;

    tx_mock.assert_calls(1);
    assert!(result.is_ok(), "poll_until_state slept after final poll");
    assert!(
        result
            .expect("future completed")
            .expect("poll succeeds")
            .is_none()
    );
}

#[tokio::test]
async fn response_wait_uses_polling_helper() {
    let server = MockServer::start();
    let submit_mock = server.mock(|when, then| {
        when.method(POST).path("/submit");
        then.status(200).json_body_obj(&serde_json::json!({
            "transactionID": "test-txn",
            "state": "STATE_NEW",
            "transactionHash": "0xabc",
            "hash": "0xabc",
        }));
    });
    let tx_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/transaction")
            .query_param("id", "test-txn");
        then.status(200).json_body_obj(&serde_json::json!([{
            "transactionID": "test-txn",
            "transactionHash": "0xabc",
            "from": ADDRESS,
            "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
            "walletAddress": BATCH_WALLET,
            "data": "0x",
            "nonce": "0",
            "value": "0",
            "state": "STATE_CONFIRMED",
            "failureReason": null,
            "type": "WALLET-CREATE",
            "metadata": null,
            "createdAt": "2026-05-24T18:00:00Z",
            "updatedAt": "2026-05-24T18:00:00Z"
        }]));
    });

    let response = client(&server.base_url(), 137)
        .deploy_deposit_wallet()
        .await
        .expect("deploy succeeds");
    let transaction = response.wait().await.expect("wait succeeds");

    submit_mock.assert();
    tx_mock.assert();
    assert_eq!(
        transaction.expect("transaction resolves").state,
        "STATE_CONFIRMED"
    );
}

#[tokio::test]
async fn response_wait_treats_invalid_as_terminal_failure() {
    let server = MockServer::start();
    let submit_mock = server.mock(|when, then| {
        when.method(POST).path("/submit");
        then.status(200).json_body_obj(&serde_json::json!({
            "transactionID": "test-txn",
            "state": "STATE_NEW",
            "transactionHash": "0xabc",
            "hash": "0xabc",
        }));
    });
    let tx_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/transaction")
            .query_param("id", "test-txn");
        then.status(200).json_body_obj(&serde_json::json!([{
            "transactionID": "test-txn",
            "transactionHash": "0xabc",
            "from": ADDRESS,
            "to": "0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c",
            "walletAddress": BATCH_WALLET,
            "data": "0x",
            "nonce": "0",
            "value": "0",
            "state": "STATE_INVALID",
            "failureReason": "bad payload",
            "type": "WALLET-CREATE",
            "metadata": null,
            "createdAt": "2026-05-24T18:00:00Z",
            "updatedAt": "2026-05-24T18:00:00Z"
        }]));
    });

    let response = client(&server.base_url(), 137)
        .deploy_deposit_wallet()
        .await
        .expect("deploy succeeds");
    let transaction = tokio::time::timeout(std::time::Duration::from_millis(250), response.wait())
        .await
        .expect("wait should not spin on STATE_INVALID")
        .expect("wait succeeds");

    submit_mock.assert();
    tx_mock.assert();
    assert!(transaction.is_none());
}
