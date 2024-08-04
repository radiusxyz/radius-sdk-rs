use std::str::FromStr;

use alloy::{
    contract,
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, PendingTransactionBuilder, ProviderBuilder, RootProvider, WalletProvider,
    },
    signers::{k256::ecdsa::SigningKey, local::LocalSigner, Signer},
    transports::http::{reqwest::Url, Client, Http},
};
use chrono::Utc;

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

type DelegationManagerContract = DelegationManager::DelegationManagerInstance<
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

type AvsDirectoryContract = AVSDirectory::AVSDirectoryInstance<
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

type EcdsaStakeRegistryContract = EcdsaStakeRegistry::EcdsaStakeRegistryInstance<
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

type AvsContract = Avs::AvsInstance<
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
    signer: LocalSigner<SigningKey>,
    delegation_manager_contract: DelegationManagerContract,
    avs_directory_contract: AvsDirectoryContract,
    ecdsa_stake_registry_contract: EcdsaStakeRegistryContract,
    avs_contract: AvsContract,
}

impl Publisher {
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
        ethereum_rpc_url: impl AsRef<str>,
        signing_key: impl AsRef<str>,
        delegation_manager_contract_address: impl AsRef<str>,
        avs_directory_contract_address: impl AsRef<str>,
        ecdsa_stake_registry_contract_address: impl AsRef<str>,
        avs_contract_address: impl AsRef<str>,
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

        let delegation_manager_contract_address =
            Address::from_str(delegation_manager_contract_address.as_ref()).map_err(|error| {
                PublisherError::ParseContractAddress(
                    delegation_manager_contract_address.as_ref().to_owned(),
                    error,
                )
            })?;
        let delegation_manager_contract =
            DelegationManager::new(delegation_manager_contract_address, provider.clone());

        let avs_directory_contract_address =
            Address::from_str(avs_directory_contract_address.as_ref()).map_err(|error| {
                PublisherError::ParseContractAddress(
                    avs_directory_contract_address.as_ref().to_owned(),
                    error,
                )
            })?;
        let avs_directory_contract =
            AVSDirectory::new(avs_directory_contract_address, provider.clone());

        let ecdsa_stake_registry_contract_address =
            Address::from_str(ecdsa_stake_registry_contract_address.as_ref()).map_err(|error| {
                PublisherError::ParseContractAddress(
                    ecdsa_stake_registry_contract_address.as_ref().to_owned(),
                    error,
                )
            })?;
        let ecdsa_stake_registry_contract =
            EcdsaStakeRegistry::new(ecdsa_stake_registry_contract_address, provider.clone());

        let avs_contract_address =
            Address::from_str(avs_contract_address.as_ref()).map_err(|error| {
                PublisherError::ParseContractAddress(
                    avs_contract_address.as_ref().to_owned(),
                    error,
                )
            })?;
        let avs_contract = Avs::new(avs_contract_address, provider.clone());

        Ok(Self {
            provider,
            signer,
            delegation_manager_contract,
            avs_directory_contract,
            ecdsa_stake_registry_contract,
            avs_contract,
        })
    }

    /// Get the address for the wallet used by [`Publisher`].
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
    /// let my_address = publisher.address();
    /// ```
    pub fn address(&self) -> Address {
        self.provider.default_signer_address()
    }

    fn signer(&self) -> &LocalSigner<SigningKey> {
        &self.signer
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

    /// Return `true` if `self` is registered as an EigenLayer operator.
    pub async fn is_operator(&self) -> Result<bool, PublisherError> {
        let is_operator = self
            .delegation_manager_contract
            .isOperator(self.address())
            .call()
            .await
            .map_err(PublisherError::IsOperator)?
            ._0;

        Ok(is_operator)
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
    pub async fn register_as_operator(&self) -> Result<FixedBytes<32>, PublisherError> {
        let operator_details = DelegationManager::OperatorDetails {
            earningsReceiver: self.address(),
            delegationApprover: Address::ZERO,
            stakerOptOutWindowBlocks: 0,
        };

        let transaction = self
            .delegation_manager_contract
            .registerAsOperator(operator_details, String::from(""));
        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterAsOperator)?;

        Ok(transaction_hash)
    }

    /// Return true if the operator is registered on Radius AVS.
    pub async fn is_operator_registered_on_avs(&self) -> Result<bool, PublisherError> {
        let is_avs = self
            .ecdsa_stake_registry_contract
            .operatorRegistered(self.address())
            .call()
            .await
            .map_err(PublisherError::IsOperatorRegisteredOnAvs)?
            ._0;

        Ok(is_avs)
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
    pub async fn register_operator_on_avs(&self) -> Result<FixedBytes<32>, PublisherError> {
        let salt = [0u8; 32];
        let salt = FixedBytes::from_slice(&salt);
        let now = Utc::now().timestamp();
        let expiry: U256 = U256::from(now + 3600);
        let digest_hash = self
            .avs_directory_contract
            .calculateOperatorAVSRegistrationDigestHash(
                self.address(),
                *self.avs_contract.address(),
                salt,
                expiry,
            )
            .call()
            .await
            .map_err(PublisherError::AvsRegistrationDigestHash)?
            ._0;

        let signature = self
            .signer()
            .sign_hash(&digest_hash)
            .await
            .map_err(PublisherError::OperatorSignature)?;

        let operator_signature = EcdsaStakeRegistry::SignatureWithSaltAndExpiry {
            signature: signature.as_bytes().into(),
            salt,
            expiry,
        };

        let transaction = self
            .ecdsa_stake_registry_contract
            .registerOperatorWithSignature(self.address(), operator_signature);
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
        block_commitment: impl AsRef<[u8]>,
        block_number: u64,
        rollup_id: u32,
        proposer_set_id: impl AsRef<str>,
    ) -> Result<FixedBytes<32>, PublisherError> {
        let block_commitment = Bytes::from_iter(block_commitment.as_ref());

        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let transaction = self.avs_contract.createNewTask(
            block_commitment,
            block_number,
            rollup_id,
            proposer_set_id,
        );
        let pending_transaction = transaction.send().await;
        let transaction_hash = self
            .extract_transaction_hash_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterBlockCommitment)?;

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
    ParseProposerSetId(alloy::hex::FromHexError),
    IsOperator(alloy::contract::Error),
    RegisterAsOperator(TransactionError),
    IsOperatorRegisteredOnAvs(alloy::contract::Error),
    AvsRegistrationDigestHash(alloy::contract::Error),
    OperatorSignature(alloy::signers::Error),
    RegisterOperatorOnAvs(TransactionError),
    RegisterBlockCommitment(TransactionError),
}

impl std::fmt::Display for PublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PublisherError {}
