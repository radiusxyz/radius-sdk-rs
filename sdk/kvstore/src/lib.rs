mod database;
pub use database::{KvStore, Lock};

mod error;
pub use error::KvStoreError;
