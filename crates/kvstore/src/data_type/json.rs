use super::prelude::*;

pub fn deserialize<T>(data: impl AsRef<[u8]>) -> Result<T, DataTypeError>
where
    T: Debug + DeserializeOwned + Serialize,
{
    serde_json::from_slice(data.as_ref()).map_err(|error| DataTypeError::Deserialize {
        type_name: any::type_name::<T>(),
        error,
    })
}

pub fn serialize<T>(data: &T) -> Result<Vec<u8>, DataTypeError>
where
    T: Debug + Serialize,
{
    serde_json::to_vec(data).map_err(|error| DataTypeError::Serialize {
        type_name: any::type_name::<T>(),
        error,
    })
}

#[derive(Debug)]
pub enum DataTypeError {
    Deserialize {
        type_name: &'static str,
        error: serde_json::Error,
    },
    Serialize {
        type_name: &'static str,
        error: serde_json::Error,
    },
}

impl std::fmt::Display for DataTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DataTypeError {}
