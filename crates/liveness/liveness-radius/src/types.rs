pub use alloy::{primitives, rpc};

alloy::sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Liveness,
    "src/contract/LivenessServiceManager.json"
);

pub enum Events {
    Block(rpc::types::Header),
    LivenessEvents(Liveness::LivenessEvents, rpc::types::Log),
}
