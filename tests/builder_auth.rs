mod common;

use common::builder_config;
use httpmock::prelude::*;
use kuest_builder_relayer_client::{BuilderApiKeyCreds, BuilderConfig, BuilderType};

#[tokio::test]
async fn local_builder_auth_headers_match_fixed_vector() {
    let headers = builder_config()
        .generate_builder_headers(
            "POST",
            "/order",
            Some(r#"{"deferExec":false,"order":{"salt":718139292476,"maker":"0x6e0c80c90ea6c15917308F820Eac91Ce2724B5b5","signer":"0x6e0c80c90ea6c15917308F820Eac91Ce2724B5b5","tokenId":"15871154585880608648532107628464183779895785213830018178010423617714102767076","makerAmount":"5000000","takerAmount":"10000000","side":"BUY","signatureType":3,"timestamp":"1758744060000","metadata":"0x0000000000000000000000000000000000000000000000000000000000000000","builder":"0x0000000000000000000000000000000000000000000000000000000000000000","signature":"0x64a2b097cf14f9a24403748b4060bedf8f33f3dbe2a38e5f85bc2a5f2b841af633a2afcc9c4d57e60e4ff1d58df2756b2ca469f984ecfd46cb0c8baba8a0d6411b"},"owner":"5d1c266a-ed39-b9bd-c1f5-f24ae3e14a7b","orderType":"GTC","expiration":"0"}"#),
            Some(1_758_744_060),
        )
        .await
        .expect("headers generate");

    assert_eq!(
        headers.kuest_builder_api_key,
        "019894b9-cb40-79c4-b2bd-6aecb6f8c6c5"
    );
    assert_eq!(
        headers.kuest_builder_passphrase,
        "1816e5ed89518467ffa78c65a2d6a62d240f6fd6d159cba7b2c4dc510800f75a"
    );
    assert_eq!(headers.kuest_builder_timestamp, "1758744060");
    assert_eq!(
        headers.kuest_builder_signature,
        "A_Q3AZVfyzcumj4FAptV5QuQNIaTWqKdbutAiUtPHZk="
    );
}

#[tokio::test]
async fn remote_builder_auth_headers_are_forwarded() {
    let server = MockServer::start();
    let remote_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/sign")
            .header("authorization", "Bearer token");
        then.status(200).json_body_obj(&serde_json::json!({
            "KUEST_BUILDER_API_KEY": "test-api-key",
            "KUEST_BUILDER_TIMESTAMP": "1758744060",
            "KUEST_BUILDER_PASSPHRASE": "test-passphrase",
            "KUEST_BUILDER_SIGNATURE": "test-signature",
        }));
    });

    let config = BuilderConfig::remote(
        &format!("{}/sign", server.base_url()),
        Some("token".to_owned()),
    )
    .expect("valid remote config");
    let headers = config
        .generate_builder_headers(
            "POST",
            "/order",
            Some(r#"{"data":"example"}"#),
            Some(1_758_744_060),
        )
        .await
        .expect("headers generate");

    remote_mock.assert();
    assert_eq!(headers.kuest_builder_api_key, "test-api-key");
    assert_eq!(headers.kuest_builder_timestamp, "1758744060");
    assert_eq!(headers.kuest_builder_passphrase, "test-passphrase");
    assert_eq!(headers.kuest_builder_signature, "test-signature");
}

#[test]
fn builder_header_payload_debug_redacts_sensitive_fields() {
    let headers = kuest_builder_relayer_client::BuilderHeaderPayload {
        kuest_builder_api_key: "test-api-key".to_owned(),
        kuest_builder_timestamp: "1758744060".to_owned(),
        kuest_builder_passphrase: "test-passphrase".to_owned(),
        kuest_builder_signature: "test-signature".to_owned(),
    };

    let debug_output = format!("{headers:?}");

    assert!(!debug_output.contains("test-api-key"));
    assert!(!debug_output.contains("test-passphrase"));
    assert!(!debug_output.contains("test-signature"));
    assert!(debug_output.contains("1758744060"));
}

#[test]
fn builder_api_key_creds_debug_redacts_sensitive_fields() {
    let creds = BuilderApiKeyCreds::new("test-api-key", "test-secret", "test-passphrase");

    let debug_output = format!("{creds:?}");

    assert!(!debug_output.contains("test-api-key"));
    assert!(!debug_output.contains("test-secret"));
    assert!(!debug_output.contains("test-passphrase"));
}

#[test]
fn remote_builder_config_debug_redacts_bearer_token() {
    let config = kuest_builder_relayer_client::RemoteBuilderConfig::new(
        "https://example.com/sign",
        Some("super-secret-token".to_owned()),
    )
    .expect("valid remote config");

    let debug_output = format!("{config:?}");

    assert!(debug_output.contains("example.com"));
    assert!(!debug_output.contains("super-secret-token"));
    assert!(debug_output.contains("<redacted>"));
}

#[test]
fn builder_type_prefers_local_when_both_are_present() {
    let config = BuilderConfig::from_parts(
        Some(
            kuest_builder_relayer_client::RemoteBuilderConfig::new(
                "http://localhost:3000/sign",
                None,
            )
            .expect("valid remote"),
        ),
        Some(BuilderApiKeyCreds::new(
            "key",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            "passphrase",
        )),
    )
    .expect("valid config");

    assert_eq!(config.get_builder_type(), BuilderType::Local);
    assert!(config.is_valid());
}
