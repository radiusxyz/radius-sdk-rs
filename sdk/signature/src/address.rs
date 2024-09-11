use serde::{Deserialize, Serialize};

use crate::{error::Error, platform::*};

pub(crate) trait AddressTrait {
    fn from_slice(&self, slice: &[u8]) -> Result<Address, Error>;

    fn from_str(&self, str: &str) -> Result<Address, Error>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Address(Vec<u8>);

impl<T> std::cmp::PartialEq<T> for Address
where
    T: AsRef<[u8]>,
{
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

impl Address {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn from_slice(platform: Platform, slice: &[u8]) -> Result<Self, Error> {
        platform.address().from_slice(slice)
    }

    pub fn from_str(platform: Platform, str: &str) -> Result<Self, Error> {
        platform.address().from_str(str)
    }
}
