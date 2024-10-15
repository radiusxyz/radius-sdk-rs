pub use alloy::{
    primitives::*,
    rpc::types::{Block, Log},
};

alloy::sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Liveness,
    "src/contract/LivenessRadius.json"
);

pub enum Events {
    Block(Block),
    LivenessEvents(Liveness::LivenessEvents),
}

impl From<Liveness::LivenessEvents> for Events {
    fn from(value: Liveness::LivenessEvents) -> Self {
        Self::LivenessEvents(value)
    }
}
