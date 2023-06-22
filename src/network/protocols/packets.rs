use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GeneralPacket {
    pub message: String,
    pub dest_id: String,
    pub source_id: String,
    pub packet_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) struct DefaultPacket {
    pub(crate) message: String,
    pub(crate) dest_id: String,
    pub(crate) source_id: String,
    pub(crate) packet_id: u32,
}

impl DefaultPacket {
    pub(crate) fn new(gp: &GeneralPacket, packet_id: u32) -> Self {
        DefaultPacket {
            message: gp.message.clone(),
            dest_id: gp.dest_id.clone(),
            source_id: gp.source_id.clone(),
            packet_id,
        }
    }
}

pub fn encode_id(id: u32, from_id: &str) -> String {
    // <from_id>_<id>
    format!("{}_{}", from_id, id)
}

pub fn decode_id(id: &str) -> u32 {
    // <from_id>_<id>
    let id = id.split('_').collect::<Vec<&str>>();
    id[1].parse::<u32>().unwrap()
}
