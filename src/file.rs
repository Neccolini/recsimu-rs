use crate::utils::read_json;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct InputFile {
    pub node_num: u32,
    pub total_cycles: u32,
    pub channel_num: u32,
    pub nodes: Vec<NodeInfo>,
    pub packets: Vec<PacketInfo>,
    pub neighbors: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub node_type: String,
}

#[derive(Deserialize)]
pub struct PacketInfo {
    pub cycle_num: u32,
    pub src_id: String,
    pub dest_id: String,
    pub msg: String,
}

impl InputFile {
    pub fn new(path: PathBuf) -> Self {
        // pathからファイルを読み込み、InputFileを作成する
        read_json::<InputFile>(path.clone())
            .map_err(|e| panic!("erro while reading {path:?}: {}", e))
            .unwrap()
    }
}
