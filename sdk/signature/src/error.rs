#[derive(Debug)]
pub enum SignatureError {
    SerializeMessage(bincode::Error),
    Ethereum(crate::platform::ethereum::EthereumError),
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SignatureError {}

impl From<crate::platform::ethereum::EthereumError> for SignatureError {
    fn from(value: crate::platform::ethereum::EthereumError) -> Self {
        Self::Ethereum(value)
    }
}
