use std::str::FromStr;

use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, ProviderBuilder, RootProvider, WalletProvider,
    },
    signers::local::LocalSigner,
    transports::http::{reqwest::Url, Client, Http},
};
use Ssal::{DeregisterSequencer, InitializeProposerSet, RegisterSequencer};

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

    pub async fn initialize_proposer_set(&self) -> Result<InitializeProposerSet, PublisherError> {
        let transaction_receipt = self
            .ssal_contract
            .initializeProposerSet()
            .send()
            .await
            .map_err(PublisherError::SendTransaction)?
            .get_receipt()
            .await
            .map_err(PublisherError::GetReceipt)?;

        let log = transaction_receipt
            .as_ref()
            .logs()
            .first()
            .ok_or(PublisherError::EmptyLogs)?
            .log_decode::<Ssal::InitializeProposerSet>()
            .map_err(PublisherError::DecodeLogData)?;

        Ok(log.inner.data)
    }

    pub async fn register_sequencer(
        &self,
        proposer_set_id: impl AsRef<str>,
    ) -> Result<RegisterSequencer, PublisherError> {
        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let transaction_receipt = self
            .ssal_contract
            .registerSequencer(proposer_set_id)
            .send()
            .await
            .map_err(PublisherError::SendTransaction)?
            .get_receipt()
            .await
            .map_err(PublisherError::GetReceipt)?;

        let log = transaction_receipt
            .as_ref()
            .logs()
            .first()
            .ok_or(PublisherError::EmptyLogs)?
            .log_decode::<Ssal::RegisterSequencer>()
            .map_err(PublisherError::DecodeLogData)?;

        Ok(log.inner.data)
    }

    pub async fn deregister_sequencer(
        &self,
        proposer_set_id: impl AsRef<str>,
    ) -> Result<DeregisterSequencer, PublisherError> {
        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let transaction_receipt = self
            .ssal_contract
            .deregisterSequencer(proposer_set_id)
            .send()
            .await
            .map_err(PublisherError::SendTransaction)?
            .get_receipt()
            .await
            .map_err(PublisherError::GetReceipt)?;

        let log = transaction_receipt
            .as_ref()
            .logs()
            .first()
            .ok_or(PublisherError::EmptyLogs)?
            .log_decode::<Ssal::DeregisterSequencer>()
            .map_err(PublisherError::DecodeLogData)?;

        Ok(log.inner.data)
    }

    pub async fn get_sequencer_list(
        &self,
        proposer_set_id: impl AsRef<str>,
        block_number: u64,
    ) -> Result<Vec<Address>, PublisherError> {
        let proposer_set_id = FixedBytes::from_str(proposer_set_id.as_ref())
            .map_err(PublisherError::ParseProposerSetId)?;

        let sequencer_list: [Address; 30] = self
            .ssal_contract
            .getSequencerList(proposer_set_id)
            .call()
            .block(block_number.into())
            .await
            .map_err(PublisherError::EthCall)?
            ._0;

        // Filter sequencer address whose value is zero (== [0; 20])
        let filtered_list: Vec<Address> = sequencer_list
            .into_iter()
            .filter(|sequencer_address| !sequencer_address.is_zero())
            .collect();

        Ok(filtered_list)
    }
}

#[derive(Debug)]
pub enum PublisherError {
    ParseEthereumRpcUrl(Box<dyn std::error::Error>),
    ParseSigningKey(alloy::signers::local::LocalSignerError),
    ParseContractAddress(alloy::hex::FromHexError),
    ParseProposerSetId(alloy::hex::FromHexError),
    SendTransaction(alloy::contract::Error),
    GetReceipt(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    EmptyLogs,
    DecodeLogData(alloy::sol_types::Error),
    EthCall(alloy::contract::Error),
    WebsocketProvider(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
}

impl std::fmt::Display for PublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PublisherError {}
