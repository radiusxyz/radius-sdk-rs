mod sign_order_commitment;
mod transaction_hash_order_commitment;

use serde::{Deserialize, Serialize};
pub use sign_order_commitment::*;
pub use transaction_hash_order_commitment::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderCommitmentType {
    TransactionHash,
    Sign,
}