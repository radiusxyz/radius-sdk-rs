pub use alloy::{
    primitives::{Address, FixedBytes},
    rpc::types::{Block, Log},
};

alloy::sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Ssal,
    "src/contract/Ssal.json"
);

pub enum Events {
    Block(Block),
    SsalEvents(Ssal::SsalEvents),
}
