use k256::{
    ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use sha3::{Digest, Keccak256};

use crate::{address::Address, error::Error, signer::PrivateKeySigner, traits::*};

pub const EIP191_PREFIX: &str = "\x19Ethereum Signed Message:\n";

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

pub struct EthereumAddressBuilder;

impl Builder for EthereumAddressBuilder {
    type Output = Address;

    fn from_slice(&self, slice: &[u8]) -> Result<Self::Output, Error> {
        let mut hasher = Keccak256::new();
        hasher.update(&slice[1..]);
        let output = hasher.finalize_reset()[12..].to_vec();

        Ok(output.into())
    }

    fn from_str(&self, str: &str) -> Result<Self::Output, Error> {
        let output = const_hex::decode(str).unwrap();

        Ok(output.into())
    }
}

pub struct EthereumSignerBuilder;

impl Builder for EthereumSignerBuilder {
    type Output = PrivateKeySigner;

    fn from_slice(&self, slice: &[u8]) -> Result<Self::Output, Error> {}

    fn from_str(&self, str: &str) -> Result<Self::Output, Error> {}
}

pub struct EthereumSigner {
    signing_key: SigningKey,
    address: Address,
}

pub struct EthereumVerifier;

impl Verifier for EthereumVerifier {
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

        let parsed_address = EthereumAddressBuilder.from_slice(public_key.as_bytes())?;
        match parsed_address == address {
            true => Ok(()),
            false => Err(EthereumError::AddressMismatch)?,
        }
    }
}

#[derive(Debug)]
pub enum EthereumError {
    InvalidSignatureLength(usize),
    ParseSignature(k256::ecdsa::signature::Error),
    ParseRecoveryId,
    RecoverVerifyingKey(k256::ecdsa::signature::Error),
    AddressMismatch,
}

impl std::fmt::Display for EthereumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EthereumError {}
