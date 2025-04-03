use jsonrpsee::types::{ErrorCode, ErrorObject};

#[derive(Debug)]
pub struct RpcError(Box<dyn std::error::Error + Send + 'static>);

unsafe impl Send for RpcError {}

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
