#[cfg(feature = "bytes")]
mod bytes;
#[cfg(any(feature = "default", feature = "json"))]
mod json;

#[cfg(feature = "bytes")]
pub use bytes::{deserialize, serialize, DataTypeError};
#[cfg(any(feature = "default", feature = "json"))]
pub use json::{deserialize, serialize, DataTypeError};

mod prelude {
    pub use std::{any, fmt::Debug};

    pub use serde::{de::DeserializeOwned, ser::Serialize};
}
