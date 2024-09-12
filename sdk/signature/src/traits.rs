use crate::{address::Address, error::SignatureError, signature::Signature};

pub trait Builder {
    type Output;

    fn build_from_slice(&self, slice: &[u8]) -> Result<Self::Output, SignatureError>;

    fn build_from_str(&self, str: &str) -> Result<Self::Output, SignatureError>;
}

pub trait RandomBuilder {
    type Output;

    fn build_from_random(&self) -> Result<Self::Output, SignatureError>;
}

pub trait Signer {
    fn address(&self) -> &Address;

    fn sign_message(&self, message: &[u8]) -> Result<Signature, SignatureError>;
}

pub trait Verifier {
    fn verify_message(
        &self,
        signature: &[u8],
        message: &[u8],
        address: &[u8],
    ) -> Result<(), SignatureError>;
}
