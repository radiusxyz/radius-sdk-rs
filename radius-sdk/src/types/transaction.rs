use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EncryptedTransaction {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RawTransaction {}
