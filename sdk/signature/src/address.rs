use crate::{chain::*, error::Error};

pub struct Address {
    address: Vec<u8>,
    chain_id: ChainId,
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

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.address)
    }
}

impl Clone for Address {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
            chain_id: self.chain_id,
        }
    }
}

impl From<(Vec<u8>, ChainId)> for Address {
    fn from(value: (Vec<u8>, ChainId)) -> Self {
        Self {
            address: value.0,
            chain_id: value.1,
        }
    }
}

impl Address {
    pub fn from_slice(slice: &[u8], chain_id: ChainId) -> Result<Self, Error> {
        match chain_id {
            ChainId::Bitcoin => Err(Error::UnsupportedChainId(chain_id)),
            ChainId::Ethereum => Ok((ethereum::ethereum_address(slice), chain_id).into()),
        }
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn len(&self) -> usize {
        self.address.len()
    }
}
