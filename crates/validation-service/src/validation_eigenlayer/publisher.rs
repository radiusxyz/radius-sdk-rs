use chrono::Utc;

use super::contracts::*;
use crate::primitives::*;

pub struct Publisher<P: Provider> {
    provider: P,
    signer: LocalSigner<SigningKey>,
    delegation_manager: DelegationManager::DelegationManagerInstance<P>,
    avs_directory: AVSDirectory::AVSDirectoryInstance<P>,
    ecdsa_stake_registry: EcdsaStakeRegistry::EcdsaStakeRegistryInstance<P>,
    avs: Avs::AvsInstance<P>,
}

impl<P: Provider> Publisher<P> {
    /// Create a new [`Publisher`] instance to call contract functions and send
    /// transactions.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
    ///     "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
    ///     "0xa82fF9aFd8f496c3d6ac40E2a0F282E47488CFc9",
    ///     "0x9E545E3C0baAB3E08CdfD552C960A1050f373042",
    /// )
    /// .unwrap();
    /// ```
    pub fn new(
        signing_key: impl AsRef<str>,
        provider: P,
        delegation_manager: DelegationManager::DelegationManagerInstance<P>,
        avs_directory: AVSDirectory::AVSDirectoryInstance<P>,
        ecdsa_stake_registry: EcdsaStakeRegistry::EcdsaStakeRegistryInstance<P>,
        avs: Avs::AvsInstance<P>,
    ) -> Result<Self, PublisherError> {
        let signer =
            LocalSigner::from_str(signing_key.as_ref()).map_err(PublisherError::ParseSigningKey)?;
        Ok(Self {
            provider,
            signer,
            delegation_manager,
            avs_directory,
            ecdsa_stake_registry,
            avs,
        })
    }

    pub fn provider(&self) -> &impl Provider {
        &self.provider
    }

    fn signer(&self) -> &LocalSigner<SigningKey> {
        &self.signer
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

    /// Return `true` if `self` is registered as an EigenLayer operator.
    pub async fn is_operator(&self, who: Address) -> Result<bool, PublisherError> {
        self.delegation_manager
            .isOperator(who)
            .call()
            .await
            .map_err(PublisherError::IsOperator)
    }

    /// Register `self` as an EigenLayer operator.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
    ///     "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
    ///     "0xa82fF9aFd8f496c3d6ac40E2a0F282E47488CFc9",
    ///     "0x9E545E3C0baAB3E08CdfD552C960A1050f373042",
    /// )
    /// .unwrap();
    ///
    /// let transaction_hash = self.register_as_operator().await.unwrap();
    /// println!("{:?}", transaction_hash);
    /// ```
    pub async fn register_as_operator(
        &self,
        operator: Address,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let operator_details = IDelegationManager::OperatorDetails {
            earningsReceiver: operator,
            delegationApprover: Address::ZERO,
            stakerOptOutWindowBlocks: 0,
        };

        let transaction = self
            .delegation_manager
            .registerAsOperator(operator_details, String::from(""));
        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterAsOperator)?;

        Ok(transaction_hash)
    }

    /// Return true if the operator is registered on Radius AVS.
    pub async fn is_operator_registered_on_avs(
        &self,
        who: Address,
    ) -> Result<bool, PublisherError> {
        self.ecdsa_stake_registry
            .operatorRegistered(who)
            .call()
            .await
            .map_err(PublisherError::IsOperatorRegisteredOnAvs)
    }

    /// Register `self` which is already an EigenLayer operator on Radius AVS.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
    ///     "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
    ///     "0xa82fF9aFd8f496c3d6ac40E2a0F282E47488CFc9",
    ///     "0x9E545E3C0baAB3E08CdfD552C960A1050f373042",
    /// )
    /// .unwrap();
    ///
    /// publisher.register_as_operator().await.unwrap();
    ///
    /// let transaction_hash = publisher.register_operator_on_avs().await.unwrap();
    /// println!("{:?}", transaction_hash);
    /// ```
    pub async fn register_operator_on_avs(
        &self,
        operator: Address,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let salt = [0u8; 32];
        let salt = FixedBytes::from_slice(&salt);
        let now = Utc::now().timestamp();
        let expiry: U256 = U256::from(now + 3600);
        let digest_hash = self
            .avs_directory
            .calculateOperatorAVSRegistrationDigestHash(operator, *self.avs.address(), salt, expiry)
            .call()
            .await
            .map_err(PublisherError::AvsRegistrationDigestHash)?;

        let signature = self
            .signer()
            .sign_hash(&digest_hash)
            .await
            .map_err(PublisherError::OperatorSignature)?;

        let operator_signature = ISignatureUtils::SignatureWithSaltAndExpiry {
            signature: signature.as_bytes().into(),
            salt,
            expiry,
        };

        let transaction = self
            .ecdsa_stake_registry
            .registerOperatorWithSignature(operator, operator_signature);
        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterOperatorOnAvs)?;

        Ok(transaction_hash)
    }

    /// Register a block commitment to be validated by other operators in a
    /// given proposer set.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
    ///     "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
    ///     "0xa82fF9aFd8f496c3d6ac40E2a0F282E47488CFc9",
    ///     "0x9E545E3C0baAB3E08CdfD552C960A1050f373042",
    /// )
    /// .unwrap();
    ///
    /// publisher.register_as_operator().await.unwrap();
    ///
    /// publisher.register_operator_on_avs().await.unwrap();
    ///
    /// let transaction_hash = publisher
    ///     .register_block_commitment(
    ///         [0; 100],
    ///         62364477,
    ///         0,
    ///         "0x38a941d2d4959baae54ba9c14502abe54ffd4ad0db290295f453ef9d7d5a3f2d",
    ///     )
    ///     .await
    ///     .unwrap();
    /// println!("{:?}", transaction_hash);
    /// ```
    pub async fn register_block_commitment(
        &self,
        cluster_id: impl AsRef<str>,
        rollup_id: impl AsRef<str>,
        block_number: u64,
        block_commitment: impl AsRef<[u8]>,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let cluster_id = cluster_id.as_ref().to_owned();
        let rollup_id = rollup_id.as_ref().to_owned();
        let block_commitment = {
            let length = block_commitment.as_ref().len();
            if length != 32 {
                return Err(PublisherError::BlockCommitmentLength(length));
            }

            Bytes::copy_from_slice(block_commitment.as_ref())
        };

        let transaction =
            self.avs
                .createNewTask(block_commitment, block_number, rollup_id, cluster_id);
        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterBlockCommitment)?;

        Ok(transaction_hash)
    }

    pub async fn respond_to_task(
        &self,
        task: IValidationServiceManager::Task,
        task_index: u32,
        block_commitment: impl AsRef<[u8]>,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let block_commitment = Bytes::from_iter(block_commitment.as_ref());
        let transaction = self.avs.respondToTask(task, task_index, block_commitment);
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
    ParseSigningKey(alloy::signers::local::LocalSignerError),
    ParseContractAddress(String, alloy::hex::FromHexError),
    ParseProposerSetId(alloy::hex::FromHexError),
    IsOperator(alloy::contract::Error),
    RegisterAsOperator(TransactionError),
    IsOperatorRegisteredOnAvs(alloy::contract::Error),
    AvsRegistrationDigestHash(alloy::contract::Error),
    OperatorSignature(alloy::signers::Error),
    RegisterOperatorOnAvs(TransactionError),
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
