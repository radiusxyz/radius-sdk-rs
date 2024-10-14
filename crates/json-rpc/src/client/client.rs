use std::pin::Pin;

use futures::{
    future::{select_ok, Fuse},
    FutureExt,
};
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};

use crate::client::{
    error::RpcClientError,
    id::Id,
    request::Request,
    response::{Payload, Response},
};

pub struct RpcClient(Client);

unsafe impl Send for RpcClient {}

unsafe impl Sync for RpcClient {}

impl Clone for RpcClient {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl RpcClient {
    pub fn new() -> Result<Self, RpcClientError> {
        let client = Client::builder()
            .build()
            .map_err(RpcClientError::BuildClient)?;

        Ok(Self(client))
    }

    pub async fn request<P, R>(
        &self,
        rpc_url: impl AsRef<str>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id>,
    ) -> Result<R, RpcClientError>
    where
        P: Serialize + Send,
        R: DeserializeOwned,
    {
        let request = Request::new(method, parameter, id);
        let response: Response<R> = self
            .0
            .post(rpc_url.as_ref())
            .json(&request)
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

    // pub async fn fetch<P, R>(
    //     &self,
    //     method: impl AsRef<str>,
    //     rpc_url_list: Vec<impl AsRef<str>>,
    //     parameter: &P,
    //     id: impl Into<Id>,
    // ) -> Result<R, RpcClientError>
    // where
    //     P: Serialize + Send,
    //     R: DeserializeOwned,
    // {
    //     let request = Request::owned(method, parameter, id);
    //     let fused_futures: Vec<Pin<Box<Fuse<_>>>> =
    // rpc_url_list.iter().map(|rpc_url| Box::pin(self.request(method,
    // id))).collect();     let response =
    // select_ok(fused_futures).await.map_err(|error|)?; }
}
