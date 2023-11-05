use crate::hardware::constants::DATA_BYTE_PER_FLIT;
use crate::utils::div_ceil;
use serde::{Deserialize, Serialize};

use crate::network::vid::get_vid;
#[derive(Debug, Clone)]
pub struct GeneralPacket {
    pub data: Vec<u8>,
    pub dest_id: String,
    pub prev_id: String,
    pub next_id: String,
    pub source_id: String,
    pub packet_id: u32,
    pub channel_id: u32,
}

impl GeneralPacket {
    pub fn get_flits_len(&self) -> u32 {
        div_ceil(self.data.len() as u32, DATA_BYTE_PER_FLIT)
    }
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
        let mut dp = bincode::deserialize::<DefaultPacket>(gp.data.as_slice())
            .map_err(|e| {
                panic!("error occured while serializing a packet: {e:?}");
            })
            .unwrap();

        dp.prev_id = get_vid(&gp.prev_id).unwrap();
        dp.next_id = get_vid(&gp.next_id).unwrap();

        dp
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
