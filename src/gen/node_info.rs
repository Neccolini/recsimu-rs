use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct NodeInfo {
    pub id: String,
    pub adjacent_nodes: Vec<String>,
    pub node_type: NodeType,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum NodeType {
    Coordinator,
    Router,
    EndDevice,
    UserType(String),
}
