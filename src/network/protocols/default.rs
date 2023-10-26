use super::packets::{GeneralPacket, InjectionPacket};
use crate::network::flit::Flit;
use crate::network::protocols::packets::DefaultPacket;
use crate::network::vid::get_pid;
use crate::network::vid::get_vid;
use crate::sim::node_type::NodeType;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
const BROADCAST_ID: u32 = u32::MAX;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DefaultProtocol {
    pub(crate) id: u32,
    send_packet_buffer: VecDeque<DefaultPacket>,
    received_packet_buffer: VecDeque<DefaultPacket>,
    network_joined: bool,
    node_type: NodeType,
    table: HashMap<u32, u32>,
    packet_num_cnt: u32,
    parent_id: Option<u32>,
    children_id: Vec<u32>,
}

impl DefaultProtocol {
    pub fn new(node_type: NodeType) -> Self {
        let mut network_joined = false;
        let mut id = 0;
        let mut send_packet_buffer = VecDeque::new();

        if let NodeType::Coordinator = node_type {
            network_joined = true;
        } else {
            // idはランダムな整数
            let mut rng = rand::thread_rng();
            id = rng.gen();

            send_packet_buffer.push_back(DefaultPacket {
                message: "preq".to_string(),
                packet_id: u32::MAX, // todo
                dest_id: BROADCAST_ID,
                source_id: id,
                prev_id: id,
                channel_id: 0,
                next_id: BROADCAST_ID,
            });
        }

        DefaultProtocol {
            id,
            send_packet_buffer,
            received_packet_buffer: VecDeque::new(),
            network_joined,
            node_type,
            table: HashMap::new(),
            packet_num_cnt: 0,
            parent_id: None,
            children_id: Vec::new(),
        }
    }

    pub fn is_joined(&self) -> bool {
        self.network_joined
    }

    pub fn push_new_packet(&mut self, packet: &InjectionPacket) {
        let dest_vid = get_vid(packet.dest_id.clone()).unwrap();
        let channel_id = self.channel_id(dest_vid);
        let next_vid = self.next_node_id(dest_vid, channel_id);

        let default_packet = self.gen_packet(self.id, dest_vid, next_vid, packet.message.clone());
        dbg!(default_packet.clone());
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

            let prev_pid = get_pid(self.id).unwrap();
            let dest_pid = get_pid(packet.dest_id).unwrap();
            let src_pid = get_pid(packet.source_id).unwrap();
            let next_pid = get_pid(packet.next_id).unwrap(); // todo unwrapをなくす

            return Some(GeneralPacket {
                data,
                packet_id: packet.packet_id,
                dest_id: dest_pid,
                source_id: src_pid,
                next_id: next_pid,
                prev_id: prev_pid,
                channel_id: self.channel_id(packet.dest_id),
            });
        }
        None
    }

    pub fn receive_packet(&mut self, packet: &GeneralPacket) {
        let packet = DefaultPacket::from_general(packet);
        self.received_packet_buffer.push_back(packet);
    }

    pub fn update(&mut self) {
        // received_packet_bufferをすべて処理
        while let Some(packet) = self.received_packet_buffer.pop_front() {
            let reply = self.process_received_packet(&packet);
            for packet in reply {
                self.send_packet_buffer.push_back(packet);
            }
        }
    }

    pub fn forward_flit(&mut self, flit: &Flit) -> Flit {
        let dest_vid = get_vid(flit.get_dest_id().unwrap()).unwrap();
        let next_vid = self.next_node_id(dest_vid, flit.get_channel_id().unwrap());

        let next_pid = get_pid(next_vid).unwrap();

        let mut new_flit = flit.clone();

        let _ = new_flit
            .set_prev_id(flit.get_next_id().unwrap())
            .map_err(|e| {
                panic!("error occured while setting prev_id: {e:?}");
            });

        let _ = new_flit.set_next_id(next_pid).map_err(|e| {
            panic!("error occured while setting next_id: {e:?}");
        });

        new_flit
    }
}

// private functions
#[allow(unused_variables)]
impl DefaultProtocol {
    fn next_node_id(&self, dest_id: u32, channel_id: u32) -> u32 {
        // tableにdest_idがあればそれに対応するnode_idを返す
        // なければparent_idを返す
        if let Some(node_id) = self.table.get(&dest_id) {
            return *node_id;
        }
        if dest_id == BROADCAST_ID {
            return BROADCAST_ID;
        }
        if self.parent_id.is_none() {
            if self.node_type == NodeType::Router {
                panic!("Router: parent_id is not set");
            } else {
                panic!("Coordinator: parent_id is not set");
            }
        }

        self.parent_id.unwrap()
    }

    // フリットごとにチャネルを選択
    // デフォルト実装では仮想チャネルは使用しない
    fn channel_id(&self, dest_id: u32) -> u32 {
        0
    }

    fn process_received_packet(&mut self, packet: &DefaultPacket) -> Vec<DefaultPacket> {
        // 自分宛でなければ何もしない
        if packet.next_id != self.id && packet.next_id != BROADCAST_ID {
            return vec![];
        }

        match self.node_type {
            NodeType::Coordinator => self.process_received_packet_coordinator(packet),
            NodeType::Router => self.process_received_packet_router(packet),
            _ => panic!("unknown node type"),
        }
    }
    #[allow(unreachable_code)]
    fn process_received_packet_coordinator(
        &mut self,
        packet: &DefaultPacket,
    ) -> Vec<DefaultPacket> {
        match (packet.dest_id, packet.message.as_str()) {
            // BROADCAST "preq"
            (BROADCAST_ID, "preq") => {
                // packを返す
                let channel_id = self.channel_id(packet.source_id);
                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    packet.source_id,
                    "pack".to_string(),
                );
                return vec![packet];
            }

            // BROADCAST "pack"
            (BROADCAST_ID, "pack") => {
                panic!("pack destination cannot be broadcast");
            }

            // BROADCAST "jreq"
            (BROADCAST_ID, "jreq") => {
                panic!("jreq destination cannot be broadcast");
            }

            // BROADCAST "jack"
            (BROADCAST_ID, "jack") => {
                panic!("jack destination cannot be broadcast");
            }

            // BROADCAST user message
            (BROADCAST_ID, _) => {
                panic!("Broadcast message by user is not supported");
            }

            // address to me, "preq"
            (id, "preq") if id == self.id => {
                panic!("preq destination must be broadcast");
            }

            // address to me, "pack"
            (id, "pack") if id == self.id => {
                panic!("pack destination cannot be coordinator");
            }

            // address to me, "jreq"
            (id, "jreq") if id == self.id => {
                self.update_table(packet.source_id, packet.prev_id);
                // jackを返す
                let channel_id = self.channel_id(packet.source_id);
                let next_id = self.next_node_id(packet.source_id, channel_id);

                let packet =
                    self.gen_packet(self.id, packet.source_id, next_id, "jack".to_string());
                return vec![packet];
            }

            // address to me, "jack"
            (id, "jack") if id == self.id => {
                // tableに追加
                self.update_table(packet.source_id, packet.prev_id);
                // packを返す
                let channel_id = self.channel_id(packet.source_id);
                let next_id = self.next_node_id(packet.source_id, channel_id);

                let packet =
                    self.gen_packet(self.id, packet.source_id, next_id, "pack".to_string());
                return vec![packet];
            }

            // address to me, user packet
            (id, message) if id == self.id => {
                // message arrived
                // なにもしない
                return vec![];
            }

            // address to others, "preq"
            (_, "preq") => {
                panic!("preq destination must be broadcast");
            }

            // address to others, "pack"
            (_, "pack") => {
                panic!("pack cannot be reached to coordinator");
            }

            // address to others, "jreq"
            (_, "jreq") => {
                panic!("jreq destination must be coordinator");
            }

            // address to others, "jack"
            (_, "jack") => {
                panic!("jack cannot be reached to coordinator");
            }

            // address to others, user packet
            _ => {
                // ルーティングを行う
                return self.routing(packet);
            }
        }

        vec![] // should be unreachable
    }

    #[allow(unreachable_code)]
    fn process_received_packet_router(&mut self, packet: &DefaultPacket) -> Vec<DefaultPacket> {
        match (packet.dest_id, packet.message.as_str()) {
            // BROADCAST "preq"
            (BROADCAST_ID, "preq") => {
                if !self.is_joined() {
                    return vec![];
                }

                // packを返す
                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    packet.source_id,
                    "pack".to_string(),
                );
                return vec![packet];
            }

            // BROADCAST "pack"
            (BROADCAST_ID, "pack") => {
                panic!("pack destination cannot be broadcast");
            }

            // BROADCAST "jreq"
            (BROADCAST_ID, "jreq") => {
                panic!("jreq destination cannot be broadcast");
            }

            // BROADCAST "jack"
            (BROADCAST_ID, "jack") => {
                panic!("jack destination cannot be broadcast");
            }

            // BROADCAST user message
            (BROADCAST_ID, _) => {
                panic!("Broadcast message by user is not supported");
            }

            // BROADCAST "preq"
            (id, "preq") if id == self.id => {
                panic!("preq destination must be broadcast");
            }

            // address to me, "pack"
            (id, "pack") if id == self.id => {
                // もし親IDが設定されていれば
                if self.parent_id.is_some() {
                    return vec![];
                }

                // 親IDを設定
                self.parent_id = Some(packet.source_id);

                // jreqを送信
                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    packet.source_id,
                    "jreq".to_string(),
                );

                return vec![packet];
            }

            // address to me, "jreq"
            (id, "jreq") if id == self.id => {
                panic!("jreq destination must be coordinator");
            }

            // address to me, "jack"
            (id, "jack") if id == self.id => {
                // ネットワーク参加完了
                self.network_joined = true;
                dbg!("network joined");
                return vec![];
            }

            // address to me, user packet
            (id, message) if id == self.id => {
                // message arrived
                // なにもしない
                return vec![];
            }

            // address to others, "preq"
            (_, "preq") => {
                panic!("preq destination must be broadcast");
            }

            // address to others, "pack"
            (_, "pack") => {
                panic!("pack cannot be reached to other router");
            }

            // address to others, "jreq"
            (_, "jreq") => {
                // テーブルに登録する
                self.update_table(packet.source_id, packet.prev_id);
                // ルーティングを行う
                return self.routing(packet);
            }

            // address to others, "jack"
            (_, "jack") => {
                // ルーティングを行う
                return self.routing(packet);
            }

            // address to others, user packet
            _ => {
                // ルーティングを行う
                return self.routing(packet);
            }
        }

        vec![] // should be unreachable
    }

    fn gen_packet(
        &mut self,
        src_id: u32,
        dest_id: u32,
        next_id: u32,
        message: String,
    ) -> DefaultPacket {
        let packet_id = self.packet_num_cnt;
        self.packet_num_cnt += 1;

        DefaultPacket {
            message,
            packet_id,
            dest_id,
            source_id: src_id,
            prev_id: self.id,
            channel_id: self.channel_id(dest_id),
            next_id,
        }
    }

    fn update_table(&mut self, dest_id: u32, next_id: u32) {
        // todo すでにあったら場合

        self.table.insert(dest_id, next_id);
    }

    fn routing(&mut self, packet: &DefaultPacket) -> Vec<DefaultPacket> {
        assert!(packet.dest_id != self.id);
        assert!(packet.dest_id != BROADCAST_ID);

        // todo ここは，table用のget関数を用意する
        // もし宛先がテーブルにあれば
        if self.table.contains_key(&packet.dest_id) {
            // テーブルから次のノードを取得
            let next_id = self.table.get(&packet.dest_id).copied().unwrap();
            // パケットを生成
            let routing_packet = self.gen_packet(
                packet.source_id,
                packet.dest_id,
                next_id,
                packet.message.clone(),
            );
            return vec![routing_packet];
        } else {
            // 親ノードあて
            if let Some(parent_id) = self.parent_id {
                // パケットを生成
                let routing_packet = self.gen_packet(
                    packet.source_id,
                    packet.dest_id,
                    parent_id,
                    packet.message.clone(),
                );
                return vec![routing_packet];
            }
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    // process_received_packet_coordinatorをテスト

    use super::*;
    #[test]
    fn test_process_received_packet_coordinator() {
        let mut protocol = DefaultProtocol::new(NodeType::Coordinator);
        let rec_packet = DefaultPacket {
            message: "preq".to_string(),
            packet_id: 0,
            dest_id: BROADCAST_ID,
            source_id: 1,
            prev_id: 0,
            channel_id: 0,
            next_id: 0,
        };
        let packets = protocol.process_received_packet_coordinator(&rec_packet);

        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].message, "pack");
        assert_eq!(packets[0].dest_id, 1);
    }

    // process_received_packet_routerをテスト
    #[test]
    fn test_process_received_packet_router() {
        let mut protocol = DefaultProtocol::new(NodeType::Router);
        protocol.network_joined = true;
        let rec_packet = DefaultPacket {
            message: "preq".to_string(),
            packet_id: 0,
            dest_id: BROADCAST_ID,
            source_id: 1,
            prev_id: 0,
            channel_id: 0,
            next_id: 0,
        };
        let packets = protocol.process_received_packet_router(&rec_packet);

        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].message, "pack");
        assert_eq!(packets[0].dest_id, 1);
    }
}
