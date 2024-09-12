use std::sync::Arc;

use serde::Serialize;

use crate::{address::Address, error::Error, platform::Platform, signature::Signature, traits::*};

pub struct PrivateKeySigner {
    inner: Arc<dyn Signer>,
}

unsafe impl Send for PrivateKeySigner {}

unsafe impl Sync for PrivateKeySigner {}

impl Clone for PrivateKeySigner {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> From<T> for PrivateKeySigner
where
    T: Signer + 'static,
{
    fn from(value: T) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }
}

impl PrivateKeySigner {
    pub fn from_slice(platform: Platform, private_key: &[u8]) -> Result<Self, Error> {
        platform.signer_builder().build_from_slice(private_key)
    }

    pub fn from_str(platform: Platform, private_key: &str) -> Result<Self, Error> {
        platform.signer_builder().build_from_str(private_key)
    }

    pub fn from_random(platform: Platform) -> Result<(Self, String), Error> {
        platform.signer_builder_random().build_from_random()
    }

    pub fn address(&self) -> &Address {
        self.inner.address()
    }

    pub fn sign_message<T>(&self, message: T) -> Result<Signature, Error>
    where
        T: Serialize,
    {
        let message_bytes = bincode::serialize(&message).map_err(Error::SerializeMessage)?;

        self.inner.sign_message(&message_bytes)
    }
}
