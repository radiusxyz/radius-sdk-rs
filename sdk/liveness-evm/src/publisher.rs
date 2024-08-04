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

type SsalContract = Ssal::SsalInstance<
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
    ssal_contract: SsalContract,
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
        ssal_contract_address: impl AsRef<str>,
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

        let ssal_contract_address =
            Address::from_str(ssal_contract_address.as_ref()).map_err(|error| {
                PublisherError::ParseContractAddress(
                    ssal_contract_address.as_ref().to_owned(),
                    error,
                )
            })?;
        let ssal_contract = Ssal::SsalInstance::new(ssal_contract_address, provider.clone());

        Ok(Self {
            provider,
            ssal_contract,
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
            .ssal_contract
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
    /// let event = publisher.initialize_proposer_set().await?;
    ///
    /// println!(
    ///     "Owner: {}\nProposer Set ID: {}",
    ///     event.owner, event.proposerSetId
    /// );
    /// ```
    pub async fn initialize_proposer_set(
        &self,
    ) -> Result<Ssal::InitializeProposerSet, PublisherError> {
        let contract_call = self.ssal_contract.initializeProposerSet();
        let pending_transaction = contract_call.send().await;
        let event: Ssal::InitializeProposerSet = self
            .extract_event_from_pending_transaction(pending_transaction)
            .await
            .map_err(PublisherError::InitializeProposerSet)?;

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
        proposer_set_id: impl AsRef<str>,
    ) -> Result<Ssal::RegisterSequencer, PublisherError> {
        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let contract_call = self.ssal_contract.registerSequencer(proposer_set_id);
        let pending_transaction = contract_call.send().await;
        let event: Ssal::RegisterSequencer = self
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
        proposer_set_id: impl AsRef<str>,
    ) -> Result<Ssal::DeregisterSequencer, PublisherError> {
        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let contract_call = self.ssal_contract.deregisterSequencer(proposer_set_id);
        let pending_transaction = contract_call.send().await;
        let event: Ssal::DeregisterSequencer = self
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
    ///     .get_sequencer_list(proposer_set_id, block_number)
    ///     .await
    ///     .unwrap();
    ///
    /// println!("{:?}", sequencer_list);
    /// ```
    pub async fn get_sequencer_list(
        &self,
        proposer_set_id: impl AsRef<str>,
        block_number: u64,
    ) -> Result<Vec<Address>, PublisherError> {
        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let sequencer_list = self
            .ssal_contract
            .getSequencerList(proposer_set_id)
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
    /// let is_registered = publisher.is_registered(proposer_set_id).await.unwrap();
    ///
    /// assert!(is_registered == true);
    /// ```
    pub async fn is_registered(
        &self,
        proposer_set_id: impl AsRef<str>,
    ) -> Result<bool, PublisherError> {
        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let is_registered: bool = self
            .ssal_contract
            .isRegistered(proposer_set_id, self.address())
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
    ParseProposerSetId(alloy::hex::FromHexError),
    GetBlockNumber(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    GetBlockMargin(alloy::contract::Error),
    InitializeProposerSet(TransactionError),
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
