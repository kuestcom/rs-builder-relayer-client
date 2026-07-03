<h1 align="center">
  <img src="https://github.com/user-attachments/assets/0cc687fb-89c4-43fa-a056-d89c307215ad" alt="Kuest" height="96" /><br/>
  Kuest Rust Builder Relayer Client
</h1>

Wallet-only Rust client for the Kuest relayer.

## Install

```toml
[dependencies]
kuest-builder-relayer-client = "0.1.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Supported Chains

- `137` Polygon mainnet
- `80002` Polygon Amoy

Both chains use the same Deposit Wallet contracts:

- Factory: `0x2CcdC6C5dDcd895aFcCD259F291de9b618A5cA6c`
- Beacon: `0x74a618eBdd62Ff8579A8FE94f5B888d7623b9C35`
- Implementation: `0xD5D8CdF42DE6AaE41291E41788e5767a137751C7`

## Environment

- `RELAYER_URL`
- `KUEST_BUILDER_API_KEY`
- `KUEST_BUILDER_SECRET`
- `KUEST_BUILDER_PASSPHRASE`

## Usage

```rust
use kuest_builder_relayer_client::{
    BuilderApiKeyCreds, BuilderConfig, DepositWalletCall, RelayClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder_config = BuilderConfig::local(BuilderApiKeyCreds::new(
        std::env::var("KUEST_BUILDER_API_KEY")?,
        std::env::var("KUEST_BUILDER_SECRET")?,
        std::env::var("KUEST_BUILDER_PASSPHRASE")?,
    ))?;

    let client = RelayClient::new_with_private_key(
        &std::env::var("RELAYER_URL")?,
        80002,
        "0xabc123...",
        Some(builder_config),
    )?;

    let wallet = client.derive_deposit_wallet()?;
    client.deploy_deposit_wallet().await?;
    client
        .execute_deposit_wallet_batch(
            &[DepositWalletCall {
                target: "0x0000000000000000000000000000000000000001".to_owned(),
                value: "0".to_owned(),
                data: "0x".to_owned(),
            }],
            &wallet,
            "1735689600",
        )
        .await?;

    Ok(())
}
```

## Public API

- `get_nonce(address, tx_type)`
- `get_transaction(transaction_id)`
- `get_transactions()`
- `get_deployed(address)`
- `derive_deposit_wallet()`
- `derive_deposit_wallet_address()`
- `get_expected_deposit_wallet()`
- `deploy_deposit_wallet()`
- `deploy_deposit_wallet_public()`
- `execute_deposit_wallet_batch(calls, wallet_address, deadline)`
- `execute_deposit_wallet_batch_public(calls, wallet_address, deadline)`
- `poll_until_state(transaction_id, states, fail_state, max_polls, poll_frequency)`

## Builder Auth

Local builder auth mirrors the existing SDKs and signs:

```text
timestamp + method + path + body
```

The generated headers remain:

- `KUEST_BUILDER_API_KEY`
- `KUEST_BUILDER_PASSPHRASE`
- `KUEST_BUILDER_TIMESTAMP`
- `KUEST_BUILDER_SIGNATURE`

Remote signing mode is also supported through `BuilderConfig::remote(...)`.

## Security Notes

- Secrets use `secrecy::SecretString` and are redacted in debug output.
- The crate does not log raw builder headers, signatures, or secrets.
