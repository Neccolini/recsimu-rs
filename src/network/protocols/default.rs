use super::packets::GeneralPacket;
use crate::log::{post_new_packet_log, update_packet_log, NewPacketLogInfo, UpdatePacketLogInfo};
use crate::network::flit::{data_to_flits, NodeId};
use crate::network::protocols::packets::{decode_id, DefaultPacket};
use crate::network::ChannelId;
use crate::network::Flit;
use crate::sim::node_type::NodeType;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DefaultProtocol {
    pub(crate) id: String,
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
            id: "".to_string(),
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
        // Logの処理
        let _ = self.log_handler(&packet.packet_id, packet);

        let packet = DefaultPacket::new(packet, decode_id(&packet.packet_id));
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

    fn log_handler(
        &self,
        packet_id: &String,
        packet: &GeneralPacket,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 最初に送る場合
        // routingする場合
        // 自分が宛先の場合
        if packet.source_id == self.id {
            // 最初に送る場合
            let new_packet_log_info = NewPacketLogInfo {
                packet_id: packet_id.clone(),
                from_id: self.id.clone(),
                dist_id: packet.dest_id.clone(),
                message: packet.message.clone(),
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
}

#[cfg(test)]
// send_packetのテスト
mod tests {
    use super::*;

    #[test]
    fn test_send_packet() {
        let mut protocol = DefaultProtocol::new(NodeType::Coordinator);
        protocol.id = "node1".to_string();
        protocol.network_joined = true;
        protocol.parent_id = "node2".to_string();
        protocol.children_id = vec!["node3".to_string(), "node4".to_string()];

        let packet = GeneralPacket {
            source_id: "node1".to_string(),
            dest_id: "node3".to_string(),
            message: "
            Momoyo Koyama as Karen Aijo
            Suzuko Mimori as Hikari Kagura
            Haruki Iwata as Mahiru Tsuyuzaki
            Aina Aiba as Claudine Saijo
            Maho Tomita as Maya Tendo
            Hinata Sato as Junna Hoshimi
            Moeka Koizumi as Nana Daiba
            Teru Ikuta as Futaba Isurugi
            Ayasa Ito as Kaoruko Hanayagi
            "
            .to_string(),
            packet_id: "packet1".to_string(),
        };
        protocol.push_new_packet(&packet);

        let flits = protocol.send_packet().unwrap();
        assert_eq!(flits.len(), 8);
    }
}
