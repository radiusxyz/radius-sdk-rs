use super::types::*;
use crate::primitives::*;

pub struct Publisher<P: Provider> {
    provider: P,
    validation: ValidationServiceManager::ValidationServiceManagerInstance<P>,
}

impl<P: Provider> Publisher<P> {
    pub fn new(
        provider: P,
        validation: ValidationServiceManager::ValidationServiceManagerInstance<P>,
    ) -> Self {
        Self {
            provider,
            validation,
        }
    }

    pub fn provider(&self) -> &impl Provider {
        &self.provider
    }

    async fn extract_transaction_hash_from_pending_transaction(
        &self,
        pending_transaction: Result<PendingTransactionBuilder<Ethereum>, ContractError>,
    ) -> Result<FixedBytes<32>, TransactionError> {
        let transaction_receipt = pending_transaction
            .map_err(TransactionError::SendTransaction)?
            .get_receipt()
            .await
            .map_err(TransactionError::GetReceipt)?;

        match transaction_receipt.as_ref().is_success() {
            true => Ok(transaction_receipt.transaction_hash),
            false => Err(TransactionError::FailedTransaction(
                transaction_receipt.transaction_hash,
            )),
        }
    }

    pub async fn register_block_commitment(
        &self,
        cluster_id: impl AsRef<str>,
        rollup_id: impl AsRef<str>,
        block_number: u64,
        block_commitment: impl AsRef<[u8]>,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let cluster_id = cluster_id.as_ref().to_owned();
        let rollup_id = rollup_id.as_ref().to_owned();
        let block_number = U256::from(block_number);
        let block_commitment: FixedBytes<32> = {
            let length = block_commitment.as_ref().len();
            if length != 32 {
                return Err(PublisherError::BlockCommitmentLength(length));
            }

            FixedBytes::from_slice(block_commitment.as_ref())
        };

        let transaction =
            self.validation
                .createNewTask(cluster_id, rollup_id, block_number, block_commitment);
        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterBlockCommitment)?;

        Ok(transaction_hash)
    }

    pub async fn respond_to_task(
        &self,
        cluster_id: impl AsRef<str>,
        rollup_id: impl AsRef<str>,
        task_index: u64,
        response: bool,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let rollup_id = rollup_id.as_ref().to_owned();
        let cluster_id = cluster_id.as_ref().to_owned();
        let task_index = task_index as u32;

        let transaction = self
            .validation
            .respondToTask(cluster_id, rollup_id, task_index, response);
        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RespondToTask)?;

        Ok(transaction_hash)
    }
}

#[derive(Debug)]
pub enum TransactionError {
    SendTransaction(alloy::contract::Error),
    GetReceipt(alloy::providers::PendingTransactionError),
    FailedTransaction(FixedBytes<32>),
    EmptyLogs,
    DecodeLogData(alloy::sol_types::Error),
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TransactionError {}

#[derive(Debug)]
pub enum PublisherError {
    ParseEthereumRpcUrl(Box<dyn std::error::Error>),
    ParseSigningKey(LocalSignerError),
    ParseContractAddress(String, alloy::hex::FromHexError),
    BlockCommitmentLength(usize),
    RegisterBlockCommitment(TransactionError),
    RespondToTask(TransactionError),
}

impl std::fmt::Display for PublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PublisherError {}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::Arc, time::Duration};

    use alloy::{
        network::EthereumWallet, providers::ProviderBuilder, signers::local::LocalSigner,
        transports::http::reqwest::Url,
    };
    use tokio::time::sleep;

    use super::{super::subscriber::Subscriber, *};

    async fn callback(event: ValidationServiceManager::NewTaskCreated, _: Arc<()>) {
        println!("clusterId: {:?}", event.clusterId);
        println!("rollupId: {:?}", event.rollupId);
        println!("referenceTaskIndex: {:?}", event.referenceTaskIndex);
        println!("blockNumber: {:?}", event.blockNumber);
        println!("commitment: {:?}", event.blockCommitment);
    }

    fn create_provider() -> impl Provider + Clone {
        let signer = LocalSigner::from_str(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        )
        .map_err(PublisherError::ParseSigningKey)
        .unwrap();

        let wallet = EthereumWallet::new(signer.clone());

        ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(Url::parse("ws://127.0.0.1:8545").unwrap())
    }

    fn create_symbiotic_validation<P: Provider + Clone>(provider: P) -> Publisher<P> {
        let validation_contract_address =
            Address::from_str("0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690").unwrap();
        let validation_contract =
            ValidationServiceManager::new(validation_contract_address, provider.clone());
        Publisher::new(provider, validation_contract)
    }

    #[tokio::test]
    async fn test_register_block_commitment() {
        let provider = create_provider();
        let symbiotic_validation = create_symbiotic_validation(provider);

        let subscriber = Subscriber::new(
            "ws://127.0.0.1:8545",
            "0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690",
        )
        .unwrap();

        tokio::spawn(async move {
            loop {
                subscriber
                    .initialize_event_handler(callback, ().into())
                    .await
                    .unwrap();

                sleep(Duration::from_secs(1)).await;
            }
        });

        symbiotic_validation
            .register_block_commitment("cluster_id", "rollup_id", 0, &[0u8; 32])
            .await
            .unwrap();

        sleep(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn test_respond_to_task() {
        let provider = create_provider();
        let symbiotic_validation = create_symbiotic_validation(provider);
        let rollup_id = "rollup_id".to_owned();
        let cluster_id = "cluster_id".to_owned();
        let block_number = 0;
        let response = true;

        symbiotic_validation
            .respond_to_task(rollup_id, cluster_id, block_number, response)
            .await
            .unwrap();
    }
}
