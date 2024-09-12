use k256::{
    ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use rand_core::OsRng;
use sha3::{Digest, Keccak256};

pub const EIP191_PREFIX: &str = "\x19Ethereum Signed Message:\n";

fn eip191_hash_message(message: &[u8]) -> Vec<u8> {
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

fn y_parity_byte_non_eip155_from_recovery_id(recovery_id: RecoveryId) -> Option<u8> {
    recovery_id.to_byte().checked_add(27)
}

fn recovery_id_from_y_parity_byte(parity_byte: u8) -> Option<RecoveryId> {
    match parity_byte.checked_sub(27) {
        Some(byte) => RecoveryId::from_byte(byte),
        None => None,
    }
}

pub struct EthereumAddressBuilder;

impl crate::Builder for EthereumAddressBuilder {
    type Output = crate::Address;

    fn build_from_slice(&self, slice: &[u8]) -> Result<Self::Output, crate::SignatureError> {
        let mut hasher = Keccak256::new();
        hasher.update(&slice[1..]);
        let output = hasher.finalize_reset()[12..].to_vec();

        Ok(output.into())
    }

    fn build_from_str(&self, str: &str) -> Result<Self::Output, crate::SignatureError> {
        let output = const_hex::decode(str).unwrap();

        Ok(output.into())
    }
}

pub struct EthereumSignerBuilder;

impl crate::Builder for EthereumSignerBuilder {
    type Output = crate::PrivateKeySigner;

    fn build_from_slice(&self, slice: &[u8]) -> Result<Self::Output, crate::SignatureError> {
        Ok(EthereumSigner::from_slice(slice)?.into())
    }

    fn build_from_str(&self, str: &str) -> Result<Self::Output, crate::SignatureError> {
        let signing_key =
            const_hex::decode_to_array::<_, 32>(str).map_err(EthereumError::ParseSigningKeyStr)?;

        Ok(EthereumSigner::from_slice(&signing_key)?.into())
    }
}

impl crate::RandomBuilder for EthereumSignerBuilder {
    type Output = (crate::PrivateKeySigner, String);

    fn build_from_random(&self) -> Result<Self::Output, crate::SignatureError> {
        let (signer, private_key_random) = EthereumSigner::from_random()?;

        Ok((signer.into(), private_key_random))
    }
}

pub struct EthereumSigner {
    signing_key: SigningKey,
    address: crate::Address,
}

impl crate::Signer for EthereumSigner {
    fn address(&self) -> &crate::Address {
        &self.address
    }

    fn sign_message(&self, message: &[u8]) -> Result<crate::Signature, crate::SignatureError> {
        let message = eip191_hash_message(message);

        let (signature, recovery_id) = self
            .signing_key
            .sign_prehash_recoverable(&message)
            .map_err(EthereumError::SignMessage)?;
        let recovery_id = y_parity_byte_non_eip155_from_recovery_id(recovery_id)
            .ok_or(EthereumError::ParityByte(recovery_id.to_byte()))?;

        let mut signature_vec = Vec::<u8>::with_capacity(65);
        signature_vec.extend_from_slice(signature.to_bytes().as_ref());
        signature_vec.push(recovery_id);

        Ok(signature_vec.into())
    }
}

impl EthereumSigner {
    pub fn from_slice(signing_key_slice: &[u8]) -> Result<Self, crate::SignatureError> {
        let signing_key =
            SigningKey::from_slice(signing_key_slice).map_err(EthereumError::ParseSigningKey)?;
        let public_key = signing_key
            .verifying_key()
            .as_affine()
            .to_encoded_point(false);
        let address = <EthereumAddressBuilder as crate::Builder>::build_from_slice(
            &EthereumAddressBuilder,
            public_key.as_bytes(),
        )?;

        Ok(Self {
            signing_key,
            address,
        })
    }

    pub fn from_random() -> Result<(Self, String), crate::SignatureError> {
        let signing_key = SigningKey::random(&mut OsRng);
        let signing_key_hex_string = const_hex::encode_prefixed(signing_key.to_bytes());
        let public_key = signing_key
            .verifying_key()
            .as_affine()
            .to_encoded_point(false);
        let address = <EthereumAddressBuilder as crate::Builder>::build_from_slice(
            &EthereumAddressBuilder,
            public_key.as_bytes(),
        )?;

        let signer = Self {
            signing_key,
            address,
        };

        Ok((signer, signing_key_hex_string))
    }
}

pub struct EthereumVerifier;

impl crate::Verifier for EthereumVerifier {
    fn verify_message(
        &self,
        signature: &[u8],
        message: &[u8],
        address: &[u8],
    ) -> Result<(), crate::SignatureError> {
        if signature.len() != 65 {
            return Err(EthereumError::InvalidSignatureLength(signature.len()))?;
        }

        let message = eip191_hash_message(message);

        let parsed_signature =
            Signature::from_slice(&signature[0..64]).map_err(EthereumError::ParseSignature)?;
        let parsed_recovery_id = recovery_id_from_y_parity_byte(signature[64])
            .ok_or(EthereumError::ParseRecoveryId(signature[64]))?;

        let public_key =
            VerifyingKey::recover_from_prehash(&message, &parsed_signature, parsed_recovery_id)
                .map_err(EthereumError::RecoverVerifyingKey)?
                .as_affine()
                .to_encoded_point(false);

        let parsed_address = <EthereumAddressBuilder as crate::Builder>::build_from_slice(
            &EthereumAddressBuilder,
            public_key.as_bytes(),
        )?;
        match parsed_address == address {
            true => Ok(()),
            false => Err(EthereumError::AddressMismatch)?,
        }
    }
}

#[derive(Debug)]
pub enum EthereumError {
    ParseSigningKey(k256::ecdsa::signature::Error),
    ParseSigningKeyStr(const_hex::FromHexError),
    SignMessage(k256::ecdsa::signature::Error),
    ParityByte(u8),
    InvalidSignatureLength(usize),
    ParseSignature(k256::ecdsa::signature::Error),
    ParseRecoveryId(u8),
    RecoverVerifyingKey(k256::ecdsa::signature::Error),
    AddressMismatch,
}

impl std::fmt::Display for EthereumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EthereumError {}
