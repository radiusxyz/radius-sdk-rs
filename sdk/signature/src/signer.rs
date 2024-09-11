use std::sync::Arc;

use serde::Serialize;

use crate::{address::Address, error::Error, platform::Platform};

pub trait SignerTrait {
    fn from_slice(&self, slice: &[u8]) -> Result<Signer, Error>
    where
        Self: Sized;

    fn from_str(&self, str: &str) -> Result<Signer, Error>
    where
        Self: Sized;
}

pub struct Signer {
    inner: Arc<SignerInner>,
}

struct SignerInner {
    address: Address,
}

unsafe impl Send for Signer {}

unsafe impl Sync for Signer {}

impl Clone for Signer {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// impl Signer {
//     pub fn from_slice(platform: Platform, signing_key: &[u8]) {}

//     pub fn from_str(platform: Platform, signing_key: &str) {}

//     pub fn address(&self) -> Address {
//         self.inner.address().into()
//     }

//     pub fn sign_message<T: Serialize>(&self, message: &T) -> Result<(),
// Error> {         let message_bytes = bincode::serialize(message).unwrap();
//         self.inner.sign_message(&message_bytes).unwrap();

//         Ok(())
//     }
// }
