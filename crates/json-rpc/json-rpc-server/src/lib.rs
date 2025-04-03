mod error;
mod types;

use std::{
    collections::BinaryHeap,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering as AtomicOrdering},
        Arc,
    },
};

pub use error::*;
use http::{header, method::Method, Extensions};
pub use jsonrpsee::server::ServerHandle;
use jsonrpsee::{
    server::{middleware::http::ProxyGetRequestLayer, RpcModule, Server},
    types::Params,
};
use num_cpus;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::{Mutex, Semaphore},
    task,
    time::{sleep, Duration},
};
use tower_http::cors::{Any, CorsLayer};
pub use types::*;
use url::Url;

static TASK_SEQUENCE: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));

#[trait_variant::make(RpcParameter: Send)]
pub trait LocalRpcParameter<C>: DeserializeOwned + Serialize
where
    C: Clone + Send + Sync + 'static,
{
    type Response: Clone + Send + 'static + DeserializeOwned + Serialize;

    fn method() -> &'static str;

    fn priority(&self) -> ProcessPriority {
        ProcessPriority::Normal
    }

    async fn handler(self, context: C) -> Result<Self::Response, RpcError>;
}

static TASK_QUEUE: Lazy<Mutex<BinaryHeap<PrioritizedRpcTask>>> =
    Lazy::new(|| Mutex::new(BinaryHeap::new()));

static MAX_CONCURRENT_REQUESTS: Lazy<Semaphore> = Lazy::new(|| {
    let concurrency = num_cpus::get() * 4; // 또는 2 ~ 8 등 상황 맞게
    Semaphore::new(concurrency)
});

async fn process_rpc_tasks() {
    loop {
        let task = {
            let mut queue = TASK_QUEUE.lock().await;
            queue.pop()
        };

        if let Some(task) = task {
            let permit = MAX_CONCURRENT_REQUESTS.acquire().await.unwrap();
            (task.job)();
            drop(permit);
        } else {
            sleep(Duration::from_millis(10)).await;
        }
    }
}

pub struct RpcServer<C>
where
    C: Clone + Send + Sync + 'static,
{
    rpc_module: RpcModule<C>,
}

impl<C> RpcServer<C>
where
    C: Clone + Send + Sync + 'static,
{
    pub fn new(context: C) -> Self {
        Self {
            rpc_module: RpcModule::new(context),
        }
    }

    async fn handler<P>(
        parameter: Params<'static>,
        context: Arc<C>,
        _extensions: Extensions,
    ) -> Result<P::Response, RpcError>
    where
        P: RpcParameter<C> + 'static,
    {
        let parsed = parameter.parse::<P>()?;
        let priority = parsed.priority();
        let sequence = TASK_SEQUENCE.fetch_add(1, AtomicOrdering::Relaxed);
        let ctx = (*context).clone();

        let (tx, rx) = tokio::sync::oneshot::channel();
        let task = PrioritizedRpcTask {
            priority,
            sequence,
            job: Box::new(move || {
                task::spawn(async move {
                    let result = P::handler(parsed, ctx).await;
                    let _ = tx.send(result);
                });
            }),
        };

        {
            let mut queue = TASK_QUEUE.lock().await;
            queue.push(task);
        }

        rx.await.map_err(|e| {
            let io_err =
                std::io::Error::new(std::io::ErrorKind::Other, format!("task canceled: {e}"));
            RpcError::from(io_err)
        })?
    }

    pub fn register_rpc_method<P>(mut self) -> Result<Self, RpcServerError>
    where
        P: RpcParameter<C> + 'static,
    {
        self.rpc_module
            .register_async_method(P::method(), Self::handler::<P>)
            .map_err(RpcServerError::RegisterMethod)?;

        Ok(self)
    }

    pub async fn init(self, rpc_url: impl AsRef<str>) -> Result<ServerHandle, RpcServerError> {
        let rpc_url = match Url::from_str(rpc_url.as_ref()) {
            Ok(url) => format!(
                "{}:{}",
                url.host_str().ok_or(ParseError::InvalidHost)?,
                url.port().ok_or(ParseError::InvalidPort)?,
            ),
            Err(error) => {
                if error == url::ParseError::RelativeUrlWithoutBase {
                    rpc_url.as_ref().to_owned()
                } else {
                    return Err(ParseError::InvalidRpcUrl(error).into());
                }
            }
        };

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers([header::CONTENT_TYPE]);
        let health_check =
            ProxyGetRequestLayer::new("/health", "health").map_err(RpcServerError::Middleware)?;
        let middleware = tower::ServiceBuilder::new().layer(cors).layer(health_check);

        let server = Server::builder()
            .set_http_middleware(middleware)
            .max_connections(1000)
            .build(rpc_url)
            .await
            .map_err(RpcServerError::Initialize)?;

        // Start background task processor
        tokio::spawn(process_rpc_tasks());

        let server_handle = server.start(self.rpc_module);

        Ok(server_handle)
    }
}
