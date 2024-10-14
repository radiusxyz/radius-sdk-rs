use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    String(Option<String>),
    Number(Option<i64>),
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Self::String(Some(value))
    }
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        Self::Number(Some(value))
    }
}
