mod address;
mod error;
mod platform;
mod signature;
mod signer;
mod traits;

pub use address::Address;
pub use error::Error;
pub use platform::Platform;
pub use signature::Signature;
pub use signer::PrivateKeySigner;
pub use traits::*;

#[test]
fn test_address_comparison() {
    pub fn alloy_address(signing_key: &str) -> alloy::primitives::Address {
        use std::str::FromStr;

        use alloy::signers::local::LocalSigner;

        let signer = LocalSigner::from_str(signing_key).unwrap();
        let signer_address = signer.address();
        println!("Alloy address: {:?}", signer_address);

        signer_address
    }

    pub fn sequencer_address(signing_key: &str) -> Address {
        let signer = PrivateKeySigner::from_str(Platform::Ethereum, signing_key).unwrap();
        let signer_address = signer.address().clone();
        println!("Sequencer address: {}", signer_address);

        signer_address
    }

    let signing_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    let address_alloy = alloy_address(signing_key);
    let address_sequencer = sequencer_address(signing_key);
    assert!(address_sequencer == address_alloy);
}

#[test]
fn test_signature_verification() {
    pub fn verify_signature<T: serde::Serialize>(signing_key: &str, message: &T) {
        use std::str::FromStr;

        use alloy::signers::{local::LocalSigner, SignerSync};

        let alloy_signer = LocalSigner::from_str(signing_key).unwrap();
        let alloy_address = alloy_signer.address();
        let message_serialized = bincode::serialize(message).unwrap();
        let alloy_signature = alloy_signer.sign_message_sync(&message_serialized).unwrap();
        println!(
            "Alloy signature (len: {}): {:?}",
            alloy_signature.as_bytes().len(),
            alloy_signature.as_bytes()
        );

        let sequencer_signer = PrivateKeySigner::from_str(Platform::Ethereum, signing_key).unwrap();
        let sequencer_signature = sequencer_signer.sign_message(message).unwrap();
        println!(
            "Sequencer signature (len: {}): {:?}",
            sequencer_signature.len(),
            sequencer_signature
        );

        assert!(alloy_signature.as_bytes() == sequencer_signature.as_bytes());

        let parsed_signature = Signature::from(alloy_signature.as_bytes().to_vec());
        println!(
            "Parsed signature (len: {}): {:?}",
            parsed_signature.len(),
            parsed_signature.as_bytes(),
        );
        parsed_signature
            .verify_message(Platform::Ethereum, message, alloy_address)
            .unwrap();
    }

    #[derive(Default, serde::Serialize)]
    struct User {
        name: String,
        age: u8,
    }

    let user = User::default();
    let signing_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    verify_signature(signing_key, &user);
}
