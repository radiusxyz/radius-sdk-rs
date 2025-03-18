#[derive(Debug)]
pub enum RpcClientError {
    Initialize(reqwest::Error),
    Request(reqwest::Error),
    ParseResponse(reqwest::Error),
    Response(String),
    IdMismatch,
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    Fetch(Box<dyn std::error::Error>),
    ChannelRecv,
}

unsafe impl Send for RpcClientError {}

impl std::fmt::Display for RpcClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RpcClientError {}
