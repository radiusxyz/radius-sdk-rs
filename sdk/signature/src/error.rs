#[derive(Debug)]
pub enum Error {
    UnsupportedChainId(crate::chain::ChainId),
    ParsePrivateKey(crate::chain::ChainId, const_hex::FromHexError),
    InitializePrivateKey(crate::chain::ChainId, Box<dyn std::error::Error>),
    SignMessage(crate::chain::ChainId, Box<dyn std::error::Error>),
    SignatureOutOfBound,
    ParseSignature(crate::chain::ChainId, Box<dyn std::error::Error>),
    ParseRecoveryId(crate::chain::ChainId, u8),
    RecoverVerifyingKey(crate::chain::ChainId, Box<dyn std::error::Error>),
    AddressMismatch(crate::chain::ChainId),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
