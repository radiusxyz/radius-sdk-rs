use k256::{
    ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use rand_core::OsRng;

use crate::{address::Address, chain::*, error::Error, PrivateKeySigner};

pub struct PrivateKey {
    private_key: SigningKey,
    address: Address,
    chain_id: ChainId,
}

impl PrivateKeySigner for PrivateKey {
    fn address(&self) -> &Address {
        &self.address
    }

    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    fn from_str(private_key: &str, chain_id: ChainId) -> Result<Self, Error> {
        let array = const_hex::decode_to_array::<_, 32>(private_key)
            .map_err(|error| Error::ParsePrivateKey(chain_id, error))?;
        let private_key = SigningKey::from_slice(&array)
            .map_err(|error| Error::InitializePrivateKey(chain_id, error.into()))?;
        let address = Address::from_slice(
            private_key
                .verifying_key()
                .as_affine()
                .to_encoded_point(false)
                .as_bytes(),
            chain_id,
        )?;

        Ok(Self {
            private_key,
            address,
            chain_id,
        })
    }

    fn generate_random(chain_id: ChainId) -> Result<Self, Error> {
        let private_key = SigningKey::random(&mut OsRng);
        let address = Address::from_slice(
            private_key
                .verifying_key()
                .as_affine()
                .to_encoded_point(false)
                .as_bytes(),
            chain_id,
        )?;

        Ok(Self {
            private_key,
            address,
            chain_id,
        })
    }

    fn sign_message(&self, message: &[u8]) -> Result<crate::Signature, Error> {
        let message = create_message(message, self.chain_id)?;

        let (signature, recovery_id) = self
            .private_key
            .sign_prehash_recoverable(message.as_slice())
            .map_err(|error| Error::SignMessage(self.chain_id, error.into()))?;

        let recovery_id = create_recovery_id_byte(recovery_id.to_byte(), self.chain_id)?;

        let mut signature_vec = Vec::<u8>::with_capacity(65);
        signature_vec.extend_from_slice(signature.to_bytes().as_ref());
        signature_vec.push(recovery_id);

        Ok(signature_vec.into())
    }
}

pub fn create_message(message: &[u8], chain_id: ChainId) -> Result<Vec<u8>, Error> {
    match chain_id {
        ChainId::Bitcoin => Err(Error::UnsupportedChainId(chain_id)),
        ChainId::Ethereum => Ok(ethereum::eip191_hash_message(message)),
    }
}

pub fn create_recovery_id_byte(recovery_id: u8, chain_id: ChainId) -> Result<u8, Error> {
    match chain_id {
        ChainId::Bitcoin => Err(Error::UnsupportedChainId(chain_id)),
        ChainId::Ethereum => Ok(ethereum::y_parity_byte_non_eip155(recovery_id)),
    }
}

pub fn parse_recovery_id_byte(
    recovery_id_byte: u8,
    chain_id: ChainId,
) -> Result<RecoveryId, Error> {
    match chain_id {
        ChainId::Bitcoin => Err(Error::UnsupportedChainId(chain_id)),
        ChainId::Ethereum => {
            RecoveryId::from_byte(ethereum::recovery_id_from_y_parity_byte(recovery_id_byte))
                .ok_or(Error::ParseRecoveryId(chain_id, recovery_id_byte))
        }
    }
}

pub fn verify(
    signature: &[u8],
    message: &[u8],
    address: &[u8],
    chain_id: ChainId,
) -> Result<(), Error> {
    if signature.len() != 65 {
        return Err(Error::SignatureOutOfBound);
    }

    let message = create_message(message, chain_id)?;

    let parsed_signature = Signature::from_slice(&signature[0..64])
        .map_err(|error| Error::ParseSignature(chain_id, error.into()))?;
    let parsed_recovery_id = parse_recovery_id_byte(*&signature[64], chain_id)?;

    let public_key = VerifyingKey::recover_from_prehash(
        message.as_slice(),
        &parsed_signature,
        parsed_recovery_id,
    )
    .map_err(|error| Error::RecoverVerifyingKey(chain_id, error.into()))?;

    let recovered_address = Address::from_slice(
        public_key.as_affine().to_encoded_point(false).as_bytes(),
        chain_id,
    )?;

    if recovered_address != address {
        return Err(Error::AddressMismatch(chain_id));
    }

    Ok(())
}
