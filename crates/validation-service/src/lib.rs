pub mod validation_dkg;
pub use validation_dkg::*;
pub mod validation_eigenlayer;
pub mod validation_symbiotic;
use std::future::Future;

pub use async_trait::async_trait;
pub use primitives::*;

pub type RestakingValidationServiceResult<T> = Result<T, RestakingValidationError>;

pub mod primitives {
    pub use std::str::FromStr;

    pub use alloy::{
        contract::Error as ContractError,
        network::Ethereum,
        primitives::*,
        providers::{
            PendingTransactionBuilder, PendingTransactionError, Provider, ProviderBuilder,
        },
        rpc::types::Log,
        signers::{
            k256::ecdsa::SigningKey,
            local::{LocalSigner, LocalSignerError},
            Signer,
        },
        transports::{RpcError, TransportErrorKind},
    };
}

pub enum MultiAddress {
    Ethereum(alloy::primitives::Address),
}

#[async_trait]
pub trait RestakingValidation {
    /// Type of proof that is used for validation
    type Proof;
    /// Type of event that this service is listening
    type Event;
    /// Type of key that is used for periodic task
    type Key;

    /// Create `task` with `key` and `proof`
    async fn create_task(
        &self,
        key: Self::Key,
        proof: Self::Proof,
    ) -> RestakingValidationServiceResult<()>;

    /// Respond `task` for `key`
    async fn respond_task(&self, key: Self::Key) -> RestakingValidationServiceResult<()>;

    /// Subscribe to `Self::Event` with given callback
    async fn subscribe_events<CB, Fut>(&self, callback: CB) -> RestakingValidationServiceResult<()>
    where
        CB: Send + Fn(Self::Event) -> Fut,
        Fut: Send + Future<Output = ()>;
}

#[derive(Debug, thiserror::Error)]
pub enum RestakingValidationError {
    #[error(transparent)]
    RpcError(RpcError<TransportErrorKind>),
    #[error(transparent)]
    FailedToGetTxReceipt(PendingTransactionError),
    #[error("Failed on event subscription")]
    SubscribeError,
    #[error("Failed on validation")]
    ValidationServiceError(#[from] Box<dyn std::error::Error>),
}
