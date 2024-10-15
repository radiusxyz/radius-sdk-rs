#[derive(Debug)]
pub enum SignatureError {
    DeserializeAddress(const_hex::FromHexError),
    DeserializeSignature(const_hex::FromHexError),
    SerializeMessage(bincode::Error),
    Ethereum(crate::chain_type::ethereum::EthereumError),
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SignatureError {}

impl From<crate::chain_type::ethereum::EthereumError> for SignatureError {
    fn from(value: crate::chain_type::ethereum::EthereumError) -> Self {
        Self::Ethereum(value)
    }
}
