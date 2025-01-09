//! Lightweight JSON RPC client for sequencer with the following
//! functionalities:
//! - [RpcClient::multicast]
//! - [RpcClient::fetch]
use std::{pin::Pin, sync::Arc, time::Duration};

use futures::{
    future::{join_all, select_ok, Fuse},
    FutureExt,
};
use reqwest::{Client, ClientBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{
    value::{to_raw_value, RawValue},
    Value,
};

#[derive(Default)]
pub struct RpcClientBuilder(ClientBuilder);

impl RpcClientBuilder {
    /// Set the connection timeout in milliseconds.
    pub fn connection_timeout(self, timeout: u64) -> Self {
        let timeout = Duration::from_millis(timeout);
        let builder = self.0.connect_timeout(timeout);

        Self(builder)
    }

    pub fn build(self) -> Result<RpcClient, RpcClientError> {
        let rpc_client = RpcClient {
            inner: self.0.build().map_err(RpcClientError::Initialize)?,
        };

        Ok(rpc_client)
    }
}

pub struct RpcClient {
    inner: Client,
}

impl RpcClient {
    pub fn new() -> Result<Self, RpcClientError> {
        let rpc_client = Self {
            inner: ClientBuilder::default()
                .build()
                .map_err(RpcClientError::Initialize)?,
        };

        Ok(rpc_client)
    }

    async fn request_inner<P, R>(
        &self,
        url: impl AsRef<str>,
        payload: P,
    ) -> Result<R, RpcClientError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        self.inner
            .post(url.as_ref())
            .json(&payload)
            .send()
            .await
            .map_err(RpcClientError::Request)?
            .json::<R>()
            .await
            .map_err(RpcClientError::ParseResponse)
    }

    async fn fire_and_forget<P>(&self, url: impl AsRef<str>, payload: P)
    where
        P: Serialize,
    {
        let _ = self.inner.post(url.as_ref()).json(&payload).send().await;
    }

    /// Send an RPC request and wait for the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use radius_sdk::json_rpc::RpcClient;
    /// use serde::Serialize;
    ///
    /// #[derive(Clone, Debug, Serialize)]
    /// pub struct GetTransactionCount(Vec<String>);
    ///
    /// impl GetTransactionCount {
    ///     pub fn new(address: &str) -> Self {
    ///         Self(vec![address.to_owned(), "latest".to_owned()])
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let rpc_url = "http://127.0.0.1:8545";
    ///     let parameter = GetTransactionCount::new("0xc6972a7b408b83ceca73da73511df7ce9469608d");
    ///
    ///     let rpc_client = RpcClient::new().unwrap();
    ///
    ///     let rpc_response: String = rpc_client
    ///         .request(rpc_url, "eth_getTransactionCount", &parameter, "ID")
    ///         .await
    ///         .unwrap();
    ///
    ///     println!("{:?}", rpc_response);
    /// }
    /// ```
    pub async fn request<P, R>(
        &self,
        rpc_url: impl AsRef<str>,
        method: impl AsRef<str>,
        parameter: P,
        id: impl Into<Id>,
    ) -> Result<R, RpcClientError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let request =
            RequestObject::new(method, &parameter, id).map_err(RpcClientError::Serialize)?;
        let response: ResponseObject = self.request_inner(rpc_url, &request).await?;

        if response.id != request.id {
            return Err(RpcClientError::IdMismatch);
        }

        response.into_payload().parse::<R>()
    }

    /// Send a batch of several requests at the same time and get the response
    /// as a vector of RPC response object [Payload].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use radius_sdk::json_rpc::{BatchRequest, RpcClient};
    /// use serde::Serialize;
    ///
    /// #[derive(Clone, Debug, Serialize)]
    /// pub struct GetTransactionCount(Vec<String>);
    ///
    /// impl GetTransactionCount {
    ///     pub fn new(address: &str) -> Self {
    ///         Self(vec![address.to_owned(), "latest".to_owned()])
    ///     }
    /// }
    ///
    /// #[derive(Clone, Debug, Serialize)]
    /// pub struct Invalid(String);
    ///
    /// impl Invalid {
    ///     pub fn new(value: impl AsRef<str>) -> Self {
    ///         Self(value.as_ref().to_owned())
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let rpc_url = "http://127.0.0.1:8545";
    ///     let parameter_1 = GetTransactionCount::new("0xc6972a7b408b83ceca73da73511df7ce9469608d");
    ///     let parameter_2 = GetTransactionCount::new("0x5fae6ea6fc7d75aff5bc37faddffd382c8b12442");
    ///     let parameter_3 = GetTransactionCount::new("0x157787214841195353a31443338e493421d989d6");
    ///     let parameter_4 = Invalid::new("invalid");
    ///
    ///     let mut batch_request = BatchRequest::new();
    ///     batch_request
    ///         .push("eth_getTransactionCount", &parameter_1, "address_1")
    ///         .unwrap();
    ///     batch_request
    ///         .push("eth_getTransactionCount", &parameter_2, "address_2")
    ///         .unwrap();
    ///     batch_request
    ///         .push("eth_getTransactionCount", &parameter_3, "address_3")
    ///         .unwrap();
    ///     batch_request
    ///         .push("invalid", &parameter_4, "invalid")
    ///         .unwrap();
    ///
    ///     let rpc_client = RpcClient::new().unwrap();
    ///
    ///     let batch_response = rpc_client
    ///         .batch_request(rpc_url, &batch_request)
    ///         .await
    ///         .unwrap();
    ///
    ///     println!("{:?}", batch_response);
    /// }
    /// ```
    pub async fn batch_request(
        &self,
        rpc_url: impl AsRef<str>,
        batch_request: &BatchRequest,
    ) -> Result<Vec<Payload>, RpcClientError> {
        let response_objects: Vec<ResponseObject> =
            self.request_inner(rpc_url, &batch_request).await?;

        let payloads: Vec<Payload> = batch_request
            .iter()
            .zip(response_objects.into_iter())
            .map(|(request, response)| {
                if request.id == response.id {
                    Ok(response.into_payload())
                } else {
                    Err(RpcClientError::IdMismatch)
                }
            })
            .collect::<Result<Vec<Payload>, RpcClientError>>()?;

        Ok(payloads)
    }

    /// Send RPC requests to multiple endpoints. Once transactions are sent,
    /// the function short-circuits without waiting for responses.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use radius_sdk::json_rpc::RpcClient;
    /// use serde::Serialize;
    ///
    /// #[derive(Clone, Debug, Serialize)]
    /// pub struct GetTransactionCount(Vec<String>);
    ///
    /// impl GetTransactionCount {
    ///     pub fn new(address: &str) -> Self {
    ///         Self(vec![address.to_owned(), "latest".to_owned()])
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let rpc_urls = vec![
    ///         "http://127.0.0.1:8545",
    ///         "http://127.0.0.1:8546",
    ///         "http://127.0.0.1:8547",
    ///     ];
    ///     let parameter = GetTransactionCount::new("0xc6972a7b408b83ceca73da73511df7ce9469608d");
    ///
    ///     let rpc_client = RpcClient::new().unwrap();
    ///
    ///     rpc_client
    ///         .multicast(rpc_urls, "eth_getTransactionCount", &parameter, 0)
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    pub async fn multicast<P>(
        &self,
        rpc_urls: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id>,
    ) -> Result<(), RpcClientError>
    where
        P: Serialize,
    {
        let request: Arc<RequestObject> = RequestObject::new(method, parameter, id)
            .map_err(RpcClientError::Serialize)?
            .into();

        let tasks: Vec<_> = rpc_urls
            .into_iter()
            .map(|rpc_url| self.fire_and_forget(rpc_url, request.clone()))
            .collect();

        join_all(tasks).await;

        Ok(())
    }

    /// Send RPC requests to multiple endpoints and return the first successful
    /// response or an error if none of the responses succeeds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use radius_sdk::json_rpc::RpcClient;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// pub struct GetTransactionCount(Vec<String>);
    ///
    /// impl GetTransactionCount {
    ///     pub fn new(address: &str) -> Self {
    ///         Self(vec![address.to_owned(), "latest".to_owned()])
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let rpc_urls = vec![
    ///         "http://127.0.0.1:8545",
    ///         "http://127.0.0.1:8546",
    ///         "http://127.0.0.1:8547",
    ///     ];
    ///     let parameter = GetTransactionCount::new("0xc6972a7b408b83ceca73da73511df7ce9469608d");
    ///
    ///     let rpc_client = RpcClient::new().unwrap();
    ///
    ///     let first_successful_response: String = rpc_client
    ///         .fetch(rpc_urls, "eth_getTransactionCount", &parameter, 0)
    ///         .await
    ///         .unwrap();
    ///
    ///     println!("{:?}", first_successful_response);
    /// }
    /// ```
    pub async fn fetch<P, R>(
        &self,
        rpc_url_list: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id>,
    ) -> Result<R, RpcClientError>
    where
        P: Clone + Serialize,
        R: DeserializeOwned,
    {
        let method = method.as_ref().to_owned();
        let request: Arc<P> = parameter.clone().into();
        let id: Id = id.into();

        let fused_futures: Vec<Pin<Box<Fuse<_>>>> = rpc_url_list
            .into_iter()
            .map(|rpc_url| {
                Box::pin(
                    self.request::<Arc<P>, R>(rpc_url, method.clone(), request.clone(), id.clone())
                        .fuse(),
                )
            })
            .collect();

        let (response, _): (R, Vec<_>) = select_ok(fused_futures)
            .await
            .map_err(|error| RpcClientError::Fetch(error.into()))?;

        Ok(response)
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

#[derive(Debug, Serialize)]
struct RequestObject {
    jsonrpc: &'static str,
    method: String,
    params: Box<RawValue>,
    id: Id,
}

impl RequestObject {
    const JSON_RPC: &str = "2.0";

    pub fn new<P: Serialize>(
        method: impl AsRef<str>,
        parameter: P,
        id: impl Into<Id>,
    ) -> Result<Self, serde_json::Error> {
        let params = to_raw_value(&parameter)?;

        Ok(Self {
            jsonrpc: Self::JSON_RPC,
            method: method.as_ref().to_owned(),
            params,
            id: id.into(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ResponseObject {
    jsonrpc: String,
    #[serde(flatten)]
    payload: Payload,
    id: Id,
}

impl ResponseObject {
    fn into_payload(self) -> Payload {
        self.payload
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Default, Serialize)]
pub struct BatchRequest(Vec<RequestObject>);

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
        let rpc_request =
            RequestObject::new(method, parameter, id).map_err(RpcClientError::Serialize)?;
        self.0.push(rpc_request);

        Ok(())
    }

    fn iter(&self) -> std::slice::Iter<RequestObject> {
        self.0.iter()
    }
}

#[derive(Debug)]
pub enum RpcClientError {
    Initialize(reqwest::Error),
    Request(reqwest::Error),
    ParseResponse(reqwest::Error),
    Response(String),
    IdMismatch,
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    Fetch(Box<dyn std::error::Error>),
}

unsafe impl Send for RpcClientError {}

impl std::fmt::Display for RpcClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RpcClientError {}
