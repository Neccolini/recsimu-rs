use once_cell::sync::Lazy;
use std::env;

static LOG_FILE: Lazy<String> = Lazy::new(|| {
    let mut path = env::current_dir().unwrap();
    path.push("log");
    path.push("log.txt");
    path.to_str().unwrap().to_string()
});

pub struct Log {
    pub nodes_info: Vec<NodeLog>,
    pub packets_info: Vec<PacketLog>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            nodes_info: Vec::new(),
            packets_info: Vec::new(),
        }
    }
    pub fn write_log(&self) {
        // ログをファイルに書き込む
    }
}

impl Default for Log {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PacketLog {
    pub packet_id: String,
    pub from_id: String,
    pub dist_id: String,
    pub packet: String,
    pub flit_num: u32,
    pub send_cycle: u32,
    pub receive_cycle: Option<u32>,
    pub route_info: Vec<RouteInfo>,
    pub is_delivered: bool,
}
pub struct RouteInfo {
    pub node_id: String,
    pub cycle: u32,
}

pub struct NodeLog {}
