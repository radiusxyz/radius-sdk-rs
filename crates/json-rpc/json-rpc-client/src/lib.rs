mod error;
mod types;

use std::{
    collections::BinaryHeap,
    pin::Pin,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub use error::*;
use futures::{
    future::{join_all, select_ok, Fuse},
    FutureExt,
};
use num_cpus;
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::{oneshot, Mutex, Notify, Semaphore},
    task,
};
pub use types::*;

static MAX_CONCURRENT_RPC: Lazy<Semaphore> = Lazy::new(|| {
    let max_concurrency = num_cpus::get() * 4;
    Semaphore::new(max_concurrency)
});

#[derive(Default)]
pub struct RpcClientBuilder(ClientBuilder);

impl RpcClientBuilder {
    pub fn connection_timeout(self, timeout: u64) -> Self {
        Self(self.0.connect_timeout(Duration::from_millis(timeout)))
    }

    pub fn request_timeout(self, timeout: u64) -> Self {
        Self(self.0.timeout(Duration::from_millis(timeout)))
    }

    pub fn build(self) -> Result<RpcClient, RpcClientError> {
        Ok(RpcClient {
            inner: Arc::new(RpcClientInner {
                client: Arc::new(self.0.build().map_err(RpcClientError::Initialize)?),
                rpc_requests: Mutex::new(BinaryHeap::new()),
                notify: Notify::new(),
            }),
        })
    }
}

pub struct RpcClient {
    inner: Arc<RpcClientInner>,
}

struct RpcClientInner {
    client: Arc<Client>,
    rpc_requests: Mutex<BinaryHeap<(String, PriorityRequest)>>,
    notify: Notify,
}

impl RpcClient {
    pub fn builder() -> RpcClientBuilder {
        RpcClientBuilder::default()
    }

    pub fn new() -> Result<Arc<Self>, RpcClientError> {
        let rpc_client = Arc::new(Self {
            inner: Arc::new(RpcClientInner {
                client: Arc::new(
                    ClientBuilder::default()
                        .build()
                        .map_err(RpcClientError::Initialize)?,
                ),
                rpc_requests: Mutex::new(BinaryHeap::new()),
                notify: Notify::new(),
            }),
        });

        let cloned_rpc_client = Arc::clone(&rpc_client);
        tokio::spawn(async move { cloned_rpc_client.process_priority_requests().await });

        Ok(rpc_client)
    }

    async fn process_priority_requests(&self) {
        loop {
            self.inner.notify.notified().await;

            loop {
                let rpc_request = {
                    let mut rpc_requests = self.inner.rpc_requests.lock().await;
                    rpc_requests.pop()
                };

                if let Some((rpc_url, priority_request)) = rpc_request {
                    let client = Arc::clone(&self.inner.client);

                    let permit = MAX_CONCURRENT_RPC.acquire().await.unwrap();

                    if priority_request.sync_mode {
                        task::spawn(async move {
                            let response = client
                                .post(&rpc_url)
                                .json(&priority_request.request)
                                .send()
                                .await
                                .map_err(RpcClientError::Request)?
                                .json::<Response>()
                                .await
                                .map_err(RpcClientError::ParseResponse)?;

                            let _ = priority_request.channel_sender.send(response);
                            Ok::<_, RpcClientError>(())
                        });
                    } else {
                        task::spawn(async move {
                            let _ = client
                                .post(&rpc_url)
                                .json(&priority_request.request)
                                .send()
                                .await;
                        });
                    }

                    drop(permit);
                } else {
                    break;
                }
            }
        }
    }
    pub async fn fire_and_forget<P>(
        &self,
        rpc_url: impl AsRef<str>,
        method: impl AsRef<str>,
        parameter: P,
        id: impl Into<Id>,
    ) where
        P: Serialize,
    {
        self.fire_and_forget_with_priority(rpc_url, method, parameter, id, Priority::Low)
            .await;
    }

    pub async fn fire_and_forget_with_priority<P>(
        &self,
        rpc_url: impl AsRef<str>,
        method: impl AsRef<str>,
        parameter: P,
        id: impl Into<Id>,
        priority: Priority,
    ) where
        P: Serialize,
    {
        let request = Request::new(method, &parameter, id)
            .map_err(RpcClientError::Serialize)
            .unwrap();

        let priority_request = PriorityRequest {
            priority,
            request,
            channel_sender: oneshot::channel().0, // drop the receiver
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            sync_mode: false,
        };

        {
            let mut rpc_requests = self.inner.rpc_requests.lock().await;
            rpc_requests.push((rpc_url.as_ref().to_string(), priority_request));
        }

        self.inner.notify.notify_one();
    }

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
        self.request_with_priority(rpc_url, method, parameter, id, Priority::Low)
            .await
    }

    pub async fn request_with_priority<P, R>(
        &self,
        rpc_url: impl AsRef<str>,
        method: impl AsRef<str>,
        parameter: P,
        id: impl Into<Id>,
        priority: Priority,
    ) -> Result<R, RpcClientError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let (channel_sender, rx) = oneshot::channel();
        let request = Request::new(method, &parameter, id).map_err(RpcClientError::Serialize)?;

        let priority_request = PriorityRequest {
            priority,
            request,
            channel_sender,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            sync_mode: true,
        };

        {
            let mut rpc_requests = self.inner.rpc_requests.lock().await;
            rpc_requests.push((rpc_url.as_ref().to_string(), priority_request));
        }

        self.inner.notify.notify_one();

        let response = rx.await.map_err(|_| RpcClientError::ChannelRecv)?;

        response.parse_payload::<R>()
    }

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
        self.fetch_with_priority(rpc_url_list, method, parameter, id, Priority::Low)
            .await
    }

    pub async fn fetch_with_priority<P, R>(
        &self,
        rpc_url_list: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id>,
        priority: Priority,
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
                    self.request_with_priority::<Arc<P>, R>(
                        rpc_url,
                        method.clone(),
                        request.clone(),
                        id.clone(),
                        priority,
                    )
                    .fuse(),
                )
            })
            .collect();

        let (response, _): (R, Vec<_>) = select_ok(fused_futures)
            .await
            .map_err(|error| RpcClientError::Fetch(error.into()))?;

        Ok(response)
    }

    pub async fn fire_and_forget_multicast<P>(
        &self,
        rpc_urls: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id> + Clone,
    ) where
        P: Serialize,
    {
        self.fire_and_forget_multicast_with_priority(rpc_urls, method, parameter, id, Priority::Low)
            .await
    }

    pub async fn fire_and_forget_multicast_with_priority<P>(
        &self,
        rpc_urls: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id> + Clone,
        priority: Priority,
    ) where
        P: Serialize,
    {
        let tasks: Vec<_> = rpc_urls
            .into_iter()
            .map(|rpc_url| {
                self.fire_and_forget_with_priority(
                    rpc_url,
                    &method,
                    parameter,
                    id.clone(),
                    priority,
                )
            })
            .collect();

        join_all(tasks).await;
    }

    pub async fn multicast<P, R>(
        &self,
        rpc_urls: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id> + Clone,
    ) -> Vec<Result<R, RpcClientError>>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        self.multicast_with_priority(rpc_urls, method, parameter, id, Priority::Low)
            .await
    }

    pub async fn multicast_with_priority<P, R>(
        &self,
        rpc_urls: Vec<impl AsRef<str>>,
        method: impl AsRef<str>,
        parameter: &P,
        id: impl Into<Id> + Clone,
        priority: Priority,
    ) -> Vec<Result<R, RpcClientError>>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let tasks: Vec<_> = rpc_urls
            .into_iter()
            .map(|rpc_url| {
                self.request_with_priority(rpc_url, &method, parameter, id.clone(), priority)
            })
            .collect();

        join_all(tasks).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde::Deserialize;
    use serde_json::json;
    use types::Id;

    use super::*;

    async fn setup_client() -> Arc<RpcClient> {
        RpcClient::new().expect("Failed to create client")
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct GetVersionResponse {
        pub version: Version,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Version {
        pub code_version: String,
        pub database_version: String,
    }

    const RPC_URL: &str = "http://34.55.76.165:3000";

    #[tokio::test]
    async fn test_basic_request() {
        let client = setup_client().await;

        let response: GetVersionResponse = client
            .request(RPC_URL, "get_version", json!({}), Id::Number(1))
            .await
            .expect("Failed to request");

        println!("Response: {:?}", response);
    }

    #[tokio::test]
    async fn test_priority_request() {
        let client = setup_client().await;

        let cloned_client_low = Arc::clone(&client);
        let response_low = tokio::spawn(async move {
            cloned_client_low
                .request_with_priority(
                    RPC_URL,
                    "get_version",
                    json!({}),
                    Id::Number(2),
                    Priority::Low, // Low priority
                )
                .await
        });

        let cloned_client_high = Arc::clone(&client);
        let response_high = tokio::spawn(async move {
            cloned_client_high
                .request_with_priority(
                    RPC_URL,
                    "get_block_height",
                    json!({"rollup_id": "nodeinfra"}),
                    Id::Number(3),
                    Priority::High, // High priority
                )
                .await
        });

        let high_priority_result: u64 = response_high
            .await
            .unwrap()
            .expect("Fail to request high priority");
        let low_priority_result: GetVersionResponse = response_low
            .await
            .unwrap()
            .expect("Fail to request low priority");

        println!("High priority: {:?}", high_priority_result);
        println!("Low priority: {:?}", low_priority_result);
    }

    #[tokio::test]
    async fn test_multiple_requests() {
        let client = setup_client().await;

        let mut tasks = vec![];

        for i in 0..100 {
            let client_clone = Arc::clone(&client);
            let rpc_url_clone = RPC_URL.to_string();

            let task = tokio::spawn(async move {
                let response: GetVersionResponse = client_clone
                    .request_with_priority(
                        rpc_url_clone,
                        "get_version",
                        json!({}),
                        Id::Number(i + 10),
                        Priority::Custom(i as u8),
                    )
                    .await
                    .unwrap();
                println!("Request {} response: {:?}", i, response);
            });

            tasks.push(task);
        }

        tokio::task::yield_now().await;

        futures::future::join_all(tasks).await;
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct GetTransactionCount(Vec<String>);

    #[tokio::test]
    async fn test_fetch() {
        let client = setup_client().await;

        let rpc_urls = vec![
            "http://34.55.76.165:3000",
            "http://34.55.76.165:3000",
            "http://34.55.76.165:3000",
        ];

        let first_successful_response: GetVersionResponse = client
            .fetch(rpc_urls, "get_version", &json!({}), 0)
            .await
            .unwrap();

        println!("{:?}", first_successful_response);
    }
}
