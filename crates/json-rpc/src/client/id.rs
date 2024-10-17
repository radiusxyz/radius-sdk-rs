use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
#[serde(from = "RawId")]
pub enum Id {
    Number(i64),
    String(String),
    Null,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum RawId {
    Number(Option<i64>),
    String(Option<String>),
}

impl From<RawId> for Id {
    fn from(value: RawId) -> Self {
        match value {
            RawId::Number(number) => match number {
                Some(number) => Self::Number(number),
                None => Self::Null,
            },
            RawId::String(string) => match string {
                Some(string) => Self::String(string),
                None => Self::Null,
            },
        }
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

#[test]
fn works() {
    let id_number = Id::from(0);
    let id_ser = serde_json::to_string(&id_number).unwrap();
    let id_de: Id = serde_json::from_str(&id_ser).unwrap();
    assert!(id_number == id_de);

    let id_string = Id::from("string");
    let id_ser = serde_json::to_string(&id_string).unwrap();
    let id_de: Id = serde_json::from_str(&id_ser).unwrap();
    assert!(id_string == id_de);

    let id_null = Id::Null;
    let id_ser = serde_json::to_string(&id_null).unwrap();
    let id_de: Id = serde_json::from_str(&id_ser).unwrap();
    assert!(id_null == id_de);
}
