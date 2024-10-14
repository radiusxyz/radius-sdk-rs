use std::sync::Arc;

use serde::Serialize;

use crate::client::id::Id;

const JSONRPC: &'static str = "2.0";

#[derive(Clone, Debug, Serialize)]
pub struct Request<T>
where
    T: Serialize + Send,
{
    jsonrpc: &'static str,
    method: String,
    params: T,
    id: Id,
}

unsafe impl<T> Send for Request<T> where T: Serialize + Send {}

impl<T> Request<T>
where
    T: Serialize + Send,
{
    pub fn new(method: impl AsRef<str>, parameter: T, id: impl Into<Id>) -> Self {
        Self {
            jsonrpc: JSONRPC,
            method: method.as_ref().to_owned(),
            params: parameter,
            id: id.into(),
        }
    }

    pub fn owned(method: impl AsRef<str>, parameter: T, id: impl Into<Id>) -> Arc<Self> {
        Self {
            jsonrpc: JSONRPC,
            method: method.as_ref().to_owned(),
            params: parameter,
            id: id.into(),
        }
        .into()
    }

    pub fn id(&self) -> &Id {
        &self.id
    }
}
