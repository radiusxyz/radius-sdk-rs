pub use alloy::{primitives::*, rpc::types::Log};

alloy::sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ValidationServiceManager,
    "src/contract/ValidationServiceManager.json"
);
