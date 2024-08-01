pub use alloy::{primitives::*, rpc::types::Log};

alloy::sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    AVSDirectory,
    "src/contract/IAVSDirectory.json"
);

alloy::sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    DelegationManager,
    "src/contract/IDelegationManager.json"
);

pub enum Events {}
