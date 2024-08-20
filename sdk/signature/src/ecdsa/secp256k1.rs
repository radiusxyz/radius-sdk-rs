use k256::{
    ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use rand_core::OsRng;

use crate::{address::Address, chain::*, error::Error, PrivateKeySigner};

pub struct PrivateKey {
    private_key: SigningKey,
    address: Address,
    chain_type: ChainType,
}

impl PrivateKeySigner for PrivateKey {
    fn address(&self) -> &Address {
        &self.address
    }

    fn chain_type(&self) -> ChainType {
        self.chain_type
    }

    fn from_slice(private_key: &[u8], chain_type: ChainType) -> Result<Self, Error> {
        let private_key = SigningKey::from_slice(private_key)
            .map_err(|error| Error::InitializePrivateKey(chain_type, error.into()))?;
        let address = Address::from_slice(
            private_key
                .verifying_key()
                .as_affine()
                .to_encoded_point(false)
                .as_bytes(),
            chain_type,
        )?;

        Ok(Self {
            private_key,
            address,
            chain_type,
        })
    }

    fn from_str(private_key: &str, chain_type: ChainType) -> Result<Self, Error> {
        let array = const_hex::decode_to_array::<_, 32>(private_key)
            .map_err(|error| Error::ParsePrivateKey(chain_type, error))?;
        let private_key = SigningKey::from_slice(&array)
            .map_err(|error| Error::InitializePrivateKey(chain_type, error.into()))?;
        let address = Address::from_slice(
            private_key
                .verifying_key()
                .as_affine()
                .to_encoded_point(false)
                .as_bytes(),
            chain_type,
        )?;

        Ok(Self {
            private_key,
            address,
            chain_type,
        })
    }

    fn generate_random(chain_type: ChainType) -> Result<(Self, Vec<u8>), Error> {
        let private_key = SigningKey::random(&mut OsRng);
        let address = Address::from_slice(
            private_key
                .verifying_key()
                .as_affine()
                .to_encoded_point(false)
                .as_bytes(),
            chain_type,
        )?;

        let private_key_vec = private_key.to_bytes().to_vec();

        let private_key = Self {
            private_key,
            address,
            chain_type,
        };

        Ok((private_key, private_key_vec))
    }

    fn sign_message(&self, message: &[u8]) -> Result<crate::Signature, Error> {
        let message = create_message(message, self.chain_type)?;

        let (signature, recovery_id) = self
            .private_key
            .sign_prehash_recoverable(message.as_slice())
            .map_err(|error| Error::SignMessage(self.chain_type, error.into()))?;

        let recovery_id = create_recovery_id_byte(recovery_id.to_byte(), self.chain_type)?;

        let mut signature_vec = Vec::<u8>::with_capacity(65);
        signature_vec.extend_from_slice(signature.to_bytes().as_ref());
        signature_vec.push(recovery_id);

        Ok(signature_vec.into())
    }
}

pub fn create_message(message: &[u8], chain_type: ChainType) -> Result<Vec<u8>, Error> {
    match chain_type {
        ChainType::Bitcoin => Err(Error::UnsupportedChainType(chain_type)),
        ChainType::Ethereum => Ok(ethereum::eip191_hash_message(message)),
    }
}

pub fn create_recovery_id_byte(recovery_id: u8, chain_type: ChainType) -> Result<u8, Error> {
    match chain_type {
        ChainType::Bitcoin => Err(Error::UnsupportedChainType(chain_type)),
        ChainType::Ethereum => Ok(ethereum::y_parity_byte_non_eip155(recovery_id)),
    }
}

pub fn parse_recovery_id_byte(
    recovery_id_byte: u8,
    chain_type: ChainType,
) -> Result<RecoveryId, Error> {
    match chain_type {
        ChainType::Bitcoin => Err(Error::UnsupportedChainType(chain_type)),
        ChainType::Ethereum => {
            RecoveryId::from_byte(ethereum::recovery_id_from_y_parity_byte(recovery_id_byte))
                .ok_or(Error::ParseRecoveryId(chain_type, recovery_id_byte))
        }
    }
}

pub fn verify(
    signature: &[u8],
    message: &[u8],
    address: &[u8],
    chain_type: ChainType,
) -> Result<(), Error> {
    if signature.len() != 65 {
        return Err(Error::SignatureOutOfBound);
    }

    let message = create_message(message, chain_type)?;

    let parsed_signature = Signature::from_slice(&signature[0..64])
        .map_err(|error| Error::ParseSignature(chain_type, error.into()))?;
    let parsed_recovery_id = parse_recovery_id_byte(signature[64], chain_type)?;

    let public_key = VerifyingKey::recover_from_prehash(
        message.as_slice(),
        &parsed_signature,
        parsed_recovery_id,
    )
    .map_err(|error| Error::RecoverVerifyingKey(chain_type, error.into()))?;

    let recovered_address = Address::from_slice(
        public_key.as_affine().to_encoded_point(false).as_bytes(),
        chain_type,
    )?;

    if recovered_address != address {
        return Err(Error::AddressMismatch(chain_type));
    }

    Ok(())
}
