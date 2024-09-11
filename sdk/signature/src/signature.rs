use serde::{Deserialize, Serialize};

use crate::{error::Error, platform::*};

pub(crate) trait SignatureTrait {
    fn verify_message(&self, signature: &[u8], message: &[u8], address: &[u8])
        -> Result<(), Error>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature(Vec<u8>);

impl From<&[u8]> for Signature {
    fn from(value: &[u8]) -> Self {
        Self(value.to_owned())
    }
}

impl From<Vec<u8>> for Signature {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Signature {
    pub fn verify_message<T: Serialize>(
        &self,
        platform: Platform,
        message: &T,
        address: impl AsRef<[u8]>,
    ) -> Result<(), Error> {
        let message_bytes = bincode::serialize(message).map_err(Error::SerializeMessage)?;

        platform
            .signature()
            .verify_message(&self.0, &message_bytes, address.as_ref())
    }
}
