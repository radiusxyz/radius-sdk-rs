#[derive(Debug)]
pub enum Error {
    ParseSigningKey(crate::signature::Curve, const_hex::FromHexError),
    InitializeSigningKey(crate::signature::Curve, Box<dyn std::error::Error>),
    SignMessage(crate::signature::Curve, Box<dyn std::error::Error>),
    SignatureOutOfBound,
    ParseSignature(crate::signature::Curve, Box<dyn std::error::Error>),
    ParseVerifyingKey(crate::signature::Curve, Box<dyn std::error::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
