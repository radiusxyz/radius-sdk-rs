mod id;
mod request;
mod response;

use std::pin::Pin;

use futures::{
    future::{join_all, select_ok, Fuse},
    FutureExt,
};
pub use id::Id;
use request::Request;
use reqwest::{Client, ClientBuilder};
use response::{Payload, Response};
use serde::{de::DeserializeOwned, Serialize};

pub struct RpcClientBuilder(ClientBuilder);

impl std::ops::Deref for RpcClientBuilder {
    type Target = ClientBuilder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RpcClientBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for RpcClientBuilder {
    fn default() -> Self {
        Self(ClientBuilder::default())
    }
}

impl RpcClientBuilder {
    pub fn build(self) -> Result<RpcClient, RpcClientError> {
        let http_client = self.0.build().map_err(RpcClientError::BuildClient)?;

        Ok(RpcClient(http_client))
    }
}

pub struct RpcClient(Client);

unsafe impl Send for RpcClient {}

unsafe impl Sync for RpcClient {}

impl Clone for RpcClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl RpcClient {
    pub fn builder() -> RpcClientBuilder {
        RpcClientBuilder::default()
    }

    pub fn new() -> Result<Self, RpcClientError> {
        let client = Client::builder()
            .build()
            .map_err(RpcClientError::BuildClient)?;

        Ok(Self(client))
    }

    async fn send<P, R>(&self, rpc_url: String, request: Request<P>) -> Result<R, RpcClientError>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let response: Response<R> = self
            .0
            .post(rpc_url)
            .json(request.as_ref())
            .send()
            .await
            .map_err(RpcClientError::Send)?
            .json()
            .await
            .map_err(RpcClientError::ParseResponse)?;

        if request.id() != response.id() {
            return Err(RpcClientError::IdMismatch);
        }

        match response.into_payload() {
            Payload::Result(result) => Ok(result),
            Payload::Error(error) => Err(error.into()),
        }
    }

    async fn fire_and_forget<P>(&self, rpc_url: String, request: Request<P>)
    where
        P: Clone + Serialize + Send,
    {
        let _ = self.0.post(rpc_url).json(request.as_ref()).send().await;
    }

    /// Send an RPC request and wait for the response.
    ///
    /// # Examples
    /// ```rust
    /// use radius_sdk::json_rpc::Id;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, Debug, Deserialize, Serialize)]
    /// pub struct AddUser {
    ///     name: String,
    ///     age: u8,
    /// }
    ///
    /// let user = AddUser {
    ///     name: "Username".to_owned(),
    ///     age: 50,
    /// };
    ///
    /// let client = RpcClient::new().unwrap();
    /// let response: String = client
    ///     .request("http://127.0.0.1:8000", "add_user", &user, Id::Null)
    ///     .await
    ///     .unwrap();
    ///
    /// println!("{}", response);
    /// ```
    pub async fn request<P, R>(
        &self,
        rpc_url: impl AsRef<str>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id>,
    ) -> Result<R, RpcClientError>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let request = Request::owned(method, parameter, id);
        let response = self
            .send::<P, R>(rpc_url.as_ref().to_owned(), request)
            .await?;

        Ok(response)
    }

    /// Send RPC requests to multiple endpoints. It is a fire-and-forget type of
    /// request that does not return `Result`.
    ///
    /// ```rust
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, Debug, Deserialize, Serialize)]
    /// pub struct AddUser {
    ///     name: String,
    ///     age: u8,
    /// }
    ///
    /// let user = AddUser {
    ///     name: "Username".to_owned(),
    ///     age: 50,
    /// };
    ///
    /// let client = RpcClient::new().unwrap();
    /// client
    ///     .multicast(
    ///         vec!["http://127.0.0.1:8000", "http://127.0.0.1:8001"],
    ///         "add_user",
    ///         &user,
    ///         0,
    ///     )
    ///     .await;
    /// ```
    pub async fn multicast<P>(
        &self,
        rpc_url_list: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id>,
    ) where
        P: Clone + Serialize + Send,
    {
        let request = Request::shared(method, parameter, id);
        let tasks: Vec<_> = rpc_url_list
            .into_iter()
            .map(|rpc_url| self.fire_and_forget(rpc_url.as_ref().to_owned(), request.clone()))
            .collect();

        join_all(tasks).await;
    }

    /// Send RPC requests to multiple endpoints and returns the first successful
    /// response or an error if none of the responses succeeds.
    /// ```rust
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Clone, Debug, Deserialize, Serialize)]
    /// pub struct AddUser {
    ///    name: String,
    ///    age: u8,
    /// }
    ///
    /// let user = AddUser {
    ///    name: "Username".to_owned(),
    ///    age: 50,
    /// };
    ///
    /// let client = RpcClient::new().unwrap();
    /// let response: String = client
    ///    .fetch(
    ///        vec!["http://127.0.0.1:8000", "http://127.0.0.1:8001"],
    ///        "add_user",
    ///        &user,
    ///        0,
    ///    )
    ///    .await
    ///    .unwrap();
    ///
    /// println!("{}", response);
    ////// ```
    pub async fn fetch<P, R>(
        &self,
        rpc_url_list: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id>,
    ) -> Result<R, RpcClientError>
    where
        P: Clone + Serialize + Send,
        R: DeserializeOwned,
    {
        let request = Request::shared(method, parameter, id);
        let fused_futures: Vec<Pin<Box<Fuse<_>>>> = rpc_url_list
            .into_iter()
            .map(|rpc_url| {
                Box::pin(
                    self.send::<P, R>(rpc_url.as_ref().to_owned(), request.clone())
                        .fuse(),
                )
            })
            .collect();
        let (response, _): (R, Vec<_>) = select_ok(fused_futures)
            .await
            .map_err(|_| RpcClientError::FetchRpcResponse)?;

        Ok(response)
    }
}

#[derive(Debug)]
pub enum RpcClientError {
    BuildClient(reqwest::Error),
    Send(reqwest::Error),
    ParseResponse(reqwest::Error),
    IdMismatch,
    Response(crate::client::response::ResponseError),
    FetchRpcResponse,
}

impl std::fmt::Display for RpcClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RpcClientError {}

impl From<crate::client::response::ResponseError> for RpcClientError {
    fn from(value: crate::client::response::ResponseError) -> Self {
        Self::Response(value)
    }
}
