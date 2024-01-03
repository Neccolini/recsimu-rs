pub mod default;
pub mod dynamic;
pub mod multi_tree;
pub mod packets;

use self::packets::{InjectionPacket, Packet};
use crate::network::option::UpdateOption;
use crate::{network::flit::Flit, sim::node_type::NodeType};

#[derive(Debug, Clone)]
pub enum CoreFunction {
    DefaultFunction(default::DefaultFunction),
    MultiTreeFunction(multi_tree::MultiTreeFunction),
    DynamicFunction(dynamic::DynamicFunction),
}

impl CoreFunction {
    pub(crate) fn new(rf_kind: &str, node_type: &NodeType, channel_num: u8) -> Self {
        #[allow(clippy::match_single_binding)]
        match rf_kind {
            "default" => CoreFunction::DefaultFunction(default::DefaultFunction::new(node_type)),
            "multi_tree" => CoreFunction::MultiTreeFunction(multi_tree::MultiTreeFunction::new(
                node_type,
                channel_num,
            )),
            "dynamic" => {
                CoreFunction::DynamicFunction(dynamic::DynamicFunction::new(node_type, channel_num))
            }
            _ => panic!("invalid routing function kind: {}", rf_kind),
        }
    }

    pub(crate) fn update(&mut self, option: Option<&UpdateOption>) {
        match self {
            CoreFunction::DefaultFunction(rf) => rf.update(),
            CoreFunction::MultiTreeFunction(rf) => rf.update(),
            CoreFunction::DynamicFunction(rf) => rf.update(option),
        }
    }

    pub(crate) fn push_new_packet(&mut self, packet: &InjectionPacket) {
        match self {
            CoreFunction::DefaultFunction(rf) => rf.push_new_packet(packet),
            CoreFunction::MultiTreeFunction(rf) => rf.push_new_packet(packet),
            CoreFunction::DynamicFunction(rf) => rf.push_new_packet(packet),
        }
    }

    pub(crate) fn send_packet(&mut self) -> Option<Packet> {
        match self {
            CoreFunction::DefaultFunction(rf) => rf.send_packet(),
            CoreFunction::MultiTreeFunction(rf) => rf.send_packet(),
            CoreFunction::DynamicFunction(rf) => rf.send_packet(),
        }
    }

    pub(crate) fn receive_packet(&mut self, packet: &Packet) {
        match self {
            CoreFunction::DefaultFunction(rf) => rf.receive_packet(packet),
            CoreFunction::MultiTreeFunction(rf) => rf.receive_packet(packet),
            CoreFunction::DynamicFunction(rf) => rf.receive_packet(packet),
        }
    }

    pub(crate) fn forward_flit(&mut self, flit: &Flit) -> Flit {
        match self {
            CoreFunction::DefaultFunction(rf) => rf.forward_flit(flit),
            CoreFunction::MultiTreeFunction(rf) => rf.forward_flit(flit),
            CoreFunction::DynamicFunction(rf) => rf.forward_flit(flit),
        }
    }

    pub(crate) fn get_id(&self) -> u32 {
        match self {
            CoreFunction::DefaultFunction(rf) => rf.id,
            CoreFunction::MultiTreeFunction(rf) => rf.id,
            CoreFunction::DynamicFunction(rf) => rf.id,
        }
    }

    pub(crate) fn is_joined(&self) -> bool {
        match self {
            CoreFunction::DefaultFunction(rf) => rf.is_joined(),
            CoreFunction::MultiTreeFunction(rf) => rf.is_joined(),
            CoreFunction::DynamicFunction(rf) => rf.is_joined(),
        }
    }

    pub(crate) fn get_message(&self, packet: &Packet) -> String {
        match self {
            CoreFunction::DefaultFunction(_) => {
                let p = packets::DefaultPacket::from_general(packet);
                p.message
            }

            CoreFunction::MultiTreeFunction(_) => {
                let p = packets::MultiTreePacket::from_general(packet);
                p.message
            }
            CoreFunction::DynamicFunction(_) => {
                let p = packets::DynamicPacket::from_general(packet);
                p.message
            }
        }
    }

    pub(crate) fn get_parent_id(&self) -> Vec<Option<u32>> {
        match self {
            CoreFunction::DefaultFunction(f) => vec![f.get_parent_id()],
            CoreFunction::MultiTreeFunction(f) => f.get_parent_id(),
            CoreFunction::DynamicFunction(f) => f.get_parent_id(),
        }
    }
}

impl Default for CoreFunction {
    fn default() -> Self {
        Self::new("default", &NodeType::Router, 1)
    }
}
