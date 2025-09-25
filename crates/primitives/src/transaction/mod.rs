mod encrypted_transaction;
mod raw_transaction;
mod error;

pub use encrypted_transaction::*;
pub use raw_transaction::*;
pub use error::*;

pub mod prelude {
    pub use serde::{Deserialize, Serialize};
    pub use ethers_core::{
        types::{self as eth_types, Bytes, U256},
        utils::rlp::{self, Decodable, DecoderError},
    };
}
pub use prelude::*;

pub fn decode_rlp_transaction(rlp_bytes: &[u8]) -> HandleTxResult<eth_types::Transaction> {
    // let hex_str = rlp_hex.trim_start_matches("0x");
    // let rlp_bytes =
    //     const_hex::decode(hex_str).map_err(|_| DecoderError::Custom("hex decode error"))?;
    let rlp = rlp::Rlp::new(rlp_bytes);
    eth_types::Transaction::decode(&rlp).map_err(|e | From::from(e))
}

#[derive(Debug)]
/// Reference for rlp encoding including RLP Header
pub struct RlpFieldOffset {
    pub start: usize,
    pub end: usize,
}

pub fn extract_rlp_offset(tx: &[u8], total_len: usize) -> HandleTxResult<Vec<RlpFieldOffset>> {
    let mut offsets = Vec::new();
    let mut pos = 0;

    while pos < tx.len() && pos < total_len {
        let start = pos;
        let mut field_cursor = &tx[pos..];
        let header = alloy::rlp::Header::decode(&mut field_cursor)?;
        let header_len = tx[pos..].len() - field_cursor.len();
        let field_total_len = header_len + header.payload_length;
        let end = pos + field_total_len;
        if end > tx.len() {
            return Err(HandleTxError::RlpError(alloy::rlp::Error::Overflow));            
        }
        pos += field_total_len;
        offsets.push(RlpFieldOffset { start, end });
    }
    
    Ok(offsets)
}

pub fn parse_private_fields(private_data: &[u8], field_count: usize) -> HandleTxResult<Vec<Vec<u8>>> {
    let mut cursor = private_data;
    let mut fields = Vec::new();
    for _ in 0..field_count {
        if cursor.is_empty() {
            fields.push(Vec::new());
            continue;
        }
        let pos = cursor.len();
        let header = alloy::rlp::Header::decode(&mut cursor)?;
        let header_len = pos - cursor.len();
        let total_len = header_len + header.payload_length;

        let start = private_data.len() - pos;
        let end = start + total_len;
        let field = private_data[start..end].to_vec();
        fields.push(field);
        cursor = &cursor[total_len..];
    }
    Ok(fields)
}
