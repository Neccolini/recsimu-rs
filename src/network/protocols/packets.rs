use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GeneralPacket {
    pub message: String,
    pub dest_id: String,
    pub source_id: String,
    pub packet_id: u32,
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
