mod bundle_order_commitment;
mod order_commitment_type;
mod single_order_commitment;

pub use bundle_order_commitment::*;
pub use order_commitment_type::*;
use serde::{Deserialize, Serialize};
pub use single_order_commitment::*;


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum OrderCommitment {
    Single(SingleOrderCommitment),
    Bundle(BundleOrderCommitment),
}

impl Default for OrderCommitment {
    fn default() -> Self {
        Self::Single(SingleOrderCommitment::default())
    }
}
