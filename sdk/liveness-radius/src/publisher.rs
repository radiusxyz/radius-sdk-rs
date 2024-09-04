use std::str::FromStr;

use alloy::{
    contract,
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, PendingTransactionBuilder, Provider, ProviderBuilder, RootProvider,
        WalletProvider,
    },
    signers::local::LocalSigner,
    sol_types::SolEvent,
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

type LivenessContract = Liveness::LivenessInstance<
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
    liveness_contract: LivenessContract,
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
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    /// ```
    pub fn new(
        ethereum_rpc_url: impl AsRef<str>,
        signing_key: impl AsRef<str>,
        liveness_contract_address: impl AsRef<str>,
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

        let liveness_contract_address = Address::from_str(liveness_contract_address.as_ref())
            .map_err(|error| {
                PublisherError::ParseContractAddress(
                    liveness_contract_address.as_ref().to_owned(),
                    error,
                )
            })?;
        let liveness_contract =
            Liveness::LivenessInstance::new(liveness_contract_address, provider.clone());

        Ok(Self {
            provider,
            liveness_contract,
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
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    ///
    /// let my_address = publisher.address();
    /// ```
    pub fn address(&self) -> Address {
        self.provider.default_signer_address()
    }

    /// Get the latest Ethereum block number available.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    ///
    /// let ethereum_latest_block_number = publisher.get_block_number().await.unwrap();
    /// ```
    pub async fn get_block_number(&self) -> Result<u64, PublisherError> {
        let block_number = self
            .provider
            .get_block_number()
            .await
            .map_err(PublisherError::GetBlockNumber)?;

        Ok(block_number)
    }

    /// # TODO:
    /// Fix the block margin return type to one of the smaller types.
    ///
    /// Get the block margin specified by the contract. Use the block margin to
    /// check the validity of the block number passed to the
    /// [`get_sequencer_list()`] function.
    ///
    /// # Examples
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    ///
    /// let block_margin = publisher.get_block_margin().await.unwrap();
    /// ```
    pub async fn get_block_margin(&self) -> Result<Uint<256, 4>, PublisherError> {
        let block_margin = self
            .liveness_contract
            .BLOCK_MARGIN()
            .call()
            .await
            .map_err(PublisherError::GetBlockMargin)?
            ._0;

        Ok(block_margin)
    }

    async fn extract_event_from_pending_transaction<'a, T>(
        &'a self,
        pending_transaction: Result<
            PendingTransactionBuilder<'a, Http<Client>, Ethereum>,
            contract::Error,
        >,
    ) -> Result<T, TransactionError>
    where
        T: SolEvent,
    {
        let transaction_receipt = pending_transaction
            .map_err(TransactionError::SendTransaction)?
            .get_receipt()
            .await
            .map_err(TransactionError::GetReceipt)?;

        match transaction_receipt.as_ref().is_success() {
            true => {
                let log = transaction_receipt
                    .as_ref()
                    .logs()
                    .first()
                    .ok_or(TransactionError::EmptyLogs)?
                    .log_decode::<T>()
                    .map_err(TransactionError::DecodeLogData)?;

                Ok(log.inner.data)
            }
            false => Err(TransactionError::FailedTransaction(
                transaction_receipt.transaction_hash,
            )),
        }
    }

    /// Send transaction to initialize the proposer set and wait for the event
    /// to return.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    ///
    /// let event = publisher.initialize_cluster("radius").await?;
    ///
    /// println!(
    ///     "Owner: {}\Cluster ID: {}",
    ///     event.owner, event.proposerSetId
    /// );
    /// ```
    pub async fn initialize_cluster(
        &self,
        cluster_id: impl AsRef<str>,
    ) -> Result<Liveness::InitializeCluster, PublisherError> {
        let contract_call = self
            .liveness_contract
            .initializeCluster(cluster_id.as_ref().to_string());
        let pending_transaction = contract_call.send().await;
        let event: Liveness::InitializeCluster = self
            .extract_event_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::InitializeCluster)?;

        Ok(event)
    }

    /// Register the current [`Publisher`] instance as a sequencer of the
    /// proposer set. The address of the registered sequencer is equivalent
    /// to that of self.address().
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    ///
    /// let event = publisher
    ///     .register_sequencer("0xdd45347e5d10daaadb40f185225fc8d860d2888b5c411aca387e17a265e2f491")
    ///     .await
    ///     .unwrap();
    ///
    /// assert!(event.sequencerAddress == publisher.address());
    /// ```
    pub async fn register_sequencer(
        &self,
        cluster_id: impl AsRef<str>,
    ) -> Result<Liveness::RegisterSequencer, PublisherError> {
        let contract_call = self
            .liveness_contract
            .registerSequencer(cluster_id.as_ref().to_string());
        let pending_transaction = contract_call.send().await;
        let event: Liveness::RegisterSequencer = self
            .extract_event_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::RegisterSequencer)?;

        Ok(event)
    }

    /// Deregister the publisher's address from the proposer set.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    ///
    /// let event = publisher
    ///     .deregister_sequencer("0xdd45347e5d10daaadb40f185225fc8d860d2888b5c411aca387e17a265e2f491")
    ///     .await
    ///     .unwrap();
    ///
    /// assert!(event.sequencerAddress == publisher.address());
    /// ```
    pub async fn deregister_sequencer(
        &self,
        cluster_id: impl AsRef<str>,
    ) -> Result<Liveness::DeregisterSequencer, PublisherError> {
        let contract_call = self
            .liveness_contract
            .deregisterSequencer(cluster_id.as_ref().to_string());
        let pending_transaction = contract_call.send().await;
        let event: Liveness::DeregisterSequencer = self
            .extract_event_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::DeregisterSequencer)?;

        Ok(event)
    }

    /// Get the addresses of registered sequencers in a given proposer set for a
    /// given block number.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )?;
    ///
    /// let block_number = publisher.get_block_number().await.unwrap();
    /// let sequencer_list = publisher
    ///     .get_sequencer_list(cluster_id, block_number)
    ///     .await
    ///     .unwrap();
    ///
    /// println!("{:?}", sequencer_list);
    /// ```
    pub async fn get_sequencer_list(
        &self,
        cluster_id: impl AsRef<str>,
        block_number: u64,
    ) -> Result<Vec<Address>, PublisherError> {
        let sequencer_list = self
            .liveness_contract
            .getSequencerList(cluster_id.as_ref().to_string())
            .call()
            .block(block_number.into())
            .await
            .map_err(PublisherError::GetSequencerList)?
            ._0;

        // Filter sequencer address whose value is zero (== [0; 20])
        let filtered_list: Vec<Address> = sequencer_list
            .into_iter()
            .filter(|sequencer_address| !sequencer_address.is_zero())
            .collect();

        Ok(filtered_list)
    }

    /// Check if the current publisher is registered as a sequencer in the
    /// proposer set.
    ///
    /// # Examples
    ///
    /// ```
    /// let publisher = Publisher::new(
    ///     "http://127.0.0.1:8545",
    ///     "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    ///
    /// let _event = publisher
    ///     .register_sequencer("0xdd45347e5d10daaadb40f185225fc8d860d2888b5c411aca387e17a265e2f491")
    ///     .await
    ///     .unwrap();
    ///
    /// let is_registered = publisher.is_registered(cluster_id).await.unwrap();
    ///
    /// assert!(is_registered == true);
    /// ```
    pub async fn is_registered(&self, cluster_id: impl AsRef<str>) -> Result<bool, PublisherError> {
        let is_registered: bool = self
            .liveness_contract
            .isRegistered(cluster_id.as_ref().to_string(), self.address())
            .call()
            .await
            .map_err(PublisherError::IsRegistered)?
            ._0;

        Ok(is_registered)
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
    GetBlockNumber(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    GetBlockMargin(alloy::contract::Error),
    InitializeCluster(TransactionError),
    RegisterSequencer(TransactionError),
    DeregisterSequencer(TransactionError),
    GetSequencerList(alloy::contract::Error),
    IsRegistered(alloy::contract::Error),
}

impl std::fmt::Display for PublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PublisherError {}
