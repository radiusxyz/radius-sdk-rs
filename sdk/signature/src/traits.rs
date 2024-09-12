use crate::{error::Error, signature::Signature};

pub trait Builder {
    type Output;

    fn from_slice(&self, slice: &[u8]) -> Result<Self::Output, Error>;

    fn from_str(&self, str: &str) -> Result<Self::Output, Error>;
}

pub trait Signer {
    fn sign_message(&self, message: &[u8]) -> Result<Signature, Error>;
}

pub trait Verifier {
    fn verify_message(&self, signature: &[u8], message: &[u8], address: &[u8])
        -> Result<(), Error>;
}
