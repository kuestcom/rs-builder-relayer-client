# Changelog

## 0.1.4

- Removed `deploy_deposit_wallet_public`; wallet deployment now requires authenticated `/submit`.

## 0.1.0

- Added a Rust Deposit Wallet relayer client with parity-focused public APIs.
- Added local and remote builder auth support using `KUEST_BUILDER_*` headers.
- Added CREATE2 wallet derivation, EIP-712 batch signing, mocked HTTP tests, CI, and docs.
