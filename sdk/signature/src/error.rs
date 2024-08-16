#[derive(Debug)]
pub enum Error {
    UnsupportedChainId,
    ParsePrivateKey(const_hex::FromHexError),
    InitializePrivateKey(Box<dyn std::error::Error>),
    SignMessage(Box<dyn std::error::Error>),
    SignatureOutOfBound,
    ParseSignature(Box<dyn std::error::Error>),
    ParseVerifyingKey(Box<dyn std::error::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
