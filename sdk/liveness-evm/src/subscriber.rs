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
    ssal_contract_address: Address,
}

impl Subscriber {
    pub async fn new(
        ethereum_websocket_url: impl AsRef<str>,
        contract_address: impl AsRef<str>,
    ) -> Result<Self, SubscriberError> {
        let connection_detail = WsConnect::new(ethereum_websocket_url.as_ref());
        let ssal_contract_address = Address::from_str(contract_address.as_ref())
            .map_err(SubscriberError::ParseContractAddress)?;

        Ok(Self {
            connection_detail,
            ssal_contract_address,
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

        let block_stream: EventStream = provider
            .subscribe_blocks()
            .await
            .map_err(SubscriberError::SubscribeToBlock)?
            .into_stream()
            .boxed()
            .into();

        let filter = Filter::new()
            .address(self.ssal_contract_address)
            .from_block(BlockNumberOrTag::Latest);

        let ssal_event_stream: EventStream = provider
            .subscribe_logs(&filter)
            .await
            .map_err(SubscriberError::SubscribeToLog)?
            .into_stream()
            .boxed()
            .into();

        let mut event_stream = select_all(vec![block_stream, ssal_event_stream]);
        while let Some(event) = event_stream.next().await {
            callback(event, context.clone()).await;
        }

        Err(SubscriberError::EventStreamDisconnected)
    }
}

#[pin_project(project = StreamType)]
enum EventStream {
    BlockStream(Pin<Box<dyn Stream<Item = Block> + Send>>),
    SsalEventStream(Pin<Box<dyn Stream<Item = Log> + Send>>),
}

impl From<Pin<Box<dyn Stream<Item = Block> + Send>>> for EventStream {
    fn from(value: Pin<Box<dyn Stream<Item = Block> + Send>>) -> Self {
        Self::BlockStream(value)
    }
}

impl From<Pin<Box<dyn Stream<Item = Log> + Send>>> for EventStream {
    fn from(value: Pin<Box<dyn Stream<Item = Log> + Send>>) -> Self {
        Self::SsalEventStream(value)
    }
}

impl Stream for EventStream {
    type Item = Events;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            StreamType::BlockStream(stream) => {
                stream.poll_next_unpin(cx).map(|event| match event {
                    Some(block) => Some(Events::Block(block)),
                    None => None,
                })
            }
            StreamType::SsalEventStream(stream) => {
                stream.poll_next_unpin(cx).map(|event| match event {
                    Some(log) => Self::decode_log(log),
                    None => None,
                })
            }
        }
    }
}

impl EventStream {
    fn decode_log(log: Log) -> Option<Events> {
        match log.topic0() {
            Some(&Ssal::InitializeProposerSet::SIGNATURE_HASH) => {
                match log.log_decode::<Ssal::InitializeProposerSet>().ok() {
                    Some(log) => {
                        let event = log.inner.data;
                        Some(Ssal::SsalEvents::InitializeProposerSet(event).into())
                    }
                    None => None,
                }
            }
            Some(&Ssal::RegisterSequencer::SIGNATURE_HASH) => {
                match log.log_decode::<Ssal::RegisterSequencer>().ok() {
                    Some(log) => {
                        let event = log.inner.data;
                        Some(Ssal::SsalEvents::RegisterSequencer(event).into())
                    }
                    None => None,
                }
            }
            Some(&Ssal::DeregisterSequencer::SIGNATURE_HASH) => {
                match log.log_decode::<Ssal::DeregisterSequencer>().ok() {
                    Some(log) => {
                        let event = log.inner.data;
                        Some(Ssal::SsalEvents::DeregisterSequencer(event).into())
                    }
                    None => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SubscriberError {
    ParseContractAddress(alloy::hex::FromHexError),
    WebsocketProvider(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    NewBlockEventStream(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    SubscribeToBlock(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    SubscribeToLog(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    EventStreamDisconnected,
}

impl std::fmt::Display for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SubscriberError {}
