pub mod ecdsa;
mod error;
mod signature;

pub use error::Error;
pub use signature::{AddressType, Curve, Signature};

pub trait Signer
where
    Self: Sized,
{
    fn from_str(signing_key: &str) -> Result<Self, Error>;

    fn sign_message(&self, message: &[u8]) -> Result<Signature, Error>;
}
