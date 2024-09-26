mod address;
mod chain_type;
mod error;
mod signature;
mod signer;
mod traits;

pub use address::Address;
pub use chain_type::ChainType;
pub use error::SignatureError;
pub use signature::Signature;
pub use signer::PrivateKeySigner;
pub use traits::*;

#[test]
fn test_address_comparison() {
    pub fn get_alloy_address(signing_key: &str) -> alloy::primitives::Address {
        use std::str::FromStr;

        use alloy::signers::local::LocalSigner;

        let signer = LocalSigner::from_str(signing_key).unwrap();
        let signer_address = signer.address();
        println!("Alloy address: {:?}", signer_address);

        signer_address
    }

    pub fn get_sequencer_address(signing_key: &str) -> Address {
        let signer = PrivateKeySigner::from_str(ChainType::Ethereum, signing_key).unwrap();
        let signer_address = signer.address().clone();
        println!("Sequencer address: {}", signer_address);

        signer_address
    }

    let signing_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    let alloy_address = get_alloy_address(signing_key);
    let sequencer_address = get_sequencer_address(signing_key);

    assert!(sequencer_address == alloy_address);

    let parsed_address =
        Address::from_str(ChainType::Ethereum, &alloy_address.to_string()).unwrap();
    println!("Parsed address: {}", parsed_address);

    assert!(parsed_address == alloy_address);
}

#[test]
fn test_signature_verification() {
    pub fn verify_signature<T: serde::Serialize>(signing_key: &str, message: &T) {
        use std::str::FromStr;

        use alloy::signers::{local::LocalSigner, SignerSync};

        // Alloy
        let alloy_signer = LocalSigner::from_str(signing_key).unwrap();
        let alloy_address = alloy_signer.address();
        let message_serialized = bincode::serialize(message).unwrap();
        let alloy_signature = alloy_signer.sign_message_sync(&message_serialized).unwrap();
        println!(
            "Alloy signature (len: {}): {:?}",
            alloy_signature.as_bytes().len(),
            alloy_signature.as_bytes()
        );

        // SDK
        let sequencer_signer =
            PrivateKeySigner::from_str(ChainType::Ethereum, signing_key).unwrap();
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
            .verify_message(ChainType::Ethereum, message, alloy_address)
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

#[test]
fn test_random() {
    use std::str::FromStr;

    use alloy::signers::local::LocalSigner;

    let (sequencer_signer, private_key_string) =
        PrivateKeySigner::from_random(ChainType::Ethereum).unwrap();
    let sequencer_address = sequencer_signer.address();
    println!("Sequencer address: {}", sequencer_address);

    let alloy_signer = LocalSigner::from_str(&private_key_string).unwrap();
    let alloy_address = alloy_signer.address();
    println!("Alloy address: {:?}", alloy_address);

    assert!(*sequencer_address == alloy_address);
}

#[test]
fn test_polymorphic_type_conversion() {
    use std::str::FromStr;

    use alloy::signers::local::LocalSigner;

    let (sequencer_signer, private_key_string) =
        PrivateKeySigner::from_random(ChainType::Ethereum).unwrap();
    let sequencer_address = sequencer_signer.address();
    println!("Sequencer address: {}", sequencer_address);

    let alloy_signer = LocalSigner::from_str(&private_key_string).unwrap();
    let alloy_address = alloy_signer.address();
    println!("Alloy address: {:?}", alloy_address);

    assert!(*sequencer_address == alloy_address);

    let address_string = serde_json::to_string(&sequencer_address.to_string()).unwrap();
    let address_from_string: Address = serde_json::from_str(&address_string).unwrap();
    println!("{:?}", address_from_string);

    let address_array = serde_json::to_string(&sequencer_address).unwrap();
    let address_from_array: Address = serde_json::from_str(&address_array).unwrap();
    println!("{:?}", address_from_array);

    assert!(address_from_string == address_from_array);
}

#[test]
fn test_hex_conversion() {
    let (sequencer_signer, _) = PrivateKeySigner::from_random(ChainType::Ethereum).unwrap();

    let address = sequencer_signer.address().clone();
    let address_hex = address.as_hex_string();
    let address_json = serde_json::to_string(&address_hex).unwrap();
    let parsed_address: Address = serde_json::from_str(&address_json).unwrap();
    assert!(address == parsed_address);

    let signature = sequencer_signer.sign_message("message").unwrap();
    let signature_hex = signature.as_hex_string();
    let signature_json = serde_json::to_string(&signature_hex).unwrap();
    let parsed_signature: Signature = serde_json::from_str(&signature_json).unwrap();
    assert!(signature == parsed_signature);
}
