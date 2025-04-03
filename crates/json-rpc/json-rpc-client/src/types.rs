use std::cmp::Ordering;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::error::RpcClientError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Priority {
    High,
    Normal,
    Low,
    Custom(u8),
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        fn priority_value(p: &Priority) -> u8 {
            match p {
                Priority::High => 5,
                Priority::Normal => 3,
                Priority::Low => 1,
                Priority::Custom(v) => *v,
            }
        }

        priority_value(self).cmp(&priority_value(other))
    }
}

#[derive(Debug)]
pub struct PriorityRequest {
    pub priority: Priority,
    pub timestamp: u64,

    pub request: Request,

    pub channel_sender: tokio::sync::oneshot::Sender<Response>,
    pub sync_mode: bool,
}

impl PartialEq for PriorityRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
            && self.request == other.request
            && self.timestamp == other.timestamp
    }
}

impl Eq for PriorityRequest {}

impl Ord for PriorityRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

impl PartialOrd for PriorityRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct Request {
    jsonrpc: &'static str,
    method: String,
    params: Value,
    id: Id,
}

impl Request {
    const JSON_RPC: &str = "2.0";

    pub fn new<P: Serialize>(
        method: impl AsRef<str>,
        parameter: P,
        id: impl Into<Id>,
    ) -> Result<Self, serde_json::Error> {
        let params = serde_json::to_value(parameter)?;

        Ok(Self {
            jsonrpc: Self::JSON_RPC,
            method: method.as_ref().to_owned(),
            params,
            id: id.into(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    String(String),
    Number(i64),
    Null,
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

impl<T: Into<Id>> From<Option<T>> for Id {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Self::Null,
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Response {
    jsonrpc: String,
    #[serde(flatten)]
    payload: Payload,
    id: Id,
}

impl Response {
    pub fn parse_payload<T: DeserializeOwned>(self) -> Result<T, RpcClientError> {
        self.payload.parse()
    }
}

#[derive(Debug, Default, Serialize)]
pub struct BatchRequest(Vec<Request>);

impl BatchRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<P>(
        &mut self,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id>,
    ) -> Result<(), RpcClientError>
    where
        P: Serialize,
    {
        let rpc_request = Request::new(method, parameter, id).map_err(RpcClientError::Serialize)?;
        self.0.push(rpc_request);

        Ok(())
    }

    pub fn iter(&self) -> std::slice::Iter<Request> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Payload {
    Result(Value),
    Error {
        code: i32,
        message: String,
        data: Option<Id>,
    },
}

impl Payload {
    pub fn parse<T: DeserializeOwned>(self) -> Result<T, RpcClientError> {
        match self {
            Self::Result(value) => {
                serde_json::from_value::<T>(value).map_err(RpcClientError::Deserialize)
            }
            Self::Error {
                code: _,
                message,
                data: _,
            } => Err(RpcClientError::Response(message)),
        }
    }
}
