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

    pub fn get_address(signing_key: &str) -> Address {
        let signer = PrivateKeySigner::from_str(ChainType::Ethereum, signing_key).unwrap();
        let signer_address = signer.address().clone();
        println!("Tx_orderer address: {:?}", signer_address.as_hex_string());

        signer_address
    }

    let signing_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    let alloy_address = get_alloy_address(signing_key);
    let tx_orderer_address = get_address(signing_key);

    assert!(tx_orderer_address == alloy_address);

    let parsed_address =
        Address::from_str(ChainType::Ethereum, &alloy_address.to_string()).unwrap();
    println!("Parsed address: {:?}", parsed_address.as_hex_string());

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
        let signer = PrivateKeySigner::from_str(ChainType::Ethereum, signing_key).unwrap();
        let tx_orderer_signature = signer.sign_message(message).unwrap();
        println!(
            "Tx_orderer signature (len: {}): {:?}",
            tx_orderer_signature.len(),
            tx_orderer_signature
        );

        assert!(alloy_signature.as_bytes() == tx_orderer_signature.as_bytes());

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

    let (signer, private_key_string) = PrivateKeySigner::from_random(ChainType::Ethereum).unwrap();
    let tx_orderer_address = signer.address();
    println!(
        "Tx_orderer address: {:?}",
        tx_orderer_address.as_hex_string()
    );

    let alloy_signer = LocalSigner::from_str(&private_key_string).unwrap();
    let alloy_address = alloy_signer.address();
    println!("Alloy address: {:?}", alloy_address);

    assert!(*tx_orderer_address == alloy_address);
}

#[test]
fn test_polymorphic_type_conversion() {
    use std::str::FromStr;

    use alloy::signers::local::LocalSigner;

    let (signer, private_key_string) = PrivateKeySigner::from_random(ChainType::Ethereum).unwrap();
    let tx_orderer_address = signer.address();
    println!(
        "Tx_orderer address: {:?}",
        tx_orderer_address.as_hex_string()
    );

    let alloy_signer = LocalSigner::from_str(&private_key_string).unwrap();
    let alloy_address = alloy_signer.address();
    println!("Alloy address: {:?}", alloy_address);

    assert!(*tx_orderer_address == alloy_address);

    let address_string = serde_json::to_string(&tx_orderer_address.as_hex_string()).unwrap();
    let address_from_string: Address = serde_json::from_str(&address_string).unwrap();
    println!("{:?}", address_from_string);

    let address_array = serde_json::to_string(&tx_orderer_address).unwrap();
    let address_from_array: Address = serde_json::from_str(&address_array).unwrap();
    println!("{:?}", address_from_array);

    assert!(address_from_string == address_from_array);
}

#[test]
fn test_hex_conversion() {
    let (signer, _) = PrivateKeySigner::from_random(ChainType::Ethereum).unwrap();

    let address = signer.address().clone();
    let address_hex = address.as_hex_string();
    let address_json = serde_json::to_string(&address_hex).unwrap();
    let parsed_address: Address = serde_json::from_str(&address_json).unwrap();
    assert!(address == parsed_address);

    let signature = signer.sign_message("message").unwrap();
    let signature_hex = signature.as_hex_string();
    let signature_json = serde_json::to_string(&signature_hex).unwrap();
    let parsed_signature: Signature = serde_json::from_str(&signature_json).unwrap();
    assert!(signature == parsed_signature);
}
