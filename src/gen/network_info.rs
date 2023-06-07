use super::node_info;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct NetworkInfo {
    pub topology: Topology,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Topology {
    #[serde(rename = "random")]
    Random(RandomTopology),
    #[serde(rename = "from_file")]
    FromFile(FromFileTopology),
    #[serde(rename = "mesh")]
    Mesh(MeshTopology),
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RandomTopology {
    pub node_num: u32,
    pub random_seed: Option<u32>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct FromFileTopology {
    pub node_num: u32,
    pub nodes: Vec<node_info::NodeInfo>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MeshTopology {
    pub node_num: u32,
    pub width: u32,
    pub height: u32,
}
