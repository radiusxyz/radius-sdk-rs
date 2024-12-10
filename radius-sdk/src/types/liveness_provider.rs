use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub enum ServiceProvider {
    Radius,
}

impl TryFrom<String> for ServiceProvider {
    type Error = ServiceProviderError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Radius" | "radius" => Ok(Self::Radius),
            _others => Err(ServiceProviderError(value)),
        }
    }
}

impl From<ServiceProvider> for String {
    fn from(value: ServiceProvider) -> Self {
        match value {
            ServiceProvider::Radius => "radius".to_owned(),
        }
    }
}

pub struct ServiceProviderError(String);

impl std::fmt::Debug for ServiceProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unsupported liveness provider: {}", self.0)
    }
}

impl std::fmt::Display for ServiceProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ServiceProviderError {}
