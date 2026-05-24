use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "WALLET")]
    Wallet,
    #[serde(rename = "WALLET-CREATE", alias = "WALLET_CREATE")]
    WalletCreate,
}

impl TransactionType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wallet => "WALLET",
            Self::WalletCreate => "WALLET-CREATE",
        }
    }
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RelayerTransactionState {
    #[serde(rename = "STATE_NEW")]
    StateNew,
    #[serde(rename = "STATE_EXECUTED")]
    StateExecuted,
    #[serde(rename = "STATE_MINED")]
    StateMined,
    #[serde(rename = "STATE_INVALID")]
    StateInvalid,
    #[serde(rename = "STATE_CONFIRMED")]
    StateConfirmed,
    #[serde(rename = "STATE_FAILED")]
    StateFailed,
}

impl RelayerTransactionState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StateNew => "STATE_NEW",
            Self::StateExecuted => "STATE_EXECUTED",
            Self::StateMined => "STATE_MINED",
            Self::StateInvalid => "STATE_INVALID",
            Self::StateConfirmed => "STATE_CONFIRMED",
            Self::StateFailed => "STATE_FAILED",
        }
    }
}

impl std::fmt::Display for RelayerTransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NoncePayload {
    pub nonce: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GetDeployedResponse {
    pub deployed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DepositWalletCall {
    pub target: String,
    pub value: String,
    pub data: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositWalletTransactionArgs {
    pub from: String,
    pub chain_id: u64,
    pub wallet_address: String,
    pub nonce: String,
    pub deadline: String,
    pub calls: Vec<DepositWalletCall>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositWalletParams {
    pub deposit_wallet: String,
    pub deadline: String,
    pub calls: Vec<DepositWalletCall>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DepositWalletBatchRequest {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub from: String,
    pub to: String,
    pub nonce: String,
    pub signature: String,
    #[serde(rename = "depositWalletParams")]
    pub deposit_wallet_params: DepositWalletParams,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DepositWalletCreateRequest {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelayerTransaction {
    #[serde(rename = "transactionID")]
    pub transaction_id: String,
    #[serde(
        rename = "transactionHash",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transaction_hash: Option<String>,
    pub from: String,
    pub to: String,
    #[serde(
        rename = "walletAddress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wallet_address: Option<String>,
    pub data: String,
    pub nonce: String,
    pub value: String,
    pub state: String,
    #[serde(
        rename = "failureReason",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub failure_reason: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubmitResponse {
    #[serde(rename = "transactionID")]
    pub transaction_id: String,
    pub state: String,
    #[serde(
        rename = "transactionHash",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transaction_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}
