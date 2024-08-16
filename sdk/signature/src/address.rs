use sha3::{Digest, Keccak256};

use crate::{error::Error, ChainId};

pub struct Address(Vec<u8>);

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Address {
    pub fn from_vec(vec: Vec<u8>, chain_id: ChainId) -> Result<Self, Error> {
        match chain_id {
            ChainId::Bitcoin => Err(Error::UnsupportedChainId),
            ChainId::Ethereum => Ok(Self::ethereum_address(vec)),
        }
    }

    fn ethereum_address(vec: Vec<u8>) -> Self {
        let mut hasher = Keccak256::new();
        hasher.update(&vec[1..]);
        let output = hasher.finalize_reset()[12..].to_vec();

        Self(output)
    }
}
