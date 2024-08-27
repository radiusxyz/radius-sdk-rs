pub mod ethereum;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ChainType {
    Bitcoin,
    Ethereum,
}
