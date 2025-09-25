use ethers_core::types as eth_types;

use super::{prelude::*, EncryptedData, RollupTransaction};
use crate::transaction::{RawTransactionHash, HandleTxResult, HandleTxError};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthBundleTransactionData {
    pub encrypted_data: EncryptedData,
    pub open_data: EthBundleOpenData,

    pub plain_data: Option<EthBundlePlainData>,
}

impl EthBundleTransactionData {
    pub fn convert_to_rollup_transaction(&self) -> HandleTxResult<RollupTransaction> {
        if self.plain_data.is_none() {
            return Err(HandleTxError::PlainDataDoesNotExist);
        }

        // TODO:
        // let rollup_transaction = self
        // .open_data
        // .convert_to_rollup_transaction(self.plain_data.as_ref().unwrap());

        Ok(RollupTransaction::EthBundle)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EthBundleOpenData {
    pub raw_tx_hash: RawTransactionHash,
}

impl EthBundleOpenData {
    pub fn raw_tx_hash(&self) -> &RawTransactionHash {
        &self.raw_tx_hash
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EthBundlePlainData {
    pub to: Option<eth_types::Address>,
    pub value: eth_types::U256,

    #[serde(rename = "data")]
    pub input: eth_types::Bytes,
}
