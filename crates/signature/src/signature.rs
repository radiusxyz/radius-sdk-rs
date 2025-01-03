use serde::{Deserialize, Serialize};

use crate::{chain_type::*, error::SignatureError, Verifier};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "SignatureType")]
pub struct Signature(Vec<u8>);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum SignatureType {
    Array(Vec<u8>),
    String(String),
}

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

impl TryFrom<SignatureType> for Signature {
    type Error = SignatureError;

    fn try_from(value: SignatureType) -> Result<Self, Self::Error> {
        match value {
            SignatureType::Array(signature) => Ok(Self(signature)),
            SignatureType::String(signature) => {
                let signature =
                    const_hex::decode(signature).map_err(SignatureError::DeserializeSignature)?;

                Ok(Self(signature))
            }
        }
    }
}

impl Signature {
    pub fn verify_message<T: Serialize>(
        &self,
        chain_type: ChainType,
        message: &T,
        address: impl AsRef<[u8]>,
    ) -> Result<(), SignatureError> {
        let message_bytes =
            bincode::serialize(message).map_err(SignatureError::SerializeMessage)?;

        chain_type
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

    pub fn as_hex_string(&self) -> String {
        const_hex::encode_prefixed(&self.0)
    }
}
