use alloy::primitives::Signature;
use alloy::signers::Signer as AlloySigner;
use std::fmt;
use std::str::FromStr as _;
use std::sync::Arc;
use std::time::Duration;

use crate::auth::BuilderConfig;
use crate::builder::{
    build_deposit_wallet_batch_request, build_deposit_wallet_create_request, derive_deposit_wallet,
};
use crate::config::{ContractConfig, get_contract_config, is_deposit_wallet_contract_config_valid};
use crate::endpoints::{
    GET_DEPLOYED, GET_NONCE, GET_TRANSACTION, GET_TRANSACTIONS, SUBMIT_PUBLIC_WALLET_TRANSACTION,
    SUBMIT_TRANSACTION,
};
use crate::error::{Error, Result};
use crate::http::HttpClient;
use crate::response::ClientRelayerTransactionResponse;
use crate::types::{
    DepositWalletCall, DepositWalletTransactionArgs, GetDeployedResponse, NoncePayload,
    RelayerTransaction, RelayerTransactionState, SubmitResponse, TransactionType,
};

#[derive(Clone)]
pub enum SignerSource {
    PrivateKey(String),
    Signer(Arc<dyn AlloySigner<Signature> + Send + Sync>),
}

impl fmt::Debug for SignerSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrivateKey(_) => f.write_str("SignerSource::PrivateKey(<redacted>)"),
            Self::Signer(_) => f.write_str("SignerSource::Signer(<dyn signer>)"),
        }
    }
}

impl SignerSource {
    #[must_use]
    pub fn from_private_key(private_key: impl Into<String>) -> Self {
        Self::PrivateKey(private_key.into())
    }

    pub fn from_signer<S>(signer: S) -> Self
    where
        S: AlloySigner<Signature> + Send + Sync + 'static,
    {
        Self::Signer(Arc::new(signer))
    }

    fn into_arc_signer(self) -> Result<Arc<dyn AlloySigner<Signature> + Send + Sync>> {
        match self {
            Self::PrivateKey(private_key) => {
                let signer = crate::PrivateKeySigner::from_str(&private_key)
                    .map_err(|err| Error::InvalidPrivateKey(err.to_string()))?;
                Ok(Arc::new(signer))
            }
            Self::Signer(signer) => Ok(signer),
        }
    }
}

#[derive(Clone)]
pub struct RelayClient {
    relayer_url: String,
    chain_id: u64,
    contract_config: ContractConfig,
    http_client: HttpClient,
    signer: Option<Arc<dyn AlloySigner<Signature> + Send + Sync>>,
    builder_config: Option<BuilderConfig>,
}

impl fmt::Debug for RelayClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RelayClient")
            .field("relayer_url", &self.relayer_url)
            .field("chain_id", &self.chain_id)
            .field("contract_config", &self.contract_config)
            .field("has_signer", &self.signer.is_some())
            .field("has_builder_config", &self.builder_config.is_some())
            .finish_non_exhaustive()
    }
}

impl RelayClient {
    pub fn new(
        relayer_url: &str,
        chain_id: u64,
        signer: Option<SignerSource>,
        builder_config: Option<BuilderConfig>,
    ) -> Result<Self> {
        Ok(Self {
            relayer_url: normalize_relayer_url(relayer_url),
            chain_id,
            contract_config: get_contract_config(chain_id)?,
            http_client: HttpClient::new(),
            signer: signer.map(SignerSource::into_arc_signer).transpose()?,
            builder_config,
        })
    }

    pub fn new_with_private_key(
        relayer_url: &str,
        chain_id: u64,
        private_key: &str,
        builder_config: Option<BuilderConfig>,
    ) -> Result<Self> {
        Self::new(
            relayer_url,
            chain_id,
            Some(SignerSource::from_private_key(private_key)),
            builder_config,
        )
    }

    pub fn new_with_signer<S>(
        relayer_url: &str,
        chain_id: u64,
        signer: S,
        builder_config: Option<BuilderConfig>,
    ) -> Result<Self>
    where
        S: AlloySigner<Signature> + Send + Sync + 'static,
    {
        Self::new(
            relayer_url,
            chain_id,
            Some(SignerSource::from_signer(signer)),
            builder_config,
        )
    }

    #[must_use]
    pub fn relayer_url(&self) -> &str {
        &self.relayer_url
    }

    #[must_use]
    pub const fn chain_id(&self) -> u64 {
        self.chain_id
    }

    #[must_use]
    pub fn contract_config(&self) -> &ContractConfig {
        &self.contract_config
    }

    pub fn set_signer(&mut self, signer: SignerSource) -> Result<()> {
        self.signer = Some(signer.into_arc_signer()?);
        Ok(())
    }

    pub fn set_builder_config(&mut self, builder_config: BuilderConfig) {
        self.builder_config = Some(builder_config);
    }

    pub async fn get_nonce(
        &self,
        signer_address: &str,
        signer_type: Option<TransactionType>,
    ) -> Result<NoncePayload> {
        let tx_type = signer_type.unwrap_or(TransactionType::Wallet).to_string();
        self.http_client
            .send_json(
                &format!("{}{}", self.relayer_url, GET_NONCE),
                reqwest::Method::GET,
                None,
                Some(&[("address", signer_address), ("type", tx_type.as_str())]),
                None,
            )
            .await
    }

    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Vec<RelayerTransaction>> {
        self.http_client
            .send_json(
                &format!("{}{}", self.relayer_url, GET_TRANSACTION),
                reqwest::Method::GET,
                None,
                Some(&[("id", transaction_id)]),
                None,
            )
            .await
    }

    pub async fn get_transactions(&self) -> Result<Vec<RelayerTransaction>> {
        self.builder_creds_needed()?;
        self.send_authed_request::<Vec<RelayerTransaction>>(
            reqwest::Method::GET,
            GET_TRANSACTIONS,
            None,
        )
        .await
    }

    pub async fn get_deployed(&self, address: &str) -> Result<bool> {
        let response = self
            .http_client
            .send_json::<GetDeployedResponse>(
                &format!("{}{}", self.relayer_url, GET_DEPLOYED),
                reqwest::Method::GET,
                None,
                Some(&[("address", address)]),
                None,
            )
            .await?;
        Ok(response.deployed)
    }

    pub fn derive_deposit_wallet(&self) -> Result<String> {
        self.signer_needed()?;
        let config = &self.contract_config.deposit_wallet_contracts;
        if !is_deposit_wallet_contract_config_valid(config) {
            return Err(Error::UnsupportedContractConfig);
        }

        derive_deposit_wallet(
            &self.signer_address()?,
            &config.factory_string(),
            &config.implementation_string(),
        )
    }

    pub fn derive_deposit_wallet_address(&self) -> Result<String> {
        self.derive_deposit_wallet()
    }

    pub fn get_expected_deposit_wallet(&self) -> Result<String> {
        self.derive_deposit_wallet()
    }

    pub async fn deploy_deposit_wallet(&self) -> Result<ClientRelayerTransactionResponse> {
        self.signer_needed()?;
        self.builder_creds_needed()?;

        let from = self.signer_address()?;
        let config = &self.contract_config.deposit_wallet_contracts;
        if !is_deposit_wallet_contract_config_valid(config) {
            return Err(Error::UnsupportedContractConfig);
        }

        let request = build_deposit_wallet_create_request(&from, config);
        self.submit_transaction(SUBMIT_TRANSACTION, &request, true)
            .await
    }

    pub async fn deploy_deposit_wallet_public(&self) -> Result<ClientRelayerTransactionResponse> {
        self.signer_needed()?;

        let from = self.signer_address()?;
        let config = &self.contract_config.deposit_wallet_contracts;
        if !is_deposit_wallet_contract_config_valid(config) {
            return Err(Error::UnsupportedContractConfig);
        }

        let request = build_deposit_wallet_create_request(&from, config);
        self.submit_transaction(SUBMIT_PUBLIC_WALLET_TRANSACTION, &request, false)
            .await
    }

    pub async fn execute_deposit_wallet_batch(
        &self,
        calls: &[DepositWalletCall],
        wallet_address: &str,
        deadline: &str,
    ) -> Result<ClientRelayerTransactionResponse> {
        self.signer_needed()?;
        self.builder_creds_needed()?;
        self.execute_deposit_wallet_batch_inner(
            calls,
            wallet_address,
            deadline,
            SUBMIT_TRANSACTION,
            true,
        )
        .await
    }

    pub async fn execute_deposit_wallet_batch_public(
        &self,
        calls: &[DepositWalletCall],
        wallet_address: &str,
        deadline: &str,
    ) -> Result<ClientRelayerTransactionResponse> {
        self.signer_needed()?;
        self.execute_deposit_wallet_batch_inner(
            calls,
            wallet_address,
            deadline,
            SUBMIT_PUBLIC_WALLET_TRANSACTION,
            false,
        )
        .await
    }

    pub async fn poll_until_state(
        &self,
        transaction_id: &str,
        states: &[RelayerTransactionState],
        fail_state: Option<RelayerTransactionState>,
        max_polls: Option<usize>,
        poll_frequency: Option<Duration>,
    ) -> Result<Option<RelayerTransaction>> {
        let poll_limit = max_polls.unwrap_or(10);
        let poll_frequency = poll_frequency
            .unwrap_or_else(|| Duration::from_secs(2))
            .max(Duration::from_secs(1));

        for _ in 0..poll_limit {
            let transactions = self.get_transaction(transaction_id).await?;
            if let Some(transaction) = transactions.into_iter().next() {
                if states
                    .iter()
                    .any(|state| transaction.state == state.as_str())
                {
                    return Ok(Some(transaction));
                }
                if let Some(fail_state) = fail_state
                    && transaction.state == fail_state.as_str()
                {
                    return Ok(None);
                }
            }

            tokio::time::sleep(poll_frequency).await;
        }

        Ok(None)
    }

    async fn execute_deposit_wallet_batch_inner(
        &self,
        calls: &[DepositWalletCall],
        wallet_address: &str,
        deadline: &str,
        endpoint: &str,
        authed: bool,
    ) -> Result<ClientRelayerTransactionResponse> {
        if calls.is_empty() {
            return Err(Error::EmptyDepositWalletCalls);
        }

        let from = self.signer_address()?;
        let config = &self.contract_config.deposit_wallet_contracts;
        if !is_deposit_wallet_contract_config_valid(config) {
            return Err(Error::UnsupportedContractConfig);
        }

        let nonce_payload = self.get_nonce(&from, Some(TransactionType::Wallet)).await?;
        if nonce_payload.nonce.trim().is_empty() {
            return Err(Error::InvalidNoncePayload);
        }

        let request = build_deposit_wallet_batch_request(
            self.signer.as_deref().ok_or(Error::SignerUnavailable)?,
            &DepositWalletTransactionArgs {
                from,
                chain_id: self.chain_id,
                wallet_address: wallet_address.to_owned(),
                nonce: nonce_payload.nonce,
                deadline: deadline.to_owned(),
                calls: calls.to_vec(),
            },
            config,
        )
        .await?;

        self.submit_transaction(endpoint, &request, authed).await
    }

    async fn submit_transaction<T: serde::Serialize>(
        &self,
        endpoint: &str,
        request: &T,
        authed: bool,
    ) -> Result<ClientRelayerTransactionResponse> {
        let body = serde_json::to_string(request)?;
        let response = if authed {
            self.send_authed_request::<SubmitResponse>(reqwest::Method::POST, endpoint, Some(body))
                .await?
        } else {
            self.http_client
                .send_json(
                    &format!("{}{}", self.relayer_url, endpoint),
                    reqwest::Method::POST,
                    None,
                    None,
                    Some(body),
                )
                .await?
        };

        Ok(ClientRelayerTransactionResponse::new(
            response.transaction_id,
            response.state,
            response.transaction_hash.clone(),
            response.hash.or(response.transaction_hash),
            self.clone(),
        ))
    }

    async fn send_authed_request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<String>,
    ) -> Result<T> {
        let builder_config = self
            .builder_config
            .as_ref()
            .ok_or(Error::BuilderCredentialsUnavailable)?;
        let builder_headers = builder_config
            .generate_builder_headers(method.as_str(), path, body.as_deref(), None)
            .await?
            .to_header_map()?;

        self.http_client
            .send_json(
                &format!("{}{}", self.relayer_url, path),
                method,
                Some(builder_headers),
                None,
                body,
            )
            .await
    }

    fn signer_needed(&self) -> Result<()> {
        if self.signer.is_none() {
            Err(Error::SignerUnavailable)
        } else {
            Ok(())
        }
    }

    fn builder_creds_needed(&self) -> Result<()> {
        if self
            .builder_config
            .as_ref()
            .is_some_and(BuilderConfig::is_valid)
        {
            Ok(())
        } else {
            Err(Error::BuilderCredentialsUnavailable)
        }
    }

    fn signer_address(&self) -> Result<String> {
        let signer = self.signer.as_ref().ok_or(Error::SignerUnavailable)?;
        Ok(signer.address().to_string())
    }
}

fn normalize_relayer_url(relayer_url: &str) -> String {
    relayer_url.trim_end_matches('/').to_owned()
}
