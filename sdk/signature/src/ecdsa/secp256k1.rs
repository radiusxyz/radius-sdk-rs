use alloy::primitives::keccak256;
pub use k256::ecdsa::SigningKey;
use k256::{
    ecdsa::{RecoveryId, Signature, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};

use crate::{
    error::Error,
    signature::{AddressType, Curve},
};

impl crate::Signer for SigningKey {
    fn from_str(signing_key: &str) -> Result<Self, Error> {
        let array = const_hex::decode_to_array::<_, 32>(signing_key)
            .map_err(|error| Error::ParseSigningKey(Curve::Secp256k1, error))?;

        Self::from_slice(&array)
            .map_err(|error| Error::InitializeSigningKey(Curve::Secp256k1, error.into()))
    }

    fn sign_message(&self, message: &[u8]) -> Result<crate::Signature, Error> {
        let (signature, recovery_id) = self
            .sign_prehash_recoverable(message.as_ref())
            .map_err(|error| Error::SignMessage(Curve::Secp256k1, error.into()))?;

        let mut signature = signature.to_vec();
        signature.push(recovery_id.to_byte());

        Ok(crate::Signature {
            curve: Curve::Secp256k1,
            signature,
        })
    }
}

pub(crate) fn verify_address(
    signature: &[u8],
    address_type: AddressType,
    address: &[u8],
    message: &[u8],
) -> Result<bool, Error> {
    if signature.len() != 65 {
        return Err(Error::SignatureOutOfBound);
    }

    // Safe to use `unwrap()` because we bound-checked the length of the signature.
    let (signature, recovery_id) = (
        Signature::from_slice(signature.get(0..64).unwrap())
            .map_err(|error| Error::ParseSignature(Curve::Secp256k1, error.into()))?,
        RecoveryId::from_byte(*signature.get(64).unwrap()).unwrap(),
    );

    let verifying_key = VerifyingKey::recover_from_prehash(message, &signature, recovery_id)
        .map_err(|error| Error::ParseVerifyingKey(Curve::Secp256k1, error.into()))?;

    match address_type {
        AddressType::Ethereum => {
            let recovered_address =
                &keccak256(&verifying_key.as_affine().to_encoded_point(false).as_bytes()[1..])
                    [12..];

            if recovered_address == address {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}
