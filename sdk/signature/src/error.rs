#[derive(Debug)]
pub enum Error {
    UnsupportedPlatform(crate::platform::Platform),
    SerializeMessage(bincode::Error),
    Ethereum(crate::platform::ethereum::EthereumError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<crate::platform::ethereum::EthereumError> for Error {
    fn from(value: crate::platform::ethereum::EthereumError) -> Self {
        Self::Ethereum(value)
    }
}
