use super::packets::{InjectionPacket, Packet};
use crate::network::core_functions::packets::MultiTreePacket;
use crate::network::flit::Flit;
use crate::network::vid::get_pid;
use crate::network::vid::get_vid;
use crate::recsimu_dbg;
use crate::sim::node_type::NodeType;
use once_cell::sync::Lazy;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::vec;

const BROADCAST_ID: u32 = u32::MAX;

static COORDINATOR_CNT: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(1));

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MultiTreeFunction {
    pub(crate) id: u32,
    channel_num: u8,
    send_packet_buffer: VecDeque<MultiTreePacket>,
    received_packet_buffer: VecDeque<MultiTreePacket>,
    network_joined: Vec<bool>,
    node_type: NodeType,
    tables: Vec<HashMap<u32, u32>>,
    packet_num_cnt: u32,
    parent_ids: Vec<u32>,
    children_id: Vec<u32>,
    channel_history: u8,
}

impl MultiTreeFunction {
    pub fn new(node_type: &NodeType, channel_num: u8) -> Self {
        let mut network_joined = vec![false; channel_num as usize];
        let id;
        let mut send_packet_buffer = VecDeque::new();
        let mut parent_ids = vec![0; channel_num as usize];

        if let NodeType::Coordinator = node_type {
            let mut coordinator_cnt = COORDINATOR_CNT
                .lock()
                .expect("failed to lock COORDINATOR_CNT");

            id = *coordinator_cnt;
            *coordinator_cnt += 1;

            if id == channel_num as u32 {
                *coordinator_cnt = 1;
            }

            assert!(
                id <= channel_num as u32,
                "id: {}, channel_num: {}",
                id,
                channel_num
            );

            parent_ids[id as usize - 1] = id;
            network_joined[id as usize - 1] = true;
        } else {
            // idはランダムな整数
            let mut rng = rand::thread_rng();
            // 2以上のランダムな整数
            id = rng.gen_range(channel_num as u32 + 1..u32::MAX);

            send_packet_buffer.push_back(MultiTreePacket {
                message: "preq".to_string(),
                packet_id: u32::MAX, // todo
                dest_id: BROADCAST_ID,
                source_id: id,
                prev_id: id,
                channel_id: u8::MAX,
                next_id: BROADCAST_ID,
            });
        }

        MultiTreeFunction {
            id,
            channel_num,
            send_packet_buffer,
            received_packet_buffer: VecDeque::new(),
            network_joined,
            node_type: node_type.clone(),
            // channel_num個のHashMapを持つようなVec
            tables: vec![HashMap::new(); channel_num as usize],
            packet_num_cnt: 0,
            parent_ids,
            children_id: Vec::new(),
            channel_history: channel_num - 1,
        }
    }

    pub fn is_joined(&self) -> bool {
        self.network_joined.iter().all(|&x| x)
    }

    pub fn push_new_packet(&mut self, packet: &InjectionPacket) {
        let dest_vid = get_vid(&packet.dest_id).unwrap();
        let channel_id = self.channel_id(dest_vid);
        let next_vid = self.next_node_id(dest_vid, channel_id);

        let packet = self.gen_packet(
            self.id,
            dest_vid,
            next_vid,
            channel_id,
            packet.message.clone(),
        );

        self.send_packet_buffer.push_back(packet);
        self.packet_num_cnt += 1;
    }

    pub fn send_packet(&mut self) -> Option<Packet> {
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

            return Some(Packet {
                data,
                packet_id: packet.packet_id,
                dest_id: dest_pid,
                source_id: src_pid,
                next_id: next_pid,
                prev_id: prev_pid,
                channel_id: packet.channel_id,
            });
        }
        None
    }

    pub fn receive_packet(&mut self, packet: &Packet) {
        let packet = MultiTreePacket::from_general(packet);
        self.received_packet_buffer.push_back(packet);
    }

    pub fn update(&mut self) {
        recsimu_dbg!(
            "{} {:?} {:?}",
            self.id,
            self.parent_ids,
            self.network_joined
        );
        // ランダムな確率でpackを送信
        let mut rng = rand::thread_rng();
        let p: f64 = rng.gen();
        // pは確率
        // 一定の確率で次のような動作を行うために用いる

        // もしparent_idsが空なら
        // 1. channel_idを決めずにpreqを送信
        // もしparent_ids.len < channel_numなら
        // 2. channel_idを決めてpreqを送信
        if p < 0.1 {
            if self.parent_ids.iter().all(|&x| x == 0) {
                // すべて0なら
                let new_packet = self.gen_packet(
                    self.id,
                    BROADCAST_ID,
                    BROADCAST_ID,
                    u8::MAX,
                    "preq".to_string(),
                );

                self.send_packet_buffer.push_back(new_packet);
            } else if self.parent_ids.iter().any(|&x| x == 0) {
                recsimu_dbg!("{} {:?}", self.id, self.parent_ids);
                // 0があれば
                // 0の要素のindexからランダムに選択
                let mut rng = rand::thread_rng();
                let indices: Vec<usize> = self
                    .parent_ids
                    .iter()
                    .enumerate()
                    .filter(|(_, &x)| x == 0)
                    .map(|(i, _)| i)
                    .collect();

                assert!(!indices.is_empty());

                let random_channel_id = indices[rng.gen_range(0..indices.len())];

                let new_packet = self.gen_packet(
                    self.id,
                    BROADCAST_ID,
                    BROADCAST_ID,
                    random_channel_id as u8,
                    "preq".to_string(),
                );

                self.send_packet_buffer.push_back(new_packet);
            }
        }

        // received_packet_bufferをすべて処理
        while let Some(packet) = self.received_packet_buffer.pop_front() {
            let reply = self.process_received_packet(&packet);
            for packet in reply {
                self.send_packet_buffer.push_back(packet);
            }
        }
    }

    pub fn forward_flit(&mut self, flit: &Flit) -> Flit {
        recsimu_dbg!("forward_flit {} {:?}", self.id, flit.clone());

        let dest_vid = get_vid(&flit.get_dest_id().unwrap()).unwrap();
        let next_vid = self.next_node_id(dest_vid, flit.get_channel_id().unwrap());

        let source_vid = get_vid(&flit.get_source_id().unwrap()).unwrap();
        let prev_vid = get_vid(&flit.get_prev_id().unwrap()).unwrap();

        self.update_table(source_vid, prev_vid, flit.get_channel_id().unwrap());

        let next_pid = get_pid(next_vid).unwrap();

        let mut new_flit = flit.clone();

        let _ = new_flit
            .set_prev_id(&flit.get_next_id().unwrap())
            .map_err(|e| {
                panic!("error occured while setting prev_id: {e:?}");
            });

        let _ = new_flit.set_next_id(&next_pid).map_err(|e| {
            panic!("error occured while setting next_id: {e:?}");
        });

        new_flit
    }
}

// private functions
#[allow(unused_variables)]
impl MultiTreeFunction {
    fn next_node_id(&self, dest_id: u32, channel_id: u8) -> u32 {
        // tableにdest_idがあればそれに対応するnode_idを返す
        // なければparent_idを返す
        recsimu_dbg!("{} {:?}", self.id, self.tables);
        let node_id = self
            .tables
            .get(channel_id as usize)
            .unwrap()
            .get(&dest_id)
            .copied();
        if let Some(node_id) = node_id {
            return node_id;
        }
        if dest_id == BROADCAST_ID {
            return BROADCAST_ID;
        }
        if self.parent_ids[channel_id as usize] == 0 {
            if self.node_type == NodeType::Router {
                panic!("Router: parent_id of channel {} is not set", channel_id);
            } else {
                panic!("Coordinator: parent_id is not set");
            }
        }

        self.parent_ids[channel_id as usize]
    }

    // フリットごとにチャネルを選択
    fn channel_id(&self, dest_id: u32) -> u8 {
        // ラウンドロビンでチャネルを選択
        (self.channel_history + 1) % self.channel_num
    }

    fn process_received_packet(&mut self, packet: &MultiTreePacket) -> Vec<MultiTreePacket> {
        // 自分宛でなければ何もしない
        if packet.next_id != self.id && packet.next_id != BROADCAST_ID {
            return vec![];
        }
        recsimu_dbg!("{} {:?}", self.id, packet.clone());
        match self.node_type {
            NodeType::Coordinator => self.process_received_packet_coordinator(packet),
            NodeType::Router => self.process_received_packet_router(packet),
            _ => panic!("unknown node type"),
        }
    }
    #[allow(unreachable_code)]
    fn process_received_packet_coordinator(
        &mut self,
        packet: &MultiTreePacket,
    ) -> Vec<MultiTreePacket> {
        match (packet.dest_id, packet.message.as_str()) {
            // BROADCAST "preq"
            (BROADCAST_ID, "preq") => {
                // packを返す
                let mut channel_id = packet.channel_id;

                if channel_id == u8::MAX {
                    channel_id = self.id as u8 - 1;
                }

                if channel_id as u32 != self.id - 1
                    && !self.network_joined[channel_id as usize]
                {
                    return vec![];
                }

                recsimu_dbg!("preq received at {} {:?}", self.id, packet);

                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    packet.source_id,
                    channel_id,
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
                // もし親IDが設定されていれば
                if self.parent_ids[packet.channel_id as usize] != 0 {
                    return vec![];
                }

                // 親IDを設定
                self.parent_ids[packet.channel_id as usize] = packet.source_id;

                // jreqを送信
                let packet = self.gen_packet(
                    self.id,
                    packet.channel_id as u32 + 1, // ここではchannel_idとcoordinator_idを一対一対応させている
                    packet.source_id,
                    packet.channel_id,
                    "jreq".to_string(),
                );

                return vec![packet];
            }

            // address to me, "jreq"
            (id, "jreq") if id == self.id => {
                recsimu_dbg!("jreq {} {:?}", self.id, packet.clone());
                if packet.channel_id as u32 + 1 != self.id {
                    return vec![];
                }
                self.update_table(packet.source_id, packet.prev_id, packet.channel_id);
                // jackを返す
                let channel_id = packet.channel_id;
                let next_id = self.next_node_id(packet.source_id, channel_id);

                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    next_id,
                    packet.channel_id,
                    "jack".to_string(),
                );
                return vec![packet];
            }

            // address to me, "jack"
            (id, "jack") if id == self.id => {
                // channel_id番目のnetwork_joinedをtrue
                self.network_joined[packet.channel_id as usize] = true;

                if self.parent_ids.iter().any(|&x| x == 0) {
                    let mut rng = rand::thread_rng();
                    let indices: Vec<usize> = self
                        .parent_ids
                        .iter()
                        .enumerate()
                        .filter(|(_, &x)| x == 0)
                        .map(|(i, _)| i)
                        .collect();

                    assert!(!indices.is_empty());

                    let random_channel_id = indices[rng.gen_range(0..indices.len())] as u8;

                    // 次のpreqを送信
                    let packet = self.gen_packet(
                        self.id,
                        BROADCAST_ID,
                        BROADCAST_ID,
                        random_channel_id,
                        "preq".to_string(),
                    );

                    return vec![packet];
                }
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
    fn process_received_packet_router(&mut self, packet: &MultiTreePacket) -> Vec<MultiTreePacket> {
        match (packet.dest_id, packet.message.as_str()) {
            // BROADCAST "preq"
            (BROADCAST_ID, "preq") => {
                if self.network_joined.iter().all(|&x| !x) {
                    // すべてfalseなら
                    return vec![];
                }

                if packet.channel_id <= self.channel_num
                    && !self.network_joined[packet.channel_id as usize]
                {
                    return vec![];
                }

                let mut rng = rand::thread_rng();
                let indices: Vec<usize> = self
                    .parent_ids
                    .iter()
                    .enumerate()
                    .filter(|(_, &x)| x != 0)
                    .map(|(i, _)| i)
                    .collect();

                assert!(!indices.is_empty());

                let random_channel_id = indices[rng.gen_range(0..indices.len())] as u8;

                // packを返す
                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    packet.source_id,
                    random_channel_id,
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
                if self.parent_ids[packet.channel_id as usize] != 0 {
                    return vec![];
                }

                // 親IDを設定
                self.parent_ids[packet.channel_id as usize] = packet.source_id;

                // jreqを送信
                let packet = self.gen_packet(
                    self.id,
                    packet.channel_id as u32 + 1, // ここではchannel_idとcoordinator_idを一対一対応させている
                    packet.source_id,
                    packet.channel_id,
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
                // channel_id番目のnetwork_joinedをtrue
                self.network_joined[packet.channel_id as usize] = true;

                if self.parent_ids.iter().any(|&x| x == 0) {
                    let mut rng = rand::thread_rng();
                    let indices: Vec<usize> = self
                        .parent_ids
                        .iter()
                        .enumerate()
                        .filter(|(_, &x)| x == 0)
                        .map(|(i, _)| i)
                        .collect();

                    assert!(!indices.is_empty());

                    let random_channel_id = indices[rng.gen_range(0..indices.len())] as u8;

                    // 次のpreqを送信
                    let packet = self.gen_packet(
                        self.id,
                        BROADCAST_ID,
                        BROADCAST_ID,
                        random_channel_id,
                        "preq".to_string(),
                    );

                    return vec![packet];
                }
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
                self.update_table(packet.source_id, packet.prev_id, packet.channel_id);
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
        channel_id: u8,
        message: String,
    ) -> MultiTreePacket {
        let packet_id = self.packet_num_cnt;
        self.packet_num_cnt += 1;

        MultiTreePacket {
            message,
            packet_id,
            dest_id,
            source_id: src_id,
            prev_id: self.id,
            channel_id,
            next_id,
        }
    }

    fn update_table(&mut self, dest_id: u32, next_id: u32, channel_id: u8) {
        // todo すでにあったら場合

        // テーブルに登録
        self.tables
            .get_mut(channel_id as usize)
            .unwrap()
            .insert(dest_id, next_id);
    }

    fn routing(&mut self, packet: &MultiTreePacket) -> Vec<MultiTreePacket> {
        assert!(packet.dest_id != self.id);
        assert!(packet.dest_id != BROADCAST_ID);

        // todo ここは，table用のget関数を用意する
        // もし宛先がテーブルにあれば
        let table = self.tables.get(packet.channel_id as usize).unwrap();

        if table.contains_key(&packet.dest_id) {
            // テーブルから次のノードを取得
            let next_id = table.get(&packet.dest_id).copied().unwrap();
            // パケットを生成
            let routing_packet = self.gen_packet(
                packet.source_id,
                packet.dest_id,
                next_id,
                packet.channel_id,
                packet.message.clone(),
            );
            return vec![routing_packet];
        } else {
            // 親ノードあて
            if let Some(parent_id) = self.parent_ids.get(packet.channel_id as usize).copied() {
                // パケットを生成
                let routing_packet = self.gen_packet(
                    packet.source_id,
                    packet.dest_id,
                    parent_id,
                    packet.channel_id,
                    packet.message.clone(),
                );
                return vec![routing_packet];
            }
        }

        vec![]
    }
}
