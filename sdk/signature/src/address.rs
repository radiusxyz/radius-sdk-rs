use serde::{Deserialize, Serialize};

use crate::{error::Error, platform::*, Builder};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Address(Vec<u8>);

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

impl Address {
    pub fn from_slice(platform: Platform, slice: &[u8]) -> Result<Self, Error> {
        platform.address_builder().build_from_slice(slice)
    }

    pub fn from_str(platform: Platform, str: &str) -> Result<Self, Error> {
        platform.address_builder().build_from_str(str)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // pub fn to_hex_string(&self) -> String {
    //     self.0.iter().try_for_each(|byte| )
    // }
}
