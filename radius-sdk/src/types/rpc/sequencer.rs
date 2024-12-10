use serde::{Deserialize, Serialize};
use signature::{Address, Signature};

use crate::types::{EncryptedTransaction, RawTransaction};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlock {
    pub message: FinalizeBlockMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FinalizeBlockMessage {
    pub executor_address: Address,
    pub block_creator_address: Address,
    pub next_block_creator_address: Address,
    pub rollup_id: String,
    pub platform_block_number: u64,
    pub rollup_block_number: u64,
}

impl FinalizeBlock {
    pub const METHOD_NAME: &'static str = "finalize_block";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlock {
    pub rollup_id: String,
    pub rollup_block_number: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlockResponse {
    pub block_number: u64,
    pub encrypted_transaction_list: Vec<Option<EncryptedTransaction>>,
    pub raw_transaction_list: Vec<RawTransaction>,
    pub block_creator_address: String,
    pub signature: String,
    pub block_commitment: String,
}

impl GetBlock {
    pub const METHOD_NAME: &'static str = "get_block";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionWithOrderCommitment {
    pub rollup_id: String,
    pub rollup_block_number: u64,
    pub transaction_order: u64,
}

impl GetEncryptedTransactionWithOrderCommitment {
    pub const METHOD_NAME: &'static str = "get_encrypted_transaction_with_order_commitment";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetEncryptedTransactionWithTransactionHash {
    pub rollup_id: String,
    pub transaction_hash: String,
}

impl GetEncryptedTransactionWithTransactionHash {
    pub const METHOD_NAME: &'static str = "get_encrypted_transaction_with_transaction_hash";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionList {
    pub rollup_id: String,
    pub rollup_block_number: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionListResponse {
    pub raw_transaction_list: Vec<String>,
}

impl GetRawTransactionList {
    pub const METHOD_NAME: &'static str = "get_raw_transaction_list";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithOrderCommitment {
    pub rollup_id: String,
    pub rollup_block_number: u64,
    pub transaction_order: u64,
}

impl GetRawTransactionWithOrderCommitment {
    pub const METHOD_NAME: &'static str = "get_raw_transaction_with_order_commitment";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetRawTransactionWithTransactionHash {
    pub rollup_id: String,
    pub transaction_hash: String,
}

impl GetRawTransactionWithTransactionHash {
    pub const METHOD_NAME: &'static str = "get_raw_transaction_with_transaction_hash";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendEncryptedTransaction {
    pub rollup_id: String,
    pub encrypted_transaction: EncryptedTransaction,
}

impl SendEncryptedTransaction {
    pub const METHOD_NAME: &'static str = "send_encrypted_transaction";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendRawTransaction {
    pub rollup_id: String,
    pub raw_transaction: RawTransaction,
}

impl SendRawTransaction {
    pub const METHOD_NAME: &'static str = "send_raw_transaction";
}
