use signature::Signature;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignOrderCommitment {
    pub data: OrderCommitmentData,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitmentData {
    pub rollup_id: String,
    pub batch_number: u64,
    pub transaction_order: u64,
    pub transaction_hash: String,
    pub pre_merkle_path: Vec<[u8; 32]>,
}

