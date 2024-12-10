use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub enum ValidationServiceProvider {
    EigenLayer,
    Symbiotic,
}

impl TryFrom<String> for ValidationServiceProvider {
    type Error = ValidationServiceProviderError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "EigenLayer" | "eigenlayer" => Ok(Self::EigenLayer),
            "Symbiotic" | "symbiotic" => Ok(Self::Symbiotic),
            _others => Err(ValidationServiceProviderError(value)),
        }
    }
}

impl From<ValidationServiceProvider> for String {
    fn from(value: ValidationServiceProvider) -> Self {
        match value {
            ValidationServiceProvider::EigenLayer => "eigenlayer".to_owned(),
            ValidationServiceProvider::Symbiotic => "symbiotic".to_owned(),
        }
    }
}

pub struct ValidationServiceProviderError(String);

impl std::fmt::Debug for ValidationServiceProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unsupported validation provider: {}", self.0)
    }
}

impl std::fmt::Display for ValidationServiceProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ValidationServiceProviderError {}
