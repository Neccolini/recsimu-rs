use super::packets::GeneralPacket;
use crate::hardware::flit::{data_to_flits, NodeId};
use crate::network::protocols::packets::DefaultPacket;
use crate::network::ChannelId;
use crate::network::Flit;
use crate::sim::node_type::NodeType;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DefaultProtocol {
    send_packet_buffer: VecDeque<DefaultPacket>,
    received_packet_buffer: VecDeque<DefaultPacket>,
    network_joined: bool,
    node_type: NodeType,
    table: HashMap<NodeId, NodeId>,
    packet_num_cnt: u32,
    parent_id: NodeId,
    children_id: Vec<NodeId>,
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
            table: HashMap::new(),
            packet_num_cnt: 0,
            parent_id: "".to_string(),
            children_id: Vec::new(),
        }
    }

    pub fn is_joined(&self) -> bool {
        self.network_joined
    }

    pub fn push_new_packet(&mut self, packet: &GeneralPacket) {
        self.send_packet_buffer
            .push_back(DefaultPacket::new(packet, self.packet_num_cnt));
        self.packet_num_cnt += 1;
    }

    pub fn send_packet(&mut self) -> Option<Vec<Flit>> {
        if let Some(packet) = self.send_packet_buffer.pop_front() {
            let data = bincode::serialize(&packet)
                .map_err(|e| {
                    panic!("error occured while serializing a packet: {e:?}");
                })
                .unwrap();

            let channel_id = self.channel_id(&packet.dest_id);
            let next_id = self.next_node_id(&packet.dest_id, &channel_id);
            let flits = data_to_flits(
                data,
                packet.source_id,
                packet.dest_id,
                next_id,
                packet.packet_id,
                channel_id,
            );
            return Some(flits);
        }
        None
    }

    pub fn receive_packet(&mut self, packet: &GeneralPacket) {
        let packet = DefaultPacket::new(packet, packet.packet_id);
        self.received_packet_buffer.push_back(packet);
    }
}

#[allow(unused_variables)]
impl DefaultProtocol {
    fn next_node_id(&self, dest_id: &NodeId, channel_id: &ChannelId) -> NodeId {
        // tableにdest_idがあればそれに対応するnode_idを返す
        // なければparent_idを返す
        if let Some(node_id) = self.table.get(dest_id) {
            return node_id.clone();
        }
        if dest_id == "broadcast" {
            return "broadcast".to_string();
        }
        self.parent_id.clone()
    }

    // フリットごとにチャネルを選択
    // デフォルト実装では仮想チャネルは使用しない
    fn channel_id(&self, dest_id: &NodeId) -> ChannelId {
        0
    }
}
