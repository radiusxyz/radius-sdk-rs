pub mod ethereum;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainType {
    Bitcoin,
    Ethereum,
}
