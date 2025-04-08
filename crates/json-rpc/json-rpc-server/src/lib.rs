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
use jsonrpsee::{
    server::{middleware::http::ProxyGetRequestLayer, RpcModule, Server, ServerHandle},
    types::Params,
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::{Mutex, Semaphore},
    task,
    time::{sleep, Duration},
};
use tower_http::cors::{Any, CorsLayer};
pub use types::*;
use url::Url;

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

// ---- RpcServer ----

pub struct RpcServer<C>
where
    C: Clone + Send + Sync + 'static,
{
    rpc_module: Mutex<RpcModule<C>>,
    task_sequence: AtomicU64,
    task_queue: Mutex<BinaryHeap<PrioritizedRpcTask>>,
    max_concurrent_requests: Semaphore,
}

impl<C> RpcServer<C>
where
    C: Clone + Send + Sync + 'static,
{
    pub fn new(context: C) -> Self {
        let concurrency = num_cpus::get() * 4;

        Self {
            rpc_module: Mutex::new(RpcModule::new(context)),
            task_sequence: AtomicU64::new(0),
            task_queue: Mutex::new(BinaryHeap::new()),
            max_concurrent_requests: Semaphore::new(concurrency),
        }
    }

    pub async fn register_rpc_method<P>(self: &Arc<Self>) -> Result<(), RpcServerError>
    where
        P: RpcParameter<C> + 'static,
    {
        let server = self.clone();
        let mut module = self.rpc_module.lock().await;

        module
            .register_async_method(P::method(), move |params, ctx, ext| {
                let server = server.clone();
                Box::pin(async move { RpcServer::handler::<P>(server, params, ctx, ext).await })
            })
            .map_err(RpcServerError::RegisterMethod)?;

        Ok(())
    }

    async fn handler<P>(
        server: Arc<Self>,
        parameter: Params<'static>,
        context: Arc<C>,
        _extensions: Extensions,
    ) -> Result<P::Response, RpcError>
    where
        P: RpcParameter<C> + 'static,
    {
        let parsed = parameter.parse::<P>()?;
        let priority = parsed.priority();
        let sequence = server.task_sequence.fetch_add(1, AtomicOrdering::Relaxed);
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
            let mut queue = server.task_queue.lock().await;
            queue.push(task);
        }

        rx.await.map_err(|e| {
            let io_err =
                std::io::Error::new(std::io::ErrorKind::Other, format!("task canceled: {e}"));
            RpcError::from(io_err)
        })?
    }

    pub async fn init(
        self: Arc<Self>,
        rpc_url: impl AsRef<str>,
    ) -> Result<ServerHandle, RpcServerError> {
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

        tokio::spawn(process_rpc_tasks(self.clone()));

        let rpc_module = self.rpc_module.lock().await.clone();
        let handle = server.start(rpc_module);
        Ok(handle)
    }
}

// ---- Background Task Runner ----

async fn process_rpc_tasks<C>(server: Arc<RpcServer<C>>)
where
    C: Clone + Send + Sync + 'static,
{
    loop {
        let task = {
            let mut queue = server.task_queue.lock().await;
            queue.pop()
        };

        if let Some(task) = task {
            let permit = server.max_concurrent_requests.acquire().await.unwrap();
            (task.job)();
            drop(permit);
        } else {
            sleep(Duration::from_millis(10)).await;
        }
    }
}
