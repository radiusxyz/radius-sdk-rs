use std::{future::Future, sync::Arc};

use hyper::{header, Method};
use jsonrpsee::{
    server::{middleware::http::ProxyGetRequestLayer, Server, ServerHandle},
    types::{ErrorCode, ErrorObjectOwned, Params},
    IntoResponse, RpcModule,
};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use url::Url;

pub type RpcParameter = Params<'static>;

pub struct RpcError(Box<dyn std::error::Error>);

impl<E> From<E> for RpcError
where
    E: std::error::Error + 'static,
{
    fn from(value: E) -> Self {
        Self(Box::new(value))
    }
}

impl From<RpcError> for ErrorObjectOwned {
    fn from(value: RpcError) -> Self {
        ErrorObjectOwned::owned::<u8>(ErrorCode::InternalError.code(), value, None)
    }
}

impl From<RpcError> for String {
    fn from(value: RpcError) -> Self {
        value.0.to_string()
    }
}

pub struct RpcServer<C>
where
    C: Send + Sync + 'static,
{
    rpc_module: RpcModule<C>,
}

impl<C> RpcServer<C>
where
    C: Send + Sync + 'static,
{
    pub fn new(context: C) -> Self {
        Self {
            rpc_module: RpcModule::new(context),
        }
    }

    pub fn register_rpc_method<H, F, R>(
        mut self,
        method: &'static str,
        handler: H,
    ) -> Result<Self, RpcServerError>
    where
        H: Fn(RpcParameter, Arc<C>) -> F + Clone + Send + Sync + 'static,
        F: Future<Output = R> + Send + 'static,
        R: IntoResponse + 'static,
    {
        self.rpc_module
            .register_async_method(method, handler)
            .map_err(RpcServerError::RegisterRpcMethod)?;

        Ok(self)
    }

    pub async fn init(self, rpc_url: impl AsRef<str>) -> Result<ServerHandle, RpcServerError> {
        let rpc_url = Url::parse(rpc_url.as_ref()).map_err(ParseUrlError::ParseRpcUrl)?;
        let rpc_url = format!(
            "{}:{}",
            rpc_url.host_str().ok_or(ParseUrlError::InvalidHost)?,
            rpc_url.port().ok_or(ParseUrlError::InvalidPort)?,
        );

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any)
            .allow_headers([header::CONTENT_TYPE]);

        let middleware = ServiceBuilder::new().layer(cors).layer(
            ProxyGetRequestLayer::new("/health", "health")
                .map_err(RpcServerError::RpcMiddleware)?,
        );

        let server = Server::builder()
            .set_http_middleware(middleware)
            .build(rpc_url)
            .await
            .map_err(RpcServerError::Initialize)?;

        Ok(server.start(self.rpc_module))
    }
}

#[derive(Debug)]
pub enum ParseUrlError {
    ParseRpcUrl(url::ParseError),
    InvalidHost,
    InvalidPort,
}

impl std::fmt::Display for ParseUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseUrlError {}

#[derive(Debug)]
pub enum RpcServerError {
    ParseRpcUrl(ParseUrlError),
    RegisterRpcMethod(jsonrpsee::core::RegisterMethodError),
    RpcMiddleware(jsonrpsee::server::middleware::http::InvalidPath),
    Initialize(std::io::Error),
}

impl std::fmt::Display for RpcServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RpcServerError {}

impl From<ParseUrlError> for RpcServerError {
    fn from(value: ParseUrlError) -> Self {
        Self::ParseRpcUrl(value)
    }
}
