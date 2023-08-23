use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GeneralPacket {
    pub data: Vec<u8>,
    pub dest_id: String,
    pub prev_id: String,
    pub next_id: String,
    pub source_id: String,
    pub packet_id: u32,
    pub channel_id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InjectionPacket {
    pub message: String,
    pub dest_id: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub(crate) struct DefaultPacket {
    pub(crate) message: String,
    pub(crate) dest_id: u32,
    pub(crate) prev_id: u32,
    pub(crate) next_id: u32,
    pub(crate) source_id: u32,
    pub(crate) packet_id: u32,
    pub(crate) channel_id: u32,
}

impl DefaultPacket {
    pub(crate) fn from_general(gp: &GeneralPacket) -> Self {
        // dataをでコード
        bincode::deserialize::<DefaultPacket>(gp.data.as_slice())
            .map_err(|e| {
                panic!("error occured while serializing a packet: {e:?}");
            })
            .unwrap()
    }
}

pub fn encode_id(id: u32, from_id: u32) -> String {
    // <from_id>_<id>
    format!("{}_{}", from_id, id)
}

pub fn decode_id(id: &str) -> u32 {
    // <from_id>_<id>
    let id = id.split('_').collect::<Vec<&str>>();
    id[1].parse::<u32>().unwrap()
}
