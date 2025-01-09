use std::{future::Future, str::FromStr};

use alloy::providers::{ProviderBuilder, WsConnect};
use futures::StreamExt;

use crate::types::*;

pub struct Subscriber {
    connection_detail: WsConnect,
    validation_contract_address: Address,
}

impl Subscriber {
    /// Create a new [`Subscriber`] instance to listen to events emitted by
    /// validation contract.
    ///
    /// # Examples
    ///
    /// ```
    /// let subscriber = Subscriber::new(
    ///     "ws://127.0.0.1:8545",
    ///     "0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690",
    /// )
    /// .unwrap();
    /// ```
    pub fn new(
        ethereum_websocket_url: impl AsRef<str>,
        validation_contract_address: impl AsRef<str>,
    ) -> Result<Self, SubscriberError> {
        let connection_detail = WsConnect::new(ethereum_websocket_url.as_ref());

        let validation_contract_address = Address::from_str(validation_contract_address.as_ref())
            .map_err(|error| {
            SubscriberError::ParseContractAddress(
                validation_contract_address.as_ref().to_owned(),
                error,
            )
        })?;

        Ok(Self {
            connection_detail,
            validation_contract_address,
        })
    }

    /// Start listening to the Block commitment registration event.
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
    ///         "0xc3e53F4d16Ae77Db1c982e75a937B9f60FE63690",
    ///     )
    ///     .unwrap()
    ///     .initialize_event_handler(callback, context.clone())
    ///     .await
    ///     .unwrap();
    /// });
    ///
    /// async fn callback(
    ///     block_commitment: ValidationServiceManager::NewTaskCreated,
    ///     _context: Arc<String>,
    /// ) {
    ///     todo!("Validate the block commitment");
    /// }
    /// ```
    pub async fn initialize_event_handler<CB, CTX, F>(
        &self,
        callback: CB,
        context: CTX,
    ) -> Result<(), SubscriberError>
    where
        CB: Fn(ValidationServiceManager::NewTaskCreated, CTX) -> F,
        CTX: Clone + Send + Sync,
        F: Future<Output = ()>,
    {
        let provider = ProviderBuilder::new()
            .on_ws(self.connection_detail.clone())
            .await
            .map_err(SubscriberError::WebsocketProvider)?;

        let validation_contract = ValidationServiceManager::ValidationServiceManagerInstance::new(
            self.validation_contract_address,
            provider.clone(),
        );

        let mut validation_contract_event_stream = validation_contract
            .NewTaskCreated_filter()
            .subscribe()
            .await
            .map_err(SubscriberError::SubscribeToAvsContract)?
            .into_stream();

        while let Some(Ok(event)) = validation_contract_event_stream.next().await {
            callback(event.0, context.clone()).await;
        }

        Err(SubscriberError::EventStreamDisconnected)
    }
}

#[derive(Debug)]
pub enum SubscriberError {
    ParseContractAddress(String, alloy::hex::FromHexError),
    WebsocketProvider(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    SubscribeToAvsContract(alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    EventStreamDisconnected,
}

impl std::fmt::Display for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SubscriberError {}
