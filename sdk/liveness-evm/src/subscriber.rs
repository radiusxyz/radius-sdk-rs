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
    /// Create a new [`Subscriber`] instance to listen to events emitted by the
    /// contract.
    ///
    /// # Examples
    ///
    /// ```
    /// let subscriber = Subscriber::new(
    ///     "ws://127.0.0.1:8545",
    ///     "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    /// )
    /// .unwrap();
    /// ```
    pub fn new(
        ethereum_websocket_url: impl AsRef<str>,
        ssal_contract_address: impl AsRef<str>,
    ) -> Result<Self, SubscriberError> {
        let connection_detail = WsConnect::new(ethereum_websocket_url.as_ref());
        let ssal_contract_address =
            Address::from_str(ssal_contract_address.as_ref()).map_err(|error| {
                SubscriberError::ParseContractAddress(
                    ssal_contract_address.as_ref().to_owned(),
                    error,
                )
            })?;

        Ok(Self {
            connection_detail,
            ssal_contract_address,
        })
    }

    /// Start listening to the Ethereum block creation and contract events.
    ///
    /// # WARNING
    ///
    /// This is a blocking operation unless spawned in a separate thread.
    ///
    /// # Examples - `tokio`
    ///
    /// ```
    /// let context = Arc::new(String::from("context"));
    ///
    /// tokio::spawn(async move {
    ///     Subscriber::new(
    ///         "ws://127.0.0.1:8545",
    ///         "0x67d269191c92Caf3cD7723F116c85e6E9bf55933",
    ///     )
    ///     .unwrap()
    ///     .initialize_event_handler(callback, ())
    ///     .await
    ///     .unwrap();
    /// });
    ///
    /// async fn callback(events: Events, context: Arc<String>) {
    ///     match events {
    ///         Events::Block(block) => {
    ///             // Handle Ethereum block creation event.
    ///         }
    ///         Events::SsalEvents(contract_events) => match contract_events {
    ///             SsalEvents::InitializeProposerSet => {
    ///                 // Handle `InitializeProposerSet` event.
    ///             }
    ///             SsalEvents::RegisterSequencer => {
    ///                 // Handle `RegisterSequencer` event.
    ///             }
    ///             SsalEvents::DeregisterSequencer => {
    ///                 // Handle `DeregisterSequencer` event.
    ///             }
    ///         },
    ///     }
    /// }
    /// ```
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
            .map_err(SubscriberError::SubscribeToLogs)?
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
                        Some(Ssal::SsalEvents::InitializeProposerSet(log.inner.data).into())
                    }
                    None => None,
                }
            }
            Some(&Ssal::RegisterSequencer::SIGNATURE_HASH) => {
                match log.log_decode::<Ssal::RegisterSequencer>().ok() {
                    Some(log) => Some(Ssal::SsalEvents::RegisterSequencer(log.inner.data).into()),
                    None => None,
                }
            }
            Some(&Ssal::DeregisterSequencer::SIGNATURE_HASH) => {
                match log.log_decode::<Ssal::DeregisterSequencer>().ok() {
                    Some(log) => Some(Ssal::SsalEvents::DeregisterSequencer(log.inner.data).into()),
                    None => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SubscriberError {
    ParseContractAddress(String, alloy::hex::FromHexError),
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
