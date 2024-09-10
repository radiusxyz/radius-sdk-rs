pub mod ethereum;

use serde::{Deserialize, Serialize};

use crate::{ecdsa, error::Error, PrivateKeySigner};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Bitcoin,
    Ethereum,
}

impl ChainType {
    pub fn create_signer_from_slice(
        self,
        private_key: &[u8],
    ) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainType(ChainType::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::from_slice(private_key, self)
            }
        }
    }

    pub fn create_signer_from_str(self, private_key: &str) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainType(ChainType::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::from_str(private_key, self)
            }
        }
    }

    pub fn create_signer_random(self) -> Result<(impl PrivateKeySigner, Vec<u8>), Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainType(ChainType::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::generate_random(self)
            }
        }
    }
}
