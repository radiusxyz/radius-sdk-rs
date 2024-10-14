#[derive(Debug)]
pub enum RpcClientError {
    BuildClient(reqwest::Error),
    Send(reqwest::Error),
    ParseResponse(reqwest::Error),
    IdMismatch,
    Response(crate::client::response::ResponseError),
}

impl std::fmt::Display for RpcClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RpcClientError {}

impl From<crate::client::response::ResponseError> for RpcClientError {
    fn from(value: crate::client::response::ResponseError) -> Self {
        Self::Response(value)
    }
}
