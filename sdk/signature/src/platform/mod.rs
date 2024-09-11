pub(crate) mod ethereum;

use serde::{Deserialize, Serialize};

use crate::{address::AddressTrait, signature::SignatureTrait, signer::SignerTrait};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Ethereum,
}

impl Platform {
    pub(crate) fn address(&self) -> impl AddressTrait {
        match self {
            Self::Ethereum => ethereum::EthereumAddress,
        }
    }

    pub(crate) fn signature(&self) -> impl SignatureTrait {
        match self {
            Self::Ethereum => ethereum::EthereumSignature,
        }
    }

    // pub(crate) fn signer(&self) -> impl SignerTrait {
    //     match self {
    //         Self::Ethereum => ethereum::EthereumSigner,
    //     }
    // }
}
