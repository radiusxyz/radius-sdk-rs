use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::{chain_type::*, error::SignatureError, Builder};

#[derive(Clone, Debug, Eq, Hash, Deserialize, Serialize)]
#[serde(try_from = "AddressType")]
pub struct Address(Vec<u8>);

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
