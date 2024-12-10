use serde::{Deserialize, Serialize};
use signature::{Address, Signature};

use crate::types::{Platform, ServiceProvider};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencer {
    pub message: RegisterSequencerMessage,
    pub signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterSequencerMessage {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
    pub address: Address,
    pub external_rpc_url: String,
    pub cluster_rpc_url: String,
}

impl RegisterSequencer {
    pub const METHOD_NAME: &'static str = "register_sequencer";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeregisterSequencer {
    message: DeregisterSequencerMessage,
    signature: Signature,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeregisterSequencerMessage {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
    pub address: Address,
    pub external_rpc_url: String,
    pub cluster_rpc_url: String,
}

impl DeregisterSequencer {
    pub const METHOD_NAME: &'static str = "deregister_sequencer";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetExecutorRpcUrlList {
    pub executor_address_list: Vec<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetExecutorRpcUrlListResponse {
    pub executor_rpc_url_list: Vec<(String, Option<String>)>,
}

impl GetExecutorRpcUrlList {
    pub const METHOD_NAME: &'static str = "get_executor_rpc_url_list";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrl {
    pub address: Address,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlResponse {
    pub sequencer_rpc_url: (String, Option<(String, String)>),
}

impl GetSequencerRpcUrl {
    pub const METHOD_NAME: &'static str = "get_sequencer_rpc_url";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlList {
    pub sequencer_address_list: Vec<Address>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SequencerRpcInfo {
    pub address: String,
    pub external_rpc_url: Option<String>,
    pub cluster_rpc_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlListResponse {
    pub sequencer_rpc_url_list: Vec<SequencerRpcInfo>,
}

impl GetSequencerRpcUrlList {
    pub const METHOD_NAME: &'static str = "get_sequencer_rpc_url_list";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlListAtBlockHeight {
    pub platform: Platform,
    pub service_provider: ServiceProvider,
    pub cluster_id: String,
    pub address: Address,
    pub block_number: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetSequencerRpcUrlListAtBlockHeighResponse {
    pub sequencer_rpc_url_list: Vec<(String, Option<(String, String)>)>,
    pub block_number: u64,
}

impl GetSequencerRpcUrlListAtBlockHeight {
    pub const METHOD_NAME: &'static str = "get_sequencer_rpc_url_list_at_block_number";
}
