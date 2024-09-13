use serde::{Deserialize, Serialize};

use crate::{chain_type::*, error::SignatureError, Verifier};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature(Vec<u8>);

impl From<&[u8]> for Signature {
    fn from(value: &[u8]) -> Self {
        Self(value.to_owned())
    }
}

impl From<Vec<u8>> for Signature {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Signature {
    pub fn verify_message<T: Serialize>(
        &self,
        platform: ChainType,
        message: &T,
        address: impl AsRef<[u8]>,
    ) -> Result<(), SignatureError> {
        let message_bytes =
            bincode::serialize(message).map_err(SignatureError::SerializeMessage)?;

        platform
            .verifier()
            .verify_message(&self.0, &message_bytes, address.as_ref())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
