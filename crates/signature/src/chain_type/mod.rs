pub(crate) mod ethereum;

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::{address::Address, signer::PrivateKeySigner, traits::*, SignatureError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(try_from = "String")]
pub enum ChainType {
    Ethereum,
}

impl TryFrom<String> for ChainType {
    type Error = SignatureError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "ethereum" => Ok(Self::Ethereum),
            _others => Err(SignatureError::UnsupportedChainType(value)),
        }
    }
}
impl ChainType {
    pub(crate) fn address_builder(&self) -> impl Builder<Output = Address> {
        match self {
            Self::Ethereum => ethereum::EthereumAddressBuilder,
        }
    }

    pub(crate) fn signer_builder(&self) -> impl Builder<Output = PrivateKeySigner> {
        match self {
            Self::Ethereum => ethereum::EthereumSignerBuilder,
        }
    }

    pub(crate) fn signer_builder_random(
        &self,
    ) -> impl RandomBuilder<Output = (PrivateKeySigner, String)> {
        match self {
            Self::Ethereum => ethereum::EthereumSignerBuilder,
        }
    }

    pub(crate) fn verifier(&self) -> impl Verifier {
        match self {
            Self::Ethereum => ethereum::EthereumVerifier,
        }
    }
}
