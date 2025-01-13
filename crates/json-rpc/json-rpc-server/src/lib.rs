use std::{future::Future, str::FromStr, sync::Arc};

use http::{header, method::Method, Extensions};
pub use jsonrpsee::server::ServerHandle;
use jsonrpsee::{
    server::{middleware::http::ProxyGetRequestLayer, RpcModule, Server},
    types::{ErrorCode, ErrorObject, Params},
};
use serde::{de::DeserializeOwned, Serialize};
use tower_http::cors::{Any, CorsLayer};
use url::Url;

pub trait RpcParameter<C>: DeserializeOwned + Serialize
where
    C: Clone + Send + Sync + 'static,
{
    type Response: Clone + Send + 'static + DeserializeOwned + Serialize;

    fn method() -> &'static str;

    fn handler(self, context: C) -> impl Future<Output = Result<Self::Response, RpcError>> + Send;
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
        let parameter = parameter.parse::<P>()?;

        P::handler(parameter, (*context).clone()).await
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
            .build(rpc_url)
            .await
            .map_err(RpcServerError::Initialize)?;
        let server_handle = server.start(self.rpc_module);

        Ok(server_handle)
    }
}

#[derive(Debug)]
pub struct RpcError(Box<dyn std::error::Error + Send + 'static>);

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<RpcError> for String {
    fn from(value: RpcError) -> Self {
        value.to_string()
    }
}

impl From<RpcError> for ErrorObject<'static> {
    fn from(value: RpcError) -> Self {
        ErrorObject::owned::<i32>(ErrorCode::InternalError.code(), value, None)
    }
}

impl<T> From<T> for RpcError
where
    T: std::error::Error + Send + 'static,
{
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

#[derive(Debug)]
pub enum RpcServerError {
    Middleware(jsonrpsee::server::middleware::http::InvalidPath),
    Parse(ParseError),
    RegisterMethod(jsonrpsee::server::RegisterMethodError),
    Initialize(std::io::Error),
}

impl std::fmt::Display for RpcServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RpcServerError {}

impl From<ParseError> for RpcServerError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidHost,
    InvalidPort,
    InvalidRpcUrl(url::ParseError),
}
