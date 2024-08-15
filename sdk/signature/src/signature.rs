use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature {
    pub curve: Curve,
    pub signature: Vec<u8>,
}

impl Signature {
    pub fn verify_address_from_message(
        &self,
        address_type: AddressType,
        address: impl AsRef<[u8]>,
        message: impl AsRef<[u8]>,
    ) -> Result<bool, Error> {
        match self.curve {
            Curve::Secp256k1 => crate::ecdsa::secp256k1::verify_address(
                &self.signature,
                address_type,
                address.as_ref(),
                message.as_ref(),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Curve {
    Secp256k1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum AddressType {
    Ethereum,
}
