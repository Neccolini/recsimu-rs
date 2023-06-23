pub mod default;
pub mod packets;

use crate::{network::protocols::default::DefaultProtocol, sim::node_type::NodeType};

use self::packets::GeneralPacket;

pub enum NetworkProtocol {
    DefaultFunction(DefaultProtocol),
}

impl NetworkProtocol {
    pub(crate) fn new(rf_kind: String, node_type: NodeType) -> Self {
        #[allow(clippy::match_single_binding)]
        match rf_kind.as_str() {
            _ => NetworkProtocol::DefaultFunction(DefaultProtocol::new(node_type)),
        }
    }
    pub(crate) fn push_new_packet(&mut self, packet: &GeneralPacket) {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.push_new_packet(packet),
        }
    }

    pub(crate) fn send_packet(&mut self) -> Option<Vec<crate::network::flit::Flit>> {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.send_packet(),
        }
    }

    pub(crate) fn get_id(&self) -> String {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.id.clone(),
        }
    }
    pub(crate) fn receive_packet(&mut self, packet: &GeneralPacket) {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.receive_packet(packet),
        }
    }
}

impl Default for NetworkProtocol {
    fn default() -> Self {
        Self::new("".to_string(), NodeType::Router)
    }
}
