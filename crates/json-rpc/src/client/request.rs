use std::sync::Arc;

use serde::Serialize;

use crate::client::id::Id;

const JSONRPC: &str = "2.0";

#[derive(Clone)]
pub enum Request<T>
where
    T: Clone + Serialize,
{
    Owned(RpcRequest<T>),
    Shared(Arc<RpcRequest<T>>),
}

unsafe impl<T> Send for Request<T> where T: Clone + Serialize {}

impl<T> std::ops::Deref for Request<T>
where
    T: Clone + Serialize,
{
    type Target = RpcRequest<T>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(inner) => inner,
            Self::Shared(inner) => inner,
        }
    }
}

impl<T> AsRef<RpcRequest<T>> for Request<T>
where
    T: Clone + Serialize,
{
    fn as_ref(&self) -> &RpcRequest<T> {
        match self {
            Self::Owned(inner) => inner,
            Self::Shared(inner) => inner,
        }
    }
}

impl<T> Request<T>
where
    T: Clone + Serialize,
{
    pub fn owned(method: impl AsRef<str>, parameter: &T, id: impl Into<Id>) -> Self {
        Self::Owned(RpcRequest::new(method, parameter, id))
    }

    pub fn shared(method: impl AsRef<str>, parameter: &T, id: impl Into<Id>) -> Self {
        Self::Shared(Arc::new(RpcRequest::new(method, parameter, id)))
    }
}

#[derive(Clone, Serialize)]
pub struct RpcRequest<T>
where
    T: Clone + Serialize,
{
    jsonrpc: &'static str,
    method: String,
    params: T,
    id: Id,
}

impl<T> RpcRequest<T>
where
    T: Clone + Serialize,
{
    pub fn new(method: impl AsRef<str>, parameter: &T, id: impl Into<Id>) -> Self {
        Self {
            jsonrpc: JSONRPC,
            method: method.as_ref().to_owned(),
            params: parameter.to_owned(),
            id: id.into(),
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }
}
