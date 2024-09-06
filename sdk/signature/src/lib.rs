mod address;
mod chain;
pub mod ecdsa;
mod error;
mod signature;
mod util;

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

    fn from_slice(private_key: &[u8], chain_type: ChainType) -> Result<Self, Error>;

    fn from_str(private_key: &str, chain_type: ChainType) -> Result<Self, Error>;

    fn generate_random(chain_type: ChainType) -> Result<(Self, Vec<u8>), Error>;

    fn sign_message(&self, message: &[u8]) -> Result<Signature, Error>;
}

impl ChainType {
    pub fn create_signer_from_slice(
        self,
        private_key: &[u8],
    ) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainType(ChainType::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::from_slice(private_key, self)
            }
        }
    }

    pub fn create_signer_from_str(self, private_key: &str) -> Result<impl PrivateKeySigner, Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainType(ChainType::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::from_str(private_key, self)
            }
        }
    }

    pub fn create_signer_random(self) -> Result<(impl PrivateKeySigner, Vec<u8>), Error> {
        match self {
            Self::Bitcoin => Err(Error::UnsupportedChainType(ChainType::Bitcoin)),
            Self::Ethereum => {
                <ecdsa::secp256k1::PrivateKey as PrivateKeySigner>::generate_random(self)
            }
        }
    }
}

#[test]
fn test_address_comparison() {
    pub fn alloy_address(signing_key: &str) -> alloy::primitives::Address {
        use std::str::FromStr;

        use alloy::signers::local::LocalSigner;

        let signer = LocalSigner::from_str(signing_key).unwrap();
        let signer_address = signer.address();
        println!("alloy address: {:?}", signer_address);

        signer_address
    }

    pub fn sequencer_address(signing_key: &str) -> Address {
        let signer = ChainType::Ethereum
            .create_signer_from_str(signing_key)
            .unwrap();
        let signer_address = signer.address().clone();
        println!("sequencer address: {:?}", signer_address);

        signer_address
    }

    let signing_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    let address_alloy = alloy_address(signing_key);
    let address_sequencer = sequencer_address(signing_key);
    assert!(address_sequencer == address_alloy);
}

#[test]
fn test_signature_verification() {
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

        let sequencer_signer = ChainType::Ethereum
            .create_signer_from_str(signing_key)
            .unwrap();
        let sequencer_signature = sequencer_signer.sign_message(message.as_bytes()).unwrap();
        println!(
            "sequencer signature (len: {}): {:?}",
            sequencer_signature.len(),
            sequencer_signature
        );

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

    verify_signature(signing_key, message);
}

#[test]
fn test_random_generation() {
    use alloy::signers::local::LocalSigner;

    let (signer_random, private_key_vec) = ChainType::Ethereum.create_signer_random().unwrap();
    println!("Generated private key vec: {:?}", private_key_vec);

    let alloy_signer = LocalSigner::from_slice(&private_key_vec).unwrap();
    let sequencer_signer = ChainType::Ethereum
        .create_signer_from_slice(&private_key_vec)
        .unwrap();

    let address_signer_random = signer_random.address();
    let address_sequencer = sequencer_signer.address();
    let address_alloy = alloy_signer.address();
    assert!(address_signer_random == address_sequencer);
    assert!(address_signer_random == address_alloy.as_slice());
}

#[test]
fn test_direct_comparison() {
    let (sequencer_signer_1, _private_key_vec) =
        ChainType::Ethereum.create_signer_random().unwrap();
    let (sequencer_signer_2, _private_key_vec) =
        ChainType::Ethereum.create_signer_random().unwrap();

    let address_sequencer_1 = sequencer_signer_1.address();
    let address_sequencer_2 = sequencer_signer_2.address();

    assert!(address_sequencer_1 == address_sequencer_1);
    assert!(address_sequencer_1 != address_sequencer_2);
}
