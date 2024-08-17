use serde::{Deserialize, Serialize};

use crate::{chain::*, error::Error};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature(Vec<u8>);

impl From<&[u8]> for Signature {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

impl From<Vec<u8>> for Signature {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Signature {
    pub fn verify_signature(
        &self,
        message: &[u8],
        address: &[u8],
        chain_id: ChainId,
    ) -> Result<(), Error> {
        match chain_id {
            ChainId::Bitcoin => Err(Error::UnsupportedChainId(chain_id)),
            ChainId::Ethereum => {
                crate::ecdsa::secp256k1::verify(&self.0, message, address, chain_id)
            }
        }
    }
}
