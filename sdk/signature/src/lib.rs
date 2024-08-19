mod address;
mod chain;
pub mod ecdsa;
mod error;
mod signature;

pub use address::Address;
pub use chain::ChainType;
pub use error::Error;
pub use signature::Signature;

pub trait PrivateKeySigner
where
    Self: Sized,
{
    fn address(&self) -> &Address;

    fn chain_type(&self) -> ChainType;

    fn from_str(private_key: &str, chain_type: ChainType) -> Result<Self, Error>;

    fn generate_random(chain_type: ChainType) -> Result<Self, Error>;

    fn sign_message(&self, message: &[u8]) -> Result<Signature, Error>;
}

impl ChainType {
    pub fn create_signer(self, private_key: &str) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainType(ChainType::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::from_str(private_key, self)
            }
        }
    }

    pub fn create_signer_random(self) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainType(ChainType::Bitcoin)),
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
        let signer_address = signer.address();
        println!("alloy address: {:?}", signer_address.as_slice());

        signer_address
    }

    pub fn sequencer_address(signing_key: &str) -> Address {
        let signer = ChainType::Ethereum.create_signer(signing_key).unwrap();
        let signer_address = signer.address().clone();
        println!("sequencer address{:?}", signer_address);

        signer_address
    }

    pub fn verify_signature(signing_key: &str, message: &str) {
        use std::str::FromStr;

        use alloy::signers::{local::LocalSigner, SignerSync};

        let alloy_signer = LocalSigner::from_str(signing_key).unwrap();
        let alloy_signature = alloy_signer.sign_message_sync(message.as_bytes()).unwrap();
        println!(
            "alloy signature (len: {}): {:?}",
            alloy_signature.as_bytes().len(),
            alloy_signature.as_bytes()
        );

        let sequencer_signer = ChainType::Ethereum.create_signer(signing_key).unwrap();
        let sequencer_signature = sequencer_signer.sign_message(message.as_bytes()).unwrap();
        println!("sequencer signature: {:?}", sequencer_signature);

        let parsed_signature = Signature::from(alloy_signature.as_bytes().to_vec());
        parsed_signature
            .verify_signature(
                message.as_bytes(),
                alloy_signer.address().as_slice(),
                ChainType::Ethereum,
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
