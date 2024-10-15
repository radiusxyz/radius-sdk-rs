pub(crate) mod ethereum;

use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::{address::Address, signer::PrivateKeySigner, traits::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChainType {
    Ethereum,
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
