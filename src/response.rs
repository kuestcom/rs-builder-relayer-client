use crate::client::RelayClient;
use crate::error::Result;
use crate::types::{RelayerTransaction, RelayerTransactionState};

#[derive(Clone, Debug)]
pub struct ClientRelayerTransactionResponse {
    pub transaction_id: String,
    pub state: String,
    pub transaction_hash: Option<String>,
    pub hash: Option<String>,
    client: RelayClient,
}

impl ClientRelayerTransactionResponse {
    pub(crate) fn new(
        transaction_id: String,
        state: String,
        transaction_hash: Option<String>,
        hash: Option<String>,
        client: RelayClient,
    ) -> Self {
        Self {
            transaction_id,
            state,
            transaction_hash,
            hash,
            client,
        }
    }

    pub async fn get_transaction(&self) -> Result<Vec<RelayerTransaction>> {
        self.client.get_transaction(&self.transaction_id).await
    }

    pub async fn wait(&self) -> Result<Option<RelayerTransaction>> {
        self.client
            .poll_until_state(
                &self.transaction_id,
                &[
                    RelayerTransactionState::StateMined,
                    RelayerTransactionState::StateConfirmed,
                ],
                Some(RelayerTransactionState::StateFailed),
                Some(100),
                None,
            )
            .await
    }
}
