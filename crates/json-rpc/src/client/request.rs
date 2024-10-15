use std::sync::Arc;

use serde::Serialize;

use crate::client::id::Id;

const JSONRPC: &str = "2.0";

#[derive(Debug, Serialize)]
pub struct Request<'param, T>
where
    T: Serialize + Send,
{
    jsonrpc: &'static str,
    method: &'param str,
    params: &'param T,
    id: Id,
}

unsafe impl<'param, T> Send for Request<'param, T> where T: Serialize + Send {}

impl<'param, T> Request<'param, T>
where
    T: Serialize + Send,
{
    pub fn new(method: &'param str, parameter: &'param T, id: &impl Into<Id>) -> Self {
        Self {
            jsonrpc: JSONRPC,
            method,
            params: parameter,
            id: id.into(),
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }
}

pub struct RequestOwned<T>
where
    T: Clone + Serialize + Send,
{
    inner: Arc<RequestOwnedInner<T>>,
}

#[derive(Debug, Serialize)]
struct RequestOwnedInner<T>
where
    T: Clone + Serialize + Send,
{
    jsonrpc: &'static str,
    method: String,
    params: T,
    id: Id,
}

impl<T> std::ops::Deref for RequestOwned<T>
where
    T: Clone + Serialize + Send,
{
    type Target = RequestOwnedInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Clone for RequestOwned<T>
where
    T: Clone + Serialize + Send,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> RequestOwned<T>
where
    T: Clone + Serialize + Send,
{
    pub fn new(method: &str, parameter: &T, id: &impl Into<Id>) -> Self {
        let inner = RequestOwnedInner {
            jsonrpc: JSONRPC,
            method: method.to_owned(),
            params: parameter.to_owned(),
            id: id.into(),
        };

        Self {
            inner: Arc::new(inner),
        }
    }
}
