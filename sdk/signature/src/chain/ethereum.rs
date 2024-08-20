use sha3::{Digest, Keccak256};

pub const EIP191_PREFIX: &str = "\x19Ethereum Signed Message:\n";

pub fn address_from_slice(slice: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(&slice[1..]);
    let output = &hasher.finalize_reset()[12..];

    output.to_vec()
}

pub fn eip191_hash_message(message: &[u8]) -> Vec<u8> {
    let len = message.len();
    let mut len_string_buffer = itoa::Buffer::new();
    let len_string = len_string_buffer.format(len);

    let mut ethereum_message = Vec::with_capacity(EIP191_PREFIX.len() + len_string.len() + len);
    ethereum_message.extend_from_slice(EIP191_PREFIX.as_bytes());
    ethereum_message.extend_from_slice(len_string.as_bytes());
    ethereum_message.extend_from_slice(message);

    let mut hasher = Keccak256::new();
    hasher.update(ethereum_message);
    let output = hasher.finalize_reset();

    output.to_vec()
}

pub fn y_parity_byte_non_eip155(parity: u8) -> u8 {
    parity + 27
}

pub fn recovery_id_from_y_parity_byte(recovery_id: u8) -> u8 {
    recovery_id - 27
}
