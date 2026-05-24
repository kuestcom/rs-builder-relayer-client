use reqwest::StatusCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid chainID: {0}")]
    InvalidChainId(u64),

    #[error("contract config unsupported on chain")]
    UnsupportedContractConfig,

    #[error("signer unavailable")]
    SignerUnavailable,

    #[error("builder credentials are required for this endpoint")]
    BuilderCredentialsUnavailable,

    #[error("invalid local builder credentials")]
    InvalidLocalBuilderCredentials,

    #[error("invalid remote url")]
    InvalidRemoteUrl,

    #[error("invalid auth token")]
    InvalidAuthToken,

    #[error("invalid nonce payload received")]
    InvalidNoncePayload,

    #[error("no deposit wallet calls to execute")]
    EmptyDepositWalletCalls,

    #[error("invalid {field}: {value}")]
    InvalidAddress { field: &'static str, value: String },

    #[error("invalid {field}: {value}")]
    InvalidHex { field: &'static str, value: String },

    #[error("invalid {field}: {value}")]
    InvalidInteger { field: &'static str, value: String },

    #[error("builder secret is not valid base64")]
    InvalidBuilderSecretEncoding(#[from] base64::DecodeError),

    #[error("failed to initialize HMAC signer")]
    InvalidHmacKeyLength(#[from] hmac::digest::InvalidLength),

    #[error("invalid header value")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("request failed with status {status}: {body}")]
    Api { status: StatusCode, body: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    AlloySigner(#[from] alloy::signers::Error),
}
