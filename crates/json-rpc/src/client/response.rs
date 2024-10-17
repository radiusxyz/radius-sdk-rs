use serde::{Deserialize, Serialize};

use crate::client::id::Id;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response<T> {
    jsonrpc: String,
    #[serde(flatten)]
    payload: Payload<T>,
    id: Id,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Payload<T> {
    Result(T),
    Error(ResponseError),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResponseError {
    code: i32,
    message: String,
    data: Option<u32>,
}

impl<T> Response<T> {
    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn into_payload(self) -> Payload<T> {
        self.payload
    }
}
