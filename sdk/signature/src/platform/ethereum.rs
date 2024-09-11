use k256::{
    ecdsa::{RecoveryId, Signature, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use sha3::{Digest, Keccak256};

use crate::{address::AddressTrait, error::Error, signature::SignatureTrait, signer::SignerTrait};

pub const EIP191_PREFIX: &str = "\x19Ethereum Signed Message:\n";

pub(crate) struct EthereumAddress;

impl AddressTrait for EthereumAddress {
    fn from_slice(&self, slice: &[u8]) -> Result<crate::Address, Error> {
        let mut hasher = Keccak256::new();
        hasher.update(&slice[1..]);
        let output = hasher.finalize_reset()[12..].to_vec();

        Ok(output.into())
    }

    fn from_str(&self, str: &str) -> Result<crate::Address, Error> {
        let output = const_hex::decode(str).unwrap();
        Ok(output.into())
    }
}

pub(crate) struct EthereumSignature;

impl SignatureTrait for EthereumSignature {
    fn verify_message(
        &self,
        signature: &[u8],
        message: &[u8],
        address: &[u8],
    ) -> Result<(), Error> {
        if signature.len() != 65 {
            return Err(EthereumError::InvalidSignatureLength(signature.len()))?;
        }

        let message = eip191_hash_message(message);

        let parsed_signature =
            Signature::from_slice(&signature[0..64]).map_err(EthereumError::ParseSignature)?;

        let parsed_recovery_id =
            RecoveryId::from_byte(signature[64] - 27).ok_or(EthereumError::ParseRecoveryId)?;

        let public_key =
            VerifyingKey::recover_from_prehash(&message, &parsed_signature, parsed_recovery_id)
                .map_err(EthereumError::RecoverVerifyingKey)?
                .as_affine()
                .to_encoded_point(false);

        let parsed_address = EthereumAddress.from_slice(public_key.as_bytes())?;
        match parsed_address == address {
            true => Ok(()),
            false => Err(EthereumError::AddressMismatch)?,
        }
    }
}

pub(crate) struct EthereumSigner;

// impl SignerTrait for EthereumSigner {
//     fn from_slice(slice: &[u8]) -> Result<Self, Error>
//     where
//         Self: Sized,
//     {
//     }

//     fn from_str(str: &str) -> Result<Self, Error>
//     where
//         Self: Sized,
//     {
//     }
// }

pub fn eip191_hash_message(message: &[u8]) -> Vec<u8> {
    let len = message.len();
    let mut len_string_buffer = itoa::Buffer::new();
    let len_string = len_string_buffer.format(len);

    let mut ethereum_message = Vec::with_capacity(EIP191_PREFIX.len() + len_string.len() + len);
    ethereum_message.extend_from_slice(EIP191_PREFIX.as_bytes());
    ethereum_message.extend_from_slice(len_string.as_bytes());
    ethereum_message.extend_from_slice(message);

    let mut hasher = Keccak256::new();
    hasher.update(ethereum_message);
    let output = hasher.finalize_reset();

    output.to_vec()
}

#[derive(Debug)]
pub enum EthereumError {
    InvalidSignatureLength(usize),
    ParseSignature(k256::ecdsa::signature::Error),
    ParseRecoveryId,
    RecoverVerifyingKey(k256::ecdsa::signature::Error),
    AddressMismatch,
}
