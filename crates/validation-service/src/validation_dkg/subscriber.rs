use std::fmt::Debug;

use alloy::{providers::Provider, rpc::types::Log};
use futures::StreamExt;

use super::{
    DkgValidationContract::{TaskCompleted, TaskCreated, TaskResponse},
    DkgValidationContractInstance, DkgValidationServiceError,
};
use crate::Future;

#[derive(Debug, Clone)]
pub struct Subscriber<P>(DkgValidationContractInstance<P>);

impl<P: Provider> Subscriber<P> {
    pub fn new(dkg_validation: DkgValidationContractInstance<P>) -> Self {
        Self(dkg_validation)
    }

    pub async fn subscribe_events<CB, Fut>(&self, callback: CB)
    where
        CB: Send + Fn(DkgValidationServiceEvent) -> Fut,
        Fut: Future<Output = ()>,
    {
        let mut task_created_event = self
            .0
            .TaskCreated_filter()
            .subscribe()
            .await
            .map_err(|e| DkgValidationServiceError::EventStreamCreateFailed(e))
            .unwrap()
            .into_stream();
        let mut task_response_event = self
            .0
            .TaskResponse_filter()
            .subscribe()
            .await
            .map_err(|e| DkgValidationServiceError::EventStreamCreateFailed(e))
            .unwrap()
            .into_stream();
        let mut task_complete_event = self
            .0
            .TaskCompleted_filter()
            .subscribe()
            .await
            .map_err(|e| DkgValidationServiceError::EventStreamCreateFailed(e))
            .unwrap()
            .into_stream();
        loop {
            tokio::select! {
                Some(Ok(event)) = task_created_event.next() => {
                    callback(event.into()).await
                },
                Some(Ok(event)) = task_response_event.next() => {
                    callback(event.into()).await
                },
                Some(Ok(event)) = task_complete_event.next() => {
                    callback(event.into()).await
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum DkgValidationServiceEvent {
    TaskCreated(TaskCreated),
    TaskResponse(TaskResponse),
    TaskCompleted(TaskCompleted),
}

macro_rules! impl_event_from {
    ($($event: ident), *) => {
        $(
            impl From<($event, Log)> for DkgValidationServiceEvent {
                fn from((event, _): ($event, Log)) -> Self {
                    DkgValidationServiceEvent::$event(event)
                }
            }
        )*
    }
}

impl_event_from!(TaskCreated, TaskResponse, TaskCompleted);
