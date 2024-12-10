use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub enum Platform {
    Ethereum,
    Local,
}

impl TryFrom<String> for Platform {
    type Error = PlatformError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Ethereum" | "ethereum" => Ok(Self::Ethereum),
            "Local" | "local" => Ok(Self::Local),
            _others => Err(PlatformError(value)),
        }
    }
}

impl From<Platform> for String {
    fn from(value: Platform) -> Self {
        match value {
            Platform::Ethereum => "ethereum".to_owned(),
            Platform::Local => "local".to_owned(),
        }
    }
}

pub struct PlatformError(String);

impl std::fmt::Debug for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unsupported platform: {}", self.0)
    }
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PlatformError {}
