use std::{future::Future, str::FromStr, sync::Arc};

use alloy::{
    network::EthereumWallet,
    primitives::Address as EthAddress,
    providers::{Provider, ProviderBuilder, WsConnect},
    signers::local::LocalSigner,
    sol,
    transports::{http::reqwest::Url, RpcError, TransportErrorKind},
};
use signature::Address;

mod subscriber;
pub use subscriber::*;

mod publisher;
use async_trait::async_trait;
pub use publisher::*;
pub use DkgValidationContract::DkgValidationContractInstance;

use crate::{RestakingValidation, RestakingValidationServiceResult};

pub type DkgValidationServiceResult<T> = Result<T, DkgValidationServiceError>;
pub type SessionId = u64;

sol! {
    #![sol(rpc, all_derives)]
    DkgValidationContract,
    "src/validation_dkg/abi/DkgBApp.json"
}

async fn create_subscriber(ws_url: &str, contract_address: EthAddress) -> Subscriber<impl Provider + Clone> {
    let ws_provider = ProviderBuilder::new()
        .connect_ws(WsConnect::new(Url::parse(ws_url).unwrap()))
        .await
        .expect("Failed to connect to WS");
    Subscriber::new(DkgValidationContractInstance::new(
        contract_address,
        ws_provider,
    ))
}

pub async fn create_pub_sub_with_signer(http_url: &str, ws_url: &str, contract_address: &str, private_key: &str) -> 
(
    Publisher<impl Provider + Clone>,
    Subscriber<impl Provider + Clone>,
) {
    let parsed_url = Url::parse(http_url).unwrap();
    let signer = LocalSigner::from_str(private_key).unwrap();
    let wallet = EthereumWallet::new(signer);
    let http_provider = ProviderBuilder::new().wallet(wallet).connect_http(parsed_url);
    let contract_address = contract_address
        .parse::<EthAddress>()
        .expect("Invalid contract address");
    let publisher = Publisher::new(DkgValidationContractInstance::new(
        contract_address,
        http_provider,
    ));
    let subscriber = create_subscriber(ws_url, contract_address).await;
    (publisher, subscriber)
}

pub async fn create_pub_sub_without_signer(http_url: &str, ws_url: &str, contract_address: &str) -> (Publisher<impl Provider + Clone>, Subscriber<impl Provider + Clone>) {
    let parsed_url = Url::parse(http_url).unwrap();
    let http_provider = ProviderBuilder::new().connect_http(parsed_url);
    let contract_address = contract_address
        .parse::<EthAddress>()
        .expect("Invalid contract address");
    let publisher = Publisher::new(DkgValidationContractInstance::new(
        contract_address,
        http_provider,
    ));
    let subscriber = create_subscriber(ws_url, contract_address).await;
    (publisher, subscriber)
}

/// Interface of DKG validation
#[async_trait]
pub trait DkgValidation: RestakingValidation {
    /// Type of address this Dkg uses
    type Address: From<Vec<u8>> + Send + Sync + 'static;

    async fn is_solver(&self, who: Self::Address) -> DkgValidationServiceResult<bool>;
    async fn is_operator(&self, who: Self::Address) -> DkgValidationServiceResult<bool>;
    async fn get_session_duration(&self) -> DkgValidationServiceResult<u64>;
    async fn get_collecting_duration(&self) -> DkgValidationServiceResult<u64>;
    async fn get_threshold(&self) -> DkgValidationServiceResult<u16>;
    async fn update_trusted_setup(&self, trusted_setup: Vec<u8>) -> DkgValidationServiceResult<()>;
    async fn get_session_per_round(&self) -> DkgValidationServiceResult<u64>;
    async fn get_active_trusted_setup(&self) -> DkgValidationServiceResult<Vec<u8>>;
    async fn get_solver_info(&self) -> DkgValidationServiceResult<(Self::Address, String, String)>;
    async fn get_active_operators(
        &self,
    ) -> DkgValidationServiceResult<Vec<(Self::Address, String, String)>>;
    async fn is_ready(&self, threshold: u16) -> DkgValidationServiceResult<bool>;
}

#[derive(Clone, Debug)]
pub struct DkgValidationService<Pub, Sub> {
    inner: Arc<DkgValidationServiceInner<Pub, Sub>>,
}

#[derive(Debug)]
pub struct DkgValidationServiceInner<Pub, Sub> {
    publisher: Publisher<Pub>,
    event_sub: Subscriber<Sub>,
}

impl<Pub: Provider, Sub: Provider> DkgValidationService<Pub, Sub> {
    pub fn new(publisher: Publisher<Pub>, event_sub: Subscriber<Sub>) -> Self {
        Self {
            inner: Arc::new(DkgValidationServiceInner {
                publisher,
                event_sub,
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DkgValidationServiceError {
    #[error("Event stream creation failed {0}")]
    EventStreamCreateFailed(RpcError<TransportErrorKind>),
    #[error("Error on contract call")]
    ContractError(#[from] alloy::contract::Error),
    #[error("Error on conversion")]
    ConversionError,
}

#[async_trait]
impl<Pub, Sub> DkgValidation for DkgValidationService<Pub, Sub>
where
    Pub: Provider + Clone + 'static,
    Sub: Provider + Clone + 'static,
{
    type Address = Address;

    async fn is_solver(&self, who: Self::Address) -> DkgValidationServiceResult<bool> {
        self.inner
            .publisher
            .is_solver(EthAddress::from_slice(&who.0))
            .await
    }
    async fn is_operator(&self, who: Self::Address) -> DkgValidationServiceResult<bool> {
        self.inner
            .publisher
            .is_operator(EthAddress::from_slice(&who.0))
            .await
    }
    async fn get_session_duration(&self) -> DkgValidationServiceResult<u64> {
        self.inner.publisher.get_session_duration().await
    }
    async fn get_collecting_duration(&self) -> DkgValidationServiceResult<u64> {
        self.inner.publisher.get_collecting_duration().await
    }
    async fn get_threshold(&self) -> DkgValidationServiceResult<u16> {
        self.inner.publisher.get_threshold().await
    }
    async fn update_trusted_setup(&self, trusted_setup: Vec<u8>) -> DkgValidationServiceResult<()> {
        self.inner
            .publisher
            .update_trusted_setup(trusted_setup)
            .await
    }
    async fn get_session_per_round(&self) -> DkgValidationServiceResult<u64> {
        self.inner.publisher.get_session_per_round().await
    }
    async fn get_active_trusted_setup(&self) -> DkgValidationServiceResult<Vec<u8>> {
        self.inner.publisher.get_active_trusted_setup().await
    }
    async fn get_solver_info(&self) -> DkgValidationServiceResult<(Self::Address, String, String)> {
        let info = self.inner.publisher.get_solver_info().await?;
        Ok((info.0.to_vec().into(), info.1, info.2))
    }
    async fn get_active_operators(
        &self,
    ) -> DkgValidationServiceResult<Vec<(Self::Address, String, String)>> {
        let operators = self.inner.publisher.get_active_operators().await?;
        Ok(operators
            .into_iter()
            .map(|o| (o.0.to_vec().into(), o.1, o.2))
            .collect())
    }
    async fn is_ready(&self, threshold: u16) -> DkgValidationServiceResult<bool> {
        self.inner.publisher.is_ready(threshold).await
    }
}

#[async_trait]
impl<Pub, Sub> RestakingValidation for DkgValidationService<Pub, Sub>
where
    Pub: Provider + 'static,
    Sub: Provider + 'static,
{
    type Proof = Vec<u8>;
    type Key = SessionId;
    type Event = DkgValidationServiceEvent;

    async fn create_task(
        &self,
        key: Self::Key,
        proof: Self::Proof,
    ) -> RestakingValidationServiceResult<()> {
        self.inner.publisher.create_new_task(key, proof).await;
        Ok(())
    }

    async fn respond_task(&self, key: Self::Key) -> RestakingValidationServiceResult<()> {
        self.inner.publisher.respond_task(0, key).await;
        Ok(())
    }

    async fn subscribe_events<CB, Fut>(&self, callback: CB) -> RestakingValidationServiceResult<()>
    where
        CB: Send + Fn(Self::Event) -> Fut,
        Fut: Send + Future<Output = ()>,
    {
        self.inner.event_sub.subscribe_events(callback).await;
        Ok(())
    }
}
