use std::{fmt, hash::Hash};

use serde::{Deserialize, Serialize, Serializer};

use crate::{chain_type::*, error::SignatureError, Builder};

#[derive(Clone, Eq, Hash, Deserialize)]
#[serde(try_from = "AddressType")]
pub struct Address(Vec<u8>);

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_hex_string())
    }
}

impl Default for Address {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_hex_string())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_hex_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum AddressType {
    Array(Vec<u8>),
    String(String),
}

impl<T: AsRef<[u8]>> std::cmp::PartialEq<T> for Address {
    fn eq(&self, other: &T) -> bool {
        self.0.as_slice() == other.as_ref()
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<Vec<u8>> for Address {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl TryFrom<AddressType> for Address {
    type Error = SignatureError;

    fn try_from(value: AddressType) -> Result<Self, Self::Error> {
        match value {
            AddressType::Array(address) => Ok(Self(address)),
            AddressType::String(address) => {
                let address =
                    const_hex::decode(address).map_err(SignatureError::DeserializeAddress)?;

                Ok(Self(address))
            }
        }
    }
}

impl Address {
    pub fn from_slice(chain_type: ChainType, slice: &[u8]) -> Result<Self, SignatureError> {
        chain_type.address_builder().build_from_slice(slice)
    }

    pub fn from_str(chain_type: ChainType, str: &str) -> Result<Self, SignatureError> {
        chain_type.address_builder().build_from_str(str)
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
