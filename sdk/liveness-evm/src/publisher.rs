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
    pub fn new(
        ethereum_rpc_url: impl AsRef<str>,
        signing_key: impl AsRef<str>,
        contract_address: impl AsRef<str>,
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

        let ssal_contract_address = Address::from_str(contract_address.as_ref())
            .map_err(PublisherError::ParseContractAddress)?;
        let ssal_contract = Ssal::SsalInstance::new(ssal_contract_address, provider.clone());

        Ok(Self {
            provider,
            ssal_contract,
        })
    }

    pub fn address(&self) -> Address {
        self.provider.default_signer_address()
    }

    pub async fn get_block_number(&self) -> Result<u64, PublisherError> {
        let block_number = self
            .provider
            .get_block_number()
            .await
            .map_err(PublisherError::GetBlockNumber)?;

        Ok(block_number)
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

        let log = transaction_receipt
            .as_ref()
            .logs()
            .first()
            .ok_or(TransactionError::EmptyLogs)?
            .log_decode::<T>()
            .map_err(TransactionError::DecodeLogData)?;

        Ok(log.inner.data)
    }

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

    pub async fn is_registered(
        &self,
        proposer_set_id: impl AsRef<str>,
    ) -> Result<bool, PublisherError> {
        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let is_registered = self
            .ssal_contract
            .isRegistered(proposer_set_id)
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
    ParseContractAddress(alloy::hex::FromHexError),
    ParseProposerSetId(alloy::hex::FromHexError),
    GetBlockNumber(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
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
