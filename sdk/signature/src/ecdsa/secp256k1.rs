use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use rand_core::OsRng;

use crate::{address::Address, error::Error, ChainId, PrivateKeySigner};

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
            .map_err(|error| Error::ParsePrivateKey(error))?;
        let private_key = SigningKey::from_slice(&array)
            .map_err(|error| Error::InitializePrivateKey(error.into()))?;
        let public_key = private_key
            .verifying_key()
            .as_affine()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec();
        let address = Address::from_vec(public_key, chain_id)?;

        Ok(Self {
            private_key,
            address,
            chain_id,
        })
    }

    fn generate_random(chain_id: ChainId) -> Result<Self, Error> {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = private_key
            .verifying_key()
            .as_affine()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec();
        let address = Address::from_vec(public_key, chain_id)?;

        Ok(Self {
            private_key,
            address,
            chain_id,
        })
    }

    fn sign_message(&self, message: &[u8]) -> Result<crate::Signature, Error> {
        let (signature, recovery_id) = self
            .private_key
            .sign_prehash_recoverable(message)
            .map_err(|error| Error::SignMessage(error.into()))?;

        let mut signature = signature.to_vec();
        signature.push(recovery_id.to_byte());

        Ok(signature.into())
    }
}

// pub(crate) fn verify_address(
//     signature: &[u8],
//     address_type: AddressType,
//     address: &[u8],
//     message: &[u8],
// ) -> Result<bool, Error> {
//     if signature.len() != 65 {
//         return Err(Error::SignatureOutOfBound);
//     }

//     // Safe to use `unwrap()` because we bound-checked the length of the
// signature.     let (signature, recovery_id) = (
//         Signature::from_slice(signature.get(0..64).unwrap())
//             .map_err(|error| Error::ParseSignature(Curve::Secp256k1,
// error.into()))?,         RecoveryId::from_byte(*signature.get(64).unwrap()).
// unwrap(),     );

//     let verifying_key = VerifyingKey::recover_from_prehash(message,
// &signature, recovery_id)         .map_err(|error|
// Error::ParseVerifyingKey(Curve::Secp256k1, error.into()))?;

//     match address_type {
//         AddressType::Ethereum => {
//             let recovered_address =
//
// &keccak256(&verifying_key.as_affine().to_encoded_point(false).as_bytes()[1..
// ])                     [12..];

//             if recovered_address == address {
//                 Ok(true)
//             } else {
//                 Ok(false)
//             }
//         }
//     }
// }
