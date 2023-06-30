use super::packets::{GeneralPacket, InjectionPacket};
use crate::network::flit::NodeId;
use crate::network::protocols::packets::DefaultPacket;
use crate::network::vid::get_pid;
use crate::network::ChannelId;
use crate::sim::node_type::NodeType;
use rand::Rng;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DefaultProtocol {
    pub(crate) id: u32,
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
        let mut id = 0;
        if let NodeType::Coordinator = node_type {
            network_joined = true;
        } else {
            // idはランダムな整数
            let mut rng = rand::thread_rng();
            id = rng.gen();
        }

        DefaultProtocol {
            id,
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

    pub fn push_new_packet(&mut self, packet: &InjectionPacket) {
        let channel_id = self.channel_id(&packet.dest_id);
        let pid = get_pid(self.id).unwrap();

        let default_packet = DefaultPacket {
            message: packet.message.clone(),
            packet_id: self.packet_num_cnt,
            dest_id: packet.dest_id.clone(),
            source_id: pid,
            channel_id,
            next_id: self.next_node_id(&packet.dest_id, &channel_id),
        };

        self.send_packet_buffer.push_back(default_packet);
        self.packet_num_cnt += 1;
    }

    pub fn send_packet(&mut self) -> Option<GeneralPacket> {
        if let Some(packet) = self.send_packet_buffer.pop_front() {
            let data = bincode::serialize(&packet)
                .map_err(|e| {
                    panic!("error occured while serializing a packet: {e:?}");
                })
                .unwrap();

            let pid = get_pid(self.id).unwrap();

            return Some(GeneralPacket {
                data,
                packet_id: packet.packet_id,
                dest_id: packet.dest_id.clone(),
                source_id: pid,
                next_id: self.next_node_id(&packet.dest_id, &packet.channel_id),
                channel_id: self.channel_id(&packet.dest_id),
            });
        }
        None
    }

    pub fn receive_packet(&mut self, packet: &GeneralPacket) {
        let packet = DefaultPacket::from_general(packet);
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
        if self.parent_id.is_empty() {
            panic!("parent_id is not set");
        }

        self.parent_id.clone()
    }

    // フリットごとにチャネルを選択
    // デフォルト実装では仮想チャネルは使用しない
    fn channel_id(&self, dest_id: &NodeId) -> ChannelId {
        0
    }
    /*
    fn log_handler(
        &self,
        packet_id: &String,
        packet: &GeneralPacket,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if packet.source_id == self.id {
            // 最初に送る場合
            let new_packet_log_info = NewPacketLogInfo {
                packet_id: packet_id.clone(),
                from_id: self.id.clone(),
                dist_id: packet.dest_id.clone(),
                flit_num: 0,   // todo
                send_cycle: 0, // todo
            };

            post_new_packet_log(new_packet_log_info)?;
        } else if packet.dest_id == self.id {
            // 自分が宛先の場合
            let update_info = UpdatePacketLogInfo {
                last_receive_cycle: None, // todo
                route_info: Some(self.id.clone()),
                is_delivered: Some(true),
                flit_log: None,
            };
            update_packet_log(packet_id, &update_info)?;
        }
        Ok(())
    }
    */
}
