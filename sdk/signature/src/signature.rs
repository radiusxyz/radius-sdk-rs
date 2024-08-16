use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature(Vec<u8>);

impl From<Vec<u8>> for Signature {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Signature {
    pub fn verify_signature(&self) -> Result<(), Error> {
        Ok(())
    }
}
