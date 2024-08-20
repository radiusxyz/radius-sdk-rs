use crate::{chain::*, error::Error, util::*};

pub struct Address {
    address: Vec<u8>,
    chain_type: ChainType,
}

impl<T: AsRef<[u8]>> std::cmp::PartialEq<T> for Address {
    fn eq(&self, other: &T) -> bool {
        self.address == other.as_ref()
    }
}

impl std::cmp::PartialEq<[u8]> for Address {
    fn eq(&self, other: &[u8]) -> bool {
        self.address == other
    }
}

impl std::cmp::PartialEq<Self> for Address {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.chain_type {
            ChainType::Bitcoin => write!(f, "{:?}", self.address),
            ChainType::Ethereum => fmt_hex_string(f, &self.address),
        }
    }
}

impl Clone for Address {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            chain_type: self.chain_type,
        }
    }
}

impl From<(Vec<u8>, ChainType)> for Address {
    fn from(value: (Vec<u8>, ChainType)) -> Self {
        Self {
            address: value.0,
            chain_type: value.1,
        }
    }
}

impl Address {
    pub fn from_slice(slice: &[u8], chain_type: ChainType) -> Result<Self, Error> {
        match chain_type {
            ChainType::Bitcoin => Err(Error::UnsupportedChainType(chain_type)),
            ChainType::Ethereum => Ok((ethereum::address_from_slice(slice), chain_type).into()),
        }
    }

    pub fn chain_type(&self) -> ChainType {
        self.chain_type
    }

    pub fn len(&self) -> usize {
        self.address.len()
    }

    pub fn is_empty(&self) -> bool {
        self.address.is_empty()
    }
}
