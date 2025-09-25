use ethers_core::utils::rlp::DecoderError;

#[derive(Debug, thiserror::Error)]
pub enum HandleTxError {
    #[error("Plain data does not exist")]
    PlainDataDoesNotExist,

    #[error("Failed to serialize ETH raw transaction: {0}")]
    SerializeEthRawTransaction(#[from] serde_json::Error),

    #[error("Invalid transaction format")]
    InvalidTransaction,

    #[error("Transaction validation failed: {0}")]
    ValidationFailed(String),

    #[error("Unsupported transaction type: {0}")]
    UnsupportedTransactionType(String),

    #[error("RLP decode error: {0}")]
    RlpDecodeError(#[from] DecoderError),

    #[error("Hex decode error: {0}")]
    HexDecodeError(#[from] const_hex::FromHexError),

    #[error("Gas limit calculation failed: {0}")]
    GasLimitCalculationFailed(String),

    #[error("Transaction hash mismatch")]
    TransactionHashMismatch,

    #[error("Encryption/Decryption error: {0}")]
    CryptographyError(String),

    #[error("RLP-related error : {0}")]
    RlpError(#[from] alloy::rlp::Error)
}

pub type HandleTxResult<T> = std::result::Result<T, HandleTxError>;