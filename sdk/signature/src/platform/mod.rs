pub(crate) mod ethereum;

use serde::{Deserialize, Serialize};

use crate::{address::Address, error::Error, signer::PrivateKeySigner, traits::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Ethereum,
}

impl Platform {
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

    pub(crate) fn verifier(&self) -> impl Verifier {
        match self {
            Self::Ethereum => ethereum::EthereumVerifier,
        }
    }
}
