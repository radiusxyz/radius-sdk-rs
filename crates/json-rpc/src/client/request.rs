use std::sync::Arc;

use serde::Serialize;

use crate::client::id::Id;

const JSONRPC: &'static str = "2.0";

#[derive(Clone, Debug, Serialize)]
pub struct Request<'param, T>
where
    T: Serialize + Send,
{
    jsonrpc: &'static str,
    method: String,
    params: &'param T,
    id: Id,
}

impl<'param, T> Request<'param, T>
where
    T: Serialize + Send,
{
    pub fn new(method: impl AsRef<str>, parameter: &'param T, id: impl Into<Id>) -> Self {
        Self {
            jsonrpc: JSONRPC,
            method: method.as_ref().to_owned(),
            params: parameter,
            id: id.into(),
        }
    }

    pub fn owned(method: impl AsRef<str>, parameter: &'param T, id: impl Into<Id>) -> Arc<Self> {
        Self {
            jsonrpc: JSONRPC,
            method: method.as_ref().to_owned(),
            params: parameter.to_owned(),
            id: id.into(),
        }
        .into()
    }

    pub fn id(&self) -> &Id {
        &self.id
    }
}
