mod data_type;
mod in_memory;
mod on_disk;

pub use in_memory::{CachedKvStore, CachedKvStoreError, Value};
pub use kvstore_macros::*;
pub use on_disk::{kvstore, KvStore, KvStoreError, KvStoreOptions, Lock};
