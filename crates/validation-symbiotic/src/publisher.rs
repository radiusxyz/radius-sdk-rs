use std::str::FromStr;

use alloy::{
    contract,
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, PendingTransactionBuilder, ProviderBuilder, RootProvider, WalletProvider,
    },
    signers::local::LocalSigner,
    transports::http::{reqwest::Url, Client, Http},
};

use crate::types::*;

type EthereumHttpProvider = FillProvider<
    JoinFill<
        JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

type ValidationContract = ValidationServiceManager::ValidationServiceManagerInstance<
    Http<Client>,
    FillProvider<
        JoinFill<
            JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider<Http<Client>>,
        Http<Client>,
        Ethereum,
    >,
>;

pub struct Publisher {
    provider: EthereumHttpProvider,
    validation_contract: ValidationContract,
}

impl Publisher {
    pub fn new(
        ethereum_rpc_url: impl AsRef<str>,
        signing_key: impl AsRef<str>,
        validation_contract_address: impl AsRef<str>,
    ) -> Result<Self, PublisherError> {
        let rpc_url: Url = ethereum_rpc_url
            .as_ref()
            .parse()
            .map_err(|error| PublisherError::ParseEthereumRpcUrl(Box::new(error)))?;

        let signer =
            LocalSigner::from_str(signing_key.as_ref()).map_err(PublisherError::ParseSigningKey)?;

        let wallet = EthereumWallet::new(signer.clone());

        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(rpc_url);

        let validation_contract_address = Address::from_str(validation_contract_address.as_ref())
            .map_err(|error| {
            PublisherError::ParseContractAddress(
                validation_contract_address.as_ref().to_owned(),
                error,
            )
        })?;
        let validation_contract =
            ValidationServiceManager::new(validation_contract_address, provider.clone());

        Ok(Self {
            provider,
            validation_contract,
        })
    }

    pub fn address(&self) -> Address {
        self.provider.default_signer_address()
    }

    async fn extract_transaction_hash_from_pending_transaction<'a>(
        &'a self,
        pending_transaction: Result<
            PendingTransactionBuilder<'a, Http<Client>, Ethereum>,
            contract::Error,
        >,
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
        block_commitment: impl AsRef<[u8]>,
        block_number: u64,
        rollup_id: impl AsRef<str>,
        cluster_id: impl AsRef<str>,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let block_commitment = Bytes::from_iter(block_commitment.as_ref());
        let rollup_id = rollup_id.as_ref().to_owned();
        let cluster_id = cluster_id.as_ref().to_owned();

        let transaction = self.validation_contract.createNewTask(
            block_commitment,
            block_number,
            rollup_id,
            cluster_id,
        );

        let pending_transaction = transaction.send().await;

        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterBlockCommitment)?;

        Ok(transaction_hash)
    }

    pub async fn respond_to_task(
        &self,
        task: ValidationServiceManager::Task,
        task_index: u32,
        block_commitment: impl AsRef<[u8]>,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let block_commitment = Bytes::from_iter(block_commitment.as_ref());

        let transaction =
            self.validation_contract
                .respondToTask(task, task_index, block_commitment);

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
    GetReceipt(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
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
    ParseSigningKey(alloy::signers::local::LocalSignerError),
    ParseContractAddress(String, alloy::hex::FromHexError),
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
    use std::{sync::Arc, time::Duration};

    use tokio::time::sleep;

    use super::*;
    use crate::subscriber::Subscriber;

    async fn callback(event: ValidationServiceManager::NewTaskCreated, _: Arc<()>) {
        println!("commitment: {:?}", event.commitment);
        println!("blockNumber: {:?}", event.blockNumber);
        println!("rollupId: {:?}", event.rollupId);
        println!("clusterId: {:?}", event.clusterId);
        println!("taskCreatedBlock: {:?}", event.taskCreatedBlock);

        println!("taskIndex: {:?}", event.taskIndex);
    }

    #[tokio::test]
    async fn test_register_block_commitment() {
        let publisher = Publisher::new(
            "http://127.0.0.1:8545",
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            "0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690",
        )
        .unwrap();

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

        publisher
            .register_block_commitment(&[0u8; 32], 0, "rollup_id", "cluster_id")
            .await
            .unwrap();

        sleep(Duration::from_secs(5)).await;
    }

    #[tokio::test]
    async fn test_respond_to_task() {
        let publisher = Publisher::new(
            "http://127.0.0.1:8545",
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            "0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690",
        )
        .unwrap();

        let task = ValidationServiceManager::Task {
            commitment: Bytes::from_iter(&[0u8; 32]),
            blockNumber: 0,
            rollupId: "rollup_id".to_owned(),
            clusterId: "cluster_id".to_owned(),
            taskCreatedBlock: 20,
        };

        publisher
            .respond_to_task(task, 0, Bytes::from_iter(&[0u8; 64]))
            .await
            .unwrap();
    }
}
