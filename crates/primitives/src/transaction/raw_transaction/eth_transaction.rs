use ethers_core::types as eth_types;

use super::{prelude::*, RawTransactionHash};
use crate::transaction::{HandleTxResult, decode_rlp_transaction};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthRawTransaction(pub String);

impl Default for EthRawTransaction {
    fn default() -> Self {
        Self("".to_string())
    }
}

impl From<String> for EthRawTransaction {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl EthRawTransaction {
    pub fn raw_transaction_hash(&self) -> RawTransactionHash {
        let decoded_transaction = decode_rlp_transaction(&self.0.as_bytes()).unwrap();

        let transaction_hash = const_hex::encode_prefixed(decoded_transaction.hash);

        RawTransactionHash::from(transaction_hash)
    }

    pub fn rollup_transaction(&self) -> HandleTxResult<eth_types::Transaction> {
        decode_rlp_transaction(&self.0.as_bytes()).map_err(|e| From::from(e))
    }
}
