use super::network_info::{FromFileTopology, MeshTopology, RandomTopology};
use super::node_info::NodeInfo;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub nodes: Vec<NodeInfo>,
}

impl Network {
    pub fn new(nodes: Vec<NodeInfo>) -> Self {
        Network { nodes }
    }
    pub fn json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}

pub fn generate_random(_: &RandomTopology) -> Result<Network, Box<dyn std::error::Error>> {
    todo!()
}

pub fn generate_mesh(_: &MeshTopology) -> Result<Network, Box<dyn std::error::Error>> {
    todo!()
}

pub fn generate_from_file(_: &FromFileTopology) -> Result<Network, Box<dyn std::error::Error>> {
    todo!()
}
