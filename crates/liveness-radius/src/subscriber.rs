use std::{
    future::Future,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use alloy::{
    eips::BlockNumberOrTag,
    primitives::Address,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{Block, Filter, Log},
    sol_types::SolEvent,
};
use futures::{stream::select_all, Stream, StreamExt};
use pin_project::pin_project;

use crate::types::{Events, Liveness};

pub struct Subscriber {
    connection_detail: WsConnect,
    liveness_contract_address: Address,
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
        liveness_contract_address: impl AsRef<str>,
    ) -> Result<Self, SubscriberError> {
        let connection_detail = WsConnect::new(ethereum_websocket_url.as_ref());
        let liveness_contract_address = Address::from_str(liveness_contract_address.as_ref())
            .map_err(|error| {
                SubscriberError::ParseContractAddress(
                    liveness_contract_address.as_ref().to_owned(),
                    error,
                )
            })?;

        Ok(Self {
            connection_detail,
            liveness_contract_address,
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
    ///         Events::LivenessEvents((event, log)) => match event {
    ///             LivenessEvents::InitializeCluster => {
    ///                 // Handle `InitializeCluster` event.
    ///             }
    ///             LivenessEvents::RegisterSequencer => {
    ///                 // Handle `RegisterSequencer` event.
    ///             }
    ///             LivenessEvents::DeregisterSequencer => {
    ///                 // Handle `DeregisterSequencer` event.
    ///             }
    ///             LivenessEvents::AddRollup => {
    ///                 // Handle `AddRollup` event.
    ///             }
    ///             LivenessEvents::RegisterRollupExecutor => {
    ///                 // Handle `RegisterRollupExecutor` event.
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
            .address(self.liveness_contract_address)
            .from_block(BlockNumberOrTag::Latest);

        let liveness_event_stream: EventStream = provider
            .subscribe_logs(&filter)
            .await
            .map_err(SubscriberError::SubscribeToLogs)?
            .into_stream()
            .boxed()
            .into();

        let mut event_stream = select_all(vec![block_stream, liveness_event_stream]);
        while let Some(event) = event_stream.next().await {
            callback(event, context.clone()).await;
        }

        Err(SubscriberError::EventStreamDisconnected)
    }
}

#[pin_project(project = StreamType)]
enum EventStream {
    BlockStream(Pin<Box<dyn Stream<Item = Block> + Send>>),
    LivenessEventStream(Pin<Box<dyn Stream<Item = Log> + Send>>),
}

impl From<Pin<Box<dyn Stream<Item = Block> + Send>>> for EventStream {
    fn from(value: Pin<Box<dyn Stream<Item = Block> + Send>>) -> Self {
        Self::BlockStream(value)
    }
}

impl From<Pin<Box<dyn Stream<Item = Log> + Send>>> for EventStream {
    fn from(value: Pin<Box<dyn Stream<Item = Log> + Send>>) -> Self {
        Self::LivenessEventStream(value)
    }
}

impl Stream for EventStream {
    type Item = Events;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            StreamType::BlockStream(stream) => stream
                .poll_next_unpin(cx)
                .map(|event| event.map(Events::Block)),
            StreamType::LivenessEventStream(stream) => {
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
            Some(&Liveness::InitializeCluster::SIGNATURE_HASH) => log
                .log_decode::<Liveness::InitializeCluster>()
                .ok()
                .map(|log_decoded| {
                    Events::LivenessEvents(
                        Liveness::LivenessEvents::InitializeCluster(log_decoded.inner.data),
                        log,
                    )
                }),
            Some(&Liveness::RegisterSequencer::SIGNATURE_HASH) => log
                .log_decode::<Liveness::RegisterSequencer>()
                .ok()
                .map(|log_decoded| {
                    Events::LivenessEvents(
                        Liveness::LivenessEvents::RegisterSequencer(log_decoded.inner.data),
                        log,
                    )
                }),
            Some(&Liveness::DeregisterSequencer::SIGNATURE_HASH) => log
                .log_decode::<Liveness::DeregisterSequencer>()
                .ok()
                .map(|log_decoded| {
                    Events::LivenessEvents(
                        Liveness::LivenessEvents::DeregisterSequencer(log_decoded.inner.data),
                        log,
                    )
                }),
            Some(&Liveness::AddRollup::SIGNATURE_HASH) => log
                .log_decode::<Liveness::AddRollup>()
                .ok()
                .map(|log_decoded| {
                    Events::LivenessEvents(
                        Liveness::LivenessEvents::AddRollup(log_decoded.inner.data),
                        log,
                    )
                }),
            Some(&Liveness::RegisterRollupExecutor::SIGNATURE_HASH) => log
                .log_decode::<Liveness::RegisterRollupExecutor>()
                .ok()
                .map(|log_decoded| {
                    Events::LivenessEvents(
                        Liveness::LivenessEvents::RegisterRollupExecutor(log_decoded.inner.data),
                        log,
                    )
                }),
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
