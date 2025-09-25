use ethers_core::types as eth_types;
use serde::{Deserialize, Serialize};

use super::RawTransactionHash;
use crate::transaction::{HandleTxResult, decode_rlp_transaction};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthRawBundleTransaction(pub String);

impl From<String> for EthRawBundleTransaction {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl EthRawBundleTransaction {
    pub fn raw_transaction_hash(&self) -> RawTransactionHash {
        let raw_transaction_string = serde_json::to_string(&self.0).unwrap();
        let parsed_raw_transaction_string: String =
            serde_json::from_str(&raw_transaction_string).unwrap();
        let decoded_transaction = decode_rlp_transaction(raw_transaction_string.as_bytes()).unwrap();

        RawTransactionHash::new(const_hex::encode_prefixed(
            decoded_transaction.hash.as_bytes(),
        ))
    }

    pub fn rollup_transaction(&self) -> HandleTxResult<eth_types::Transaction> {
        let raw_transaction_string = serde_json::to_string(&self.0).unwrap();
        let parsed_raw_transaction_string: String =
            serde_json::from_str(&raw_transaction_string).unwrap();
        decode_rlp_transaction(raw_transaction_string.as_bytes())
            .map_err(|e| From::from(e))
    }
}
