use std::{
    future::Future,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use alloy::{
    eips::BlockNumberOrTag,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::Filter,
    sol_types::SolEvent,
};
use futures::{stream::select_all, Stream, StreamExt};
use pin_project::pin_project;

use crate::types::*;

pub struct Subscriber {
    connection_detail: WsConnect,
    avs_directory_contract_address: Address,
    delegation_manager_contract_address: Address,
}

impl Subscriber {
    /// Create a new [`Subscriber`] instance to listen to events emitted by
    /// `AVSDirectory` and `DelegationManager` contract.
    ///
    /// # Examples
    ///
    /// ```
    /// let subscriber = Subscriber::new(
    ///     "ws://127.0.0.1:8545",
    ///     "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707",
    ///     "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9",
    /// )
    /// .unwrap();
    /// ```
    pub fn new(
        ethereum_websocket_url: impl AsRef<str>,
        avs_directory_contract_address: impl AsRef<str>,
        delegation_manager_contract_address: impl AsRef<str>,
    ) -> Result<Self, SubscriberError> {
        let connection_detail = WsConnect::new(ethereum_websocket_url.as_ref());
        let avs_directory_contract_address =
            Address::from_str(avs_directory_contract_address.as_ref())
                .map_err(SubscriberError::ParseAVSDirectoryContractAddress)?;

        let delegation_manager_contract_address =
            Address::from_str(delegation_manager_contract_address.as_ref())
                .map_err(SubscriberError::ParseDelegationManagerContractAddress)?;

        Ok(Self {
            connection_detail,
            avs_directory_contract_address,
            delegation_manager_contract_address,
        })
    }

    pub async fn initialize_event_handler<CB, CTX, F>(
        &self,
        callback: CB,
        context: CTX,
    ) -> Result<(), SubscriberError>
    where
        CB: Fn(Events, CTX) -> F,
        CTX: Clone + Send + Sync,
        F: Future<Output = ()>,
    {
        let provider = ProviderBuilder::new()
            .on_ws(self.connection_detail.clone())
            .await
            .map_err(SubscriberError::WebsocketProvider)?;

        let avs_directory_filter = Filter::new()
            .address(self.avs_directory_contract_address)
            .from_block(BlockNumberOrTag::Latest);

        let delegation_manager_filter = Filter::new()
            .address(self.delegation_manager_contract_address)
            .from_block(BlockNumberOrTag::Latest);

        Err(SubscriberError::EventStreamDisconnected)
    }
}

#[derive(Debug)]
pub enum SubscriberError {
    ParseAVSDirectoryContractAddress(alloy::hex::FromHexError),
    ParseDelegationManagerContractAddress(alloy::hex::FromHexError),
    WebsocketProvider(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    NewBlockEventStream(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    SubscribeToBlock(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    SubscribeToLogs(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    EventStreamDisconnected,
}

impl std::fmt::Display for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SubscriberError {}
