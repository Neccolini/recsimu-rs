pub mod default;
pub mod packets;

use crate::{
    network::flit::Flit, network::protocols::default::DefaultProtocol, sim::node_type::NodeType,
};

use self::packets::{GeneralPacket, InjectionPacket};

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

    pub(crate) fn update(&mut self) {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.update(),
        }
    }

    pub(crate) fn push_new_packet(&mut self, packet: &InjectionPacket) {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.push_new_packet(packet),
        }
    }

    pub(crate) fn send_packet(&mut self) -> Option<GeneralPacket> {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.send_packet(),
        }
    }

    pub(crate) fn receive_packet(&mut self, packet: &GeneralPacket) {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.receive_packet(packet),
        }
    }

    pub(crate) fn forward_flit(&mut self, flit: &Flit) -> Flit {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.forward_flit(flit),
        }
    }

    pub(crate) fn get_id(&self) -> u32 {
        match self {
            NetworkProtocol::DefaultFunction(rf) => rf.id,
        }
    }
}

impl Default for NetworkProtocol {
    fn default() -> Self {
        Self::new("".to_string(), NodeType::Router)
    }
}
