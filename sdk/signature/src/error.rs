#[derive(Debug)]
pub enum Error {
    UnsupportedChainType(crate::chain::ChainType),
    ParsePrivateKey(crate::chain::ChainType, const_hex::FromHexError),
    InitializePrivateKey(crate::chain::ChainType, Box<dyn std::error::Error>),
    SignMessage(crate::chain::ChainType, Box<dyn std::error::Error>),
    SignatureOutOfBound,
    ParseSignature(crate::chain::ChainType, Box<dyn std::error::Error>),
    ParseRecoveryId(crate::chain::ChainType, u8),
    RecoverVerifyingKey(crate::chain::ChainType, Box<dyn std::error::Error>),
    AddressMismatch(crate::chain::ChainType),
    BytesToHexString(std::fmt::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
