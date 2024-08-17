mod address;
mod chain;
pub mod ecdsa;
mod error;
mod signature;

pub use address::Address;
pub use chain::ChainId;
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

impl ChainId {
    pub fn create_signer(self, private_key: &str) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainId(ChainId::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::from_str(private_key, self)
            }
        }
    }

    pub fn create_signer_random(self) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainId(ChainId::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::generate_random(self)
            }
        }
    }
}

#[test]
fn works() {
    pub fn alloy_address(signing_key: &str) -> alloy::primitives::Address {
        use std::str::FromStr;

        use alloy::signers::local::LocalSigner;

        let signer = LocalSigner::from_str(signing_key).unwrap();

        signer.address()
    }

    pub fn sequencer_address(signing_key: &str) -> Address {
        let signer = ChainId::Ethereum.create_signer(signing_key).unwrap();

        signer.address().clone()
    }

    pub fn verify_signature(signing_key: &str, message: &str) {
        use std::str::FromStr;

        use alloy::signers::{local::LocalSigner, SignerSync};

        let signer = LocalSigner::from_str(signing_key).unwrap();
        let signature = signer.sign_message_sync(message.as_bytes()).unwrap();
        println!(
            "alloy signature (len: {}): {:?}",
            signature.as_bytes().len(),
            signature.as_bytes()
        );

        let sequencer_signature = Signature::from(signature.as_bytes().to_vec());
        sequencer_signature
            .verify_signature(
                message.as_bytes(),
                signer.address().as_slice(),
                ChainId::Ethereum,
            )
            .unwrap();
    }

    let signing_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let message = "12345";

    let address_alloy = alloy_address(signing_key);
    let address_sequencer = sequencer_address(signing_key);
    assert!(address_sequencer == address_alloy);

    verify_signature(signing_key, message);
}
