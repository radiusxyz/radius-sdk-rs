use std::sync::Arc;

use crate::{error::Error, platform::Platform, traits::*};

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
        platform.signer_builder().from_slice(private_key)
    }

    pub fn from_str(platform: Platform, private_key: &str) -> Result<Self, Error> {
        platform.signer_builder().from_str(private_key)
    }

    // pub fn sign_message(&self) -> Result<Signature, Error> {}
}
