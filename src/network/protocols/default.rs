use super::packets::GeneralPacket;
use crate::network::protocols::packets::DefaultPacket;
use crate::network::Flit;
use crate::sim::node_type::NodeType;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DefaultProtocol {
    send_packet_buffer: VecDeque<DefaultPacket>,
    received_packet_buffer: VecDeque<DefaultPacket>,
    network_joined: bool,
    node_type: NodeType,
}

impl DefaultProtocol {
    pub fn new(node_type: NodeType) -> Self {
        let mut network_joined = false;
        if let NodeType::Coordinator = node_type {
            network_joined = true;
        }

        DefaultProtocol {
            send_packet_buffer: VecDeque::new(),
            received_packet_buffer: VecDeque::new(),
            network_joined,
            node_type,
        }
    }

    pub fn is_joined(&self) -> bool {
        self.network_joined
    }

    pub fn push_new_packet(&mut self, packet: &GeneralPacket) {
        self.send_packet_buffer
            .push_back(DefaultPacket::new(packet));
    }

    pub fn send_packet(&mut self) -> Option<Vec<Flit>> {
        if let Some(packet) = self.send_packet_buffer.pop_front() {
            return Some(packet.to_flits());
        }
        None
    }

    pub fn receive_packet(&mut self, packet: &GeneralPacket) {
        let packet = DefaultPacket::new(packet);
        self.received_packet_buffer.push_back(packet);
    }
}
