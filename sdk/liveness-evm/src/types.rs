pub use alloy::{
    primitives::*,
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

impl From<Ssal::SsalEvents> for Events {
    fn from(value: Ssal::SsalEvents) -> Self {
        Self::SsalEvents(value)
    }
}
