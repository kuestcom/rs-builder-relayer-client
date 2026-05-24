pub mod auth;
pub mod builder;
pub mod client;
pub mod config;
pub mod constants;
pub mod endpoints;
pub mod error;
mod http;
pub mod response;
pub mod types;

pub use alloy::signers::Signer;
pub use alloy::signers::local::PrivateKeySigner;
pub use auth::{
    BuilderApiKeyCreds, BuilderConfig, BuilderHeaderPayload, BuilderType, RemoteBuilderConfig,
};
pub use builder::{
    build_deposit_wallet_batch_request, build_deposit_wallet_create_request, derive_deposit_wallet,
};
pub use client::{RelayClient, SignerSource};
pub use config::{ContractConfig, DepositWalletContractConfig, get_contract_config};
pub use error::{Error, Result};
pub use response::ClientRelayerTransactionResponse;
pub use types::{
    DepositWalletBatchRequest, DepositWalletCall, DepositWalletCreateRequest, DepositWalletParams,
    DepositWalletTransactionArgs, GetDeployedResponse, NoncePayload, RelayerTransaction,
    RelayerTransactionState, SubmitResponse, TransactionType,
};
