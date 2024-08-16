mod address;
pub mod ecdsa;
mod error;
mod signature;

pub use address::Address;
pub use error::Error;
pub use signature::Signature;

pub trait PrivateKeySigner
where
    Self: Sized,
{
    fn address(&self) -> &Address;

    fn chain_id(&self) -> ChainId;

    fn from_str(private_key: &str, chain_id: ChainId) -> Result<Self, Error>;

    fn generate_random(chain_id: ChainId) -> Result<Self, Error>;

    fn sign_message(&self, message: &[u8]) -> Result<Signature, Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainId {
    Bitcoin,
    Ethereum,
}

impl ChainId {
    pub fn create_signer(self, private_key: &str) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainId),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::from_str(private_key, self)
            }
        }
    }

    pub fn create_signer_random(self) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainId),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::generate_random(self)
            }
        }
    }
}

#[test]
fn works() {
    pub fn alloy_signer(signing_key: &str, message: &str) {
        use std::str::FromStr;

        use alloy::signers::{local::LocalSigner, SignerSync};
        use k256::elliptic_curve::sec1::ToEncodedPoint;

        let signer = LocalSigner::from_str(signing_key).unwrap();
        println!("{:?}", signer.address().to_vec());
        let signature = signer.sign_message_sync(message.as_bytes()).unwrap();

        let verifying_key = signature.recover_from_msg(message.as_bytes()).unwrap();
        // println!(
        //     "{:?}",
        //     verifying_key.as_affine().to_encoded_point(false).as_bytes()
        // );

        // let address = signature
        //     .recover_address_from_msg(message.as_bytes())
        //     .unwrap();
        // println!("{:?}", address.as_slice());
    }

    pub fn sequencer_signer(private_key: &str, message: &str) {
        let signer = ChainId::Ethereum.create_signer(private_key).unwrap();
        println!("{:?}", signer.address());
    }

    let signing_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let message = "12345";

    alloy_signer(signing_key, message);
    sequencer_signer(signing_key, message);
}
