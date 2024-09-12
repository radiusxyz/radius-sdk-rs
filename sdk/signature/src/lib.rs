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
        println!("alloy address: {:?}", signer_address);

        signer_address
    }

    pub fn sequencer_address(signing_key: &str) -> Address {
        let signer = PrivateKeySigner::from_str(Platform::Ethereum, signing_key).unwrap();
        let signer_address = signer.address().clone();
        println!("sequencer address: {:?}", signer_address);

        signer_address
    }

    let signing_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    let address_alloy = alloy_address(signing_key);
    let address_sequencer = sequencer_address(signing_key);
    assert!(address_sequencer == address_alloy);
}
