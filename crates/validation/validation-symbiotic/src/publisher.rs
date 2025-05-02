use std::str::FromStr;

use alloy::{
    contract,
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
        Identity, PendingTransactionBuilder, ProviderBuilder, RootProvider, WalletProvider,
    },
    signers::local::LocalSigner,
    transports::http::{reqwest::Url, Client, Http},
};

use crate::types::*;

type EthereumHttpProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
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
            JoinFill<
                Identity,
                JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
            >,
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

    async fn extract_transaction_hash_from_pending_transaction(
        &self,
        pending_transaction: Result<
            PendingTransactionBuilder<Http<Client>, Ethereum>,
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

    pub async fn register_batch_commitment(
        &self,
        cluster_id: impl AsRef<str>,
        rollup_id: impl AsRef<str>,
        batch_number: u64,
        batch_commitment: impl AsRef<[u8]>,

        reference_task_index: u64,
        vault_address_list: Vec<impl AsRef<str>>,
        operator_merkle_root_list: Vec<[u8; 32]>,
        total_staker_reward_list: Vec<u64>,
        total_operator_reward_list: Vec<u64>,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let cluster_id = cluster_id.as_ref().to_owned();
        let rollup_id = rollup_id.as_ref().to_owned();
        let batch_number = U256::from(batch_number);
        let pending_reward_task_index = U256::from(reference_task_index);

        let vault_address_list = vault_address_list
            .iter()
            .map(|address| Address::from_str(address.as_ref()).unwrap())
            .collect::<Vec<Address>>();

        let operator_merkle_root_list = operator_merkle_root_list
            .iter()
            .map(|root| FixedBytes::from_slice(root))
            .collect::<Vec<FixedBytes<32>>>();
        let total_staker_reward_list = total_staker_reward_list
            .iter()
            .map(|reward| U256::from(*reward))
            .collect::<Vec<U256>>();
        let total_operator_reward_list = total_operator_reward_list
            .iter()
            .map(|reward| U256::from(*reward))
            .collect::<Vec<U256>>();

        let batch_commitment: FixedBytes<32> = {
            let length = batch_commitment.as_ref().len();
            if length != 32 {
                return Err(PublisherError::BlockCommitmentLength(length));
            }

            FixedBytes::from_slice(batch_commitment.as_ref())
        };

        let task_params: IValidationServiceManager::Task = IValidationServiceManager::Task {
            clusterId: cluster_id,
            rollupId: rollup_id,
            batchNumber: batch_number,
            batchCommitment: batch_commitment,
        };

        let distribution_params: IValidationServiceManager::DistributionParams =
            IValidationServiceManager::DistributionParams {
                pendingRewardTaskIndex: pending_reward_task_index,
                vaultAddresses: vault_address_list,
                operatorMerkleRoots: operator_merkle_root_list,
                totalStakerReward: total_staker_reward_list,
                totalOperatorReward: total_operator_reward_list,
            };

        let transaction = self
            .validation_contract
            .createNewTask(task_params, distribution_params);

        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterBlockCommitment)?;

        Ok(transaction_hash)
    }

    pub async fn get_task_manager_contract_address(&self) -> Result<Address, PublisherError> {
        let task_manager_contract_address = self
            .validation_contract
            .taskManager()
            .call()
            .await
            .map_err(PublisherError::GetTaskManager)?
            ._0;

        Ok(task_manager_contract_address)
    }

    pub async fn respond_to_task(
        &self,
        cluster_id: impl AsRef<str>,
        rollup_id: impl AsRef<str>,
        reference_task_index: u64,
        response: bool,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let rollup_id = rollup_id.as_ref().to_owned();
        let cluster_id = cluster_id.as_ref().to_owned();
        let reference_task_index = U256::from(reference_task_index);

        let transaction = self
            .validation_contract
            .respondToTask(cluster_id, rollup_id, reference_task_index, response)
            .gas(3_000_000);

        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RespondToTask)?;

        Ok(transaction_hash)
    }

    pub async fn get_distribution_data(
        &self,
        cluster_id: impl AsRef<str>,
        rollup_id: impl AsRef<str>,
        reward_task_id: u64,
    ) -> Result<(Vec<Address>, Vec<[u8; 32]>, Vec<u64>, Vec<u64>), PublisherError> {
        let cluster_id = cluster_id.as_ref().to_owned();
        let rollup_id = rollup_id.as_ref().to_owned();
        let reward_task_id = U256::from(reward_task_id);

        let result: ValidationServiceManager::getDistributionDataReturn = self
            .validation_contract
            .getDistributionData(cluster_id, rollup_id, reward_task_id)
            .call()
            .await
            .map_err(PublisherError::GetDistributionData)?;

        let vault_address_list = result.vaultAddresses.clone();
        let operator_merkle_root_list: Vec<[u8; 32]> = result
            .operatorMerkleRoots
            .iter()
            .map(|root| root.0)
            .collect();

        let total_staker_reward_list: Vec<u64> = result
            .totalStakerReward
            .iter()
            .map(|reward| reward.to::<u64>())
            .collect();

        let total_operator_reward_list: Vec<u64> = result
            .totalOperatorReward
            .iter()
            .map(|reward| reward.to::<u64>())
            .collect();

        Ok((
            vault_address_list,
            operator_merkle_root_list,
            total_staker_reward_list,
            total_operator_reward_list,
        ))
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
    ParseSigningKey(alloy::signers::local::LocalSignerError),
    ParseContractAddress(String, alloy::hex::FromHexError),
    BlockCommitmentLength(usize),
    RegisterBlockCommitment(TransactionError),
    RespondToTask(TransactionError),
    GetDistributionData(alloy::contract::Error),
    GetTaskManager(alloy::contract::Error),
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

    // async fn callback(event: ValidationServiceManager::NewTaskCreated, _:
    // Arc<()>) {     println!("clusterId: {:?}", event.clusterId);
    //     println!("rollupId: {:?}", event.rollupId);
    //     println!("referenceTaskIndex: {:?}", event.referenceTaskIndex);
    //     println!("batchNumber: {:?}", event.batchNumber);
    //     println!("commitment: {:?}", event.batchCommitment);
    //     println!("taskCreatedBlock: {:?}", event.taskCreatedBlock);
    // }

    // #[tokio::test]
    // async fn test_register_batch_commitment() {
    //     let publisher = Publisher::new(
    //         "http://127.0.0.1:8545",
    //         "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    //         "0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690",
    //     )
    //     .unwrap();

    //     let subscriber = Subscriber::new(
    //         "ws://127.0.0.1:8545",
    //         "0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690",
    //     )
    //     .unwrap();

    //     tokio::spawn(async move {
    //         loop {
    //             subscriber
    //                 .initialize_event_handler(callback, ().into())
    //                 .await
    //                 .unwrap();

    //             sleep(Duration::from_secs(1)).await;
    //         }
    //     });

    //     publisher
    //         .register_batch_commitment("cluster_id", "rollup_id", 0, &[0u8; 32])
    //         .await
    //         .unwrap();

    //     sleep(Duration::from_secs(5)).await;
    // }

    // #[tokio::test]
    // async fn test_respond_to_task() {
    //     let publisher = Publisher::new(
    //         "http://127.0.0.1:8545",
    //         "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    //         "0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690",
    //     )
    //     .unwrap();

    //     let rollup_id = "rollup_id".to_owned();
    //     let cluster_id = "cluster_id".to_owned();
    //     let batch_number = 0;
    //     let response = true;

    //     publisher
    //         .respond_to_task(rollup_id, cluster_id, batch_number, response)
    //         .await
    //         .unwrap();
    // }

    #[tokio::test]
    async fn test_get_distribution_data() {
        let publisher = Publisher::new(
            "https://ethereum-holesky-rpc.publicnode.com",
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
            "0x71924dA7C61009e3B2Dac0247881d2727D7328D5",
        )
        .unwrap();

        let cluster_id = "radius".to_owned();
        let rollup_id = "rollup_id_2".to_owned();
        let task_index = 10;

        publisher
            .get_distribution_data(cluster_id, rollup_id, task_index)
            .await
            .unwrap();
    }
}
