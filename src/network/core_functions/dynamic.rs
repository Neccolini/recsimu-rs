use super::packets::{InjectionPacket, Packet};
use crate::network::core_functions::packets::DynamicPacket;
use crate::network::flit::Flit;
use crate::network::option::UpdateOption;
use crate::network::vid::get_pid;
use crate::network::vid::get_vid;
use crate::recsimu_dbg;
use crate::sim::node_type::NodeType;
use crate::sim::rec;
use once_cell::sync::Lazy;
use rand::Rng;
use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::vec;

const BROADCAST_ID: u32 = u32::MAX;
const MAX_REC_CNT: u8 = 3;

static COORDINATOR_CNT: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(1));

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DynamicFunction {
    pub(crate) id: u32,
    channel_num: u8,
    send_packet_buffer: VecDeque<DynamicPacket>,
    received_packet_buffer: VecDeque<DynamicPacket>,
    network_joined: Vec<bool>,
    node_type: NodeType,
    tables: Vec<HashMap<u32, u32>>,
    packet_num_cnt: u32,
    root_ids: Vec<u32>,
    parent_ids: Vec<u32>,
    children_ids: ChildrenTable,
    channel_history: u8,
    rec_info: HashMap<u8, RecInfo>,
}

impl DynamicFunction {
    pub fn new(node_type: &NodeType, channel_num: u8) -> Self {
        let mut network_joined = vec![false; channel_num as usize];
        let id;
        let mut send_packet_buffer = VecDeque::new();
        let mut parent_ids = vec![0; channel_num as usize];
        let root_ids = (1..=channel_num as u32).collect::<Vec<u32>>();

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

            send_packet_buffer.push_back(DynamicPacket {
                message: "preq".to_string(),
                packet_id: u32::MAX, // todo
                dest_id: BROADCAST_ID,
                source_id: id,
                prev_id: id,
                channel_id: u8::MAX,
                next_id: BROADCAST_ID,
            });
        }

        DynamicFunction {
            id,
            channel_num,
            send_packet_buffer,
            received_packet_buffer: VecDeque::new(),
            network_joined,
            node_type: node_type.clone(),
            // channel_num個のHashMapを持つようなVec
            tables: vec![HashMap::new(); channel_num as usize],
            packet_num_cnt: 0,
            root_ids,
            parent_ids,
            children_ids: ChildrenTable::new(channel_num),
            channel_history: channel_num - 1,
            rec_info: HashMap::new(),
        }
    }

    pub fn is_joined(&self) -> bool {
        self.network_joined.iter().all(|&x| x)
    }

    pub fn get_parent_id(&self) -> Vec<Option<u32>> {
        self.parent_ids
            .iter()
            .map(|&x| if x == 0 { None } else { Some(x) })
            .collect()
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
        let packet = DynamicPacket::from_general(packet);
        self.received_packet_buffer.push_back(packet);
    }

    pub fn update(&mut self, option: Option<&UpdateOption>) {
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
            if !self.rec_info.is_empty() {
            } else if self.parent_ids.iter().all(|&x| x == 0) {
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

        self.dynamic_rec_in_update(option);

        // received_packet_bufferをすべて処理
        while let Some(packet) = self.received_packet_buffer.pop_front() {
            let reply = self.process_received_packet(&packet);

            for packet in reply {
                self.send_packet_buffer.push_back(packet);
            }
        }
    }

    pub fn forward_flit(&mut self, flit: &Flit) -> Flit {
        let dest_vid = get_vid(&flit.get_dest_id().unwrap()).unwrap();
        let next_vid = self.next_node_id(dest_vid, flit.get_channel_id().unwrap());

        let source_vid = get_vid(&flit.get_source_id().unwrap()).unwrap();
        let prev_vid = get_vid(&flit.get_prev_id().unwrap()).unwrap();

        if prev_vid
            != self
                .parent_ids
                .get(flit.get_channel_id().unwrap() as usize)
                .copied()
                .unwrap()
        {
            self.children_ids
                .insert(flit.get_channel_id().unwrap(), prev_vid);
        }

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
impl DynamicFunction {
    fn next_node_id(&self, dest_id: u32, channel_id: u8) -> u32 {
        // tableにdest_idがあればそれに対応するnode_idを返す
        // なければparent_idを返す
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
    fn channel_id(&mut self, dest_id: u32) -> u8 {
        // ラウンドロビンでチャネルを選択
        let channel_id = (self.channel_history + 1) % self.channel_num;
        self.channel_history = channel_id;
        channel_id
    }

    fn process_received_packet(&mut self, packet: &DynamicPacket) -> Vec<DynamicPacket> {
        print_packet(self.id, &packet);
        // 自分宛でなければ何もしない
        if packet.next_id != self.id && packet.next_id != BROADCAST_ID {
            return vec![];
        }

        let (is_rec_packets, res) = self.dynamic_rec(&packet);
        eprintln!("send packets: ");
        res.iter().map(|x| print_packet(self.id, &x));

        if is_rec_packets {
            return res;
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
        packet: &DynamicPacket,
    ) -> Vec<DynamicPacket> {
        match (packet.dest_id, packet.message.as_str()) {
            // BROADCAST "preq"
            (BROADCAST_ID, "preq") => {
                // packを返す
                let mut channel_id = packet.channel_id;

                if channel_id == u8::MAX {
                    channel_id = self.id as u8 - 1;
                }

                if channel_id as u32 != self.id - 1 && !self.network_joined[channel_id as usize] {
                    return vec![];
                }

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
                if packet.channel_id as u32 + 1 != self.id {
                    return vec![];
                }
                self.update_table(packet.source_id, packet.prev_id, packet.channel_id);

                if packet.source_id == packet.prev_id {
                    self.children_ids
                        .insert(packet.channel_id, packet.source_id);
                }

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
                eprintln!("network joined {} {}", self.id, packet.prev_id);
                self.network_joined[packet.channel_id as usize] = true;

                if self.is_joined() {
                    // recsimu_dbg!("{} {:?}", get_pid(self.id).unwrap(), self.parent_ids.iter().map(|&x| get_pid(x).unwrap()).collect::<Vec<String>>());
                }

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
    fn process_received_packet_router(&mut self, packet: &DynamicPacket) -> Vec<DynamicPacket> {
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

                if indices.is_empty() {
                    return vec![];
                }

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
                eprintln!("network joined {} {}", self.id, packet.prev_id);
                // channel_id番目のnetwork_joinedをtrue
                self.network_joined[packet.channel_id as usize] = true;

                if self.is_joined() {
                    // recsimu_dbg!("{} {:?}", get_pid(self.id).unwrap(), self.parent_ids.iter().map(|&x| get_pid(x).unwrap()).collect::<Vec<String>>());
                }

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

                self.children_ids.insert(packet.channel_id, packet.prev_id);

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
    ) -> DynamicPacket {
        let packet_id = self.packet_num_cnt;
        self.packet_num_cnt += 1;

        DynamicPacket {
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

    fn routing(&mut self, packet: &DynamicPacket) -> Vec<DynamicPacket> {
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

    fn dynamic_rec(&mut self, packet: &DynamicPacket) -> (bool, Vec<DynamicPacket>) {
        let mut res = vec![];

        if packet.message == "rec" {
            // 自身の子供に対して再構成フラグを送信
            for &child_id in self
                .children_ids
                .get(&packet.channel_id)
                .cloned()
                .unwrap()
                .iter()
            {
                let packet = self.gen_packet(
                    packet.source_id,
                    child_id,
                    child_id,
                    packet.channel_id,
                    "rec".to_string(),
                );
                res.push(packet);
            }

            let rec_info = RecInfo::new(
                packet.channel_id,
                &self.children_ids.get(&packet.channel_id).unwrap(),
                packet.source_id,
                Some(self.parent_ids[packet.channel_id as usize]),
            );

            self.rec_info.insert(packet.channel_id, rec_info);

            self.root_ids[packet.channel_id as usize] = packet.source_id;
            // 葉ノードなら
            eprintln!("{} {:?}", self.id, self.parent_ids.clone());
            if self.children_ids.is_empty(&packet.channel_id) {
                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    self.parent_ids[packet.channel_id as usize],
                    packet.channel_id,
                    "recr".to_string(),
                );
                res.push(packet);
            }
        }
        // 自身の親に対して再構成フラグ完了を伝える
        else if packet.message == "recr" {
            // 自身の親がいなければ（自身が再構成開始ノードなら）
            if packet.dest_id == self.id {
                if !self.rec_info.contains_key(&packet.channel_id) {
                    return (true, vec![]);
                }
                // 探索を開始する
                let rec_info = self
                    .rec_info
                    .get_mut(&packet.channel_id)
                    .expect("error receiving recr");

                assert!(self.id == rec_info.init_node_id);

                let init_node_id = rec_info.init_node_id.to_string().clone();

                /*
                if rec_info.is_rec {
                    return (true, vec![]);
                }
                */

                rec_info.begin();

                let packet = self.gen_packet(
                    self.id,
                    BROADCAST_ID,
                    BROADCAST_ID,
                    packet.channel_id,
                    "R".to_string() + init_node_id.as_str(),
                );

                res.push(packet);
            } else {
                return (true, self.routing(packet));
            }
        }
        // transferメッセージを受け取った場合
        else if packet.message == "tf" {
            if !self.rec_info.contains_key(&packet.channel_id) {
                return (true, vec![]);
            }

            let rec_info = self.rec_info.get_mut(&packet.channel_id).unwrap();

            let init_node_id = rec_info.init_node_id.to_string().clone();

            rec_info.begin();

            let packet = self.gen_packet(
                self.id,
                BROADCAST_ID,
                BROADCAST_ID,
                packet.channel_id,
                "R".to_string() + init_node_id.as_str(),
            );
            res.push(packet);
        }
        // failメッセージを子から受け取った場合
        else if packet.message == "fl" {
            if !self.rec_info.contains_key(&packet.channel_id) {
                return (true, vec![]);
            }
            //別の子にtfを送信
            let rec_info = self.rec_info.get_mut(&packet.channel_id).unwrap();

            let is_empty = rec_info.children.is_empty();
            if is_empty {
                rec_info.end();

                if let Some(old_parent_id) = rec_info.old_parent_id {
                    // failメッセージを送信
                    let packet = self.gen_packet(
                        self.id,
                        old_parent_id,
                        old_parent_id,
                        packet.channel_id,
                        "fl".to_string(),
                    );

                    res.push(packet);
                } else {
                    println!("no path found");
                }
            } else {
                // 次の子にtfを送信
                let next_id = rec_info.children.iter().next().cloned().unwrap(); // emptyかは確認したのでunwrapして良い
                rec_info.children.remove(&next_id);

                let packet = self.gen_packet(
                    self.id,
                    next_id,
                    next_id,
                    packet.channel_id,
                    "tf".to_string(),
                );
                res.push(packet);
            }
        }
        // Rreqを受け取った
        else if packet.message.starts_with('R') {
            let init_id = packet
                .message
                .chars()
                .skip(1)
                .collect::<String>()
                .parse::<u32>()
                .unwrap();

            if !self.rec_info.contains_key(&packet.channel_id) {
                // 自身が再構成サブツリーに含まれない場合
                // 自分が送信者の親ノードになる
                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    packet.source_id,
                    packet.channel_id,
                    "P".to_string()
                        + self.root_ids[packet.channel_id as usize]
                            .to_string()
                            .as_str(),
                );

                res.push(packet);

                return (true, res);
            }

            let rec_info = self
                .rec_info
                .get_mut(&packet.channel_id)
                .expect("error receiving Rreq");

            // 自分のツリーのidが小さい場合のみ返答する
            if init_id <= rec_info.init_node_id {
                return (true, res);
            } else {
                let packet = self.gen_packet(
                    self.id,
                    packet.source_id,
                    packet.source_id,
                    packet.channel_id,
                    "P".to_string()
                        + self.root_ids[packet.channel_id as usize]
                            .to_string()
                            .as_str(),
                );
                res.push(packet);
            }
        }
        // Packを受信
        else if packet.message.starts_with('P') {
            if !self.rec_info.contains_key(&packet.channel_id) {
                return (true, res);
            }

            let root_id = packet
                .message
                .chars()
                .skip(1)
                .collect::<String>()
                .parse::<u32>()
                .unwrap();

            if root_id == self.root_ids[packet.channel_id as usize] {
                return (true, res);
            }

            assert!(
                root_id < self.root_ids[packet.channel_id as usize],
                "root_id: {}, self.root_ids[packet.channel_id as usize]: {}",
                root_id,
                self.root_ids[packet.channel_id as usize]
            );

            self.root_ids[packet.channel_id as usize] = root_id; // ルートノードの更新

            self.parent_ids[packet.channel_id as usize] = packet.source_id; // 親ノードの更新

            let rec_info = self
                .rec_info
                .get(&packet.channel_id)
                .expect("error receiving Pack")
                .clone();

            // 自分が経路を発見したノードなら
            if rec_info.is_rec {
                let pset = self.gen_packet(
                    self.id,
                    packet.source_id,
                    packet.source_id,
                    packet.channel_id,
                    "pset".to_string(),
                );

                res.push(pset);
            }

            // 子たちに対してルートが変わったことを知らせる
            for &child_id in self
                .children_ids
                .get(&packet.channel_id)
                .cloned()
                .unwrap()
                .iter()
            {
                let rset = self.gen_packet(
                    root_id,
                    child_id,
                    child_id,
                    packet.channel_id,
                    "rset".to_string(),
                );
                res.push(rset);
            }

            if let Some(old_parent_id) = rec_info.old_parent_id {
                self.children_ids.insert(packet.channel_id, old_parent_id); // 子ノードの更新

                // 経路更新フェーズ
                // old_parent_idに対してPackを送信
                let packet = self.gen_packet(
                    self.id,
                    old_parent_id,
                    old_parent_id,
                    packet.channel_id,
                    "P".to_string() + root_id.to_string().as_str(),
                );
                res.push(packet);
            }

            // endフェーズ
            // self.rec_info.remove(&packet.channel_id);
        }
        // psetメッセージを受信した場合
        else if packet.message == "pset" {
            // 子に追加
            self.children_ids
                .insert(packet.channel_id, packet.source_id);

            if let Some(rec_info) = self.rec_info.get_mut(&packet.channel_id) {
                rec_info.children.insert(packet.source_id);
            }
        }
        // rsetメッセージを受信した場合
        else if packet.message == "rset" {
            // root_idを更新
            self.root_ids[packet.channel_id as usize] = packet.source_id;

            // 子に対してrsetを送信
            for &child_id in self
                .children_ids
                .get(&packet.channel_id)
                .cloned()
                .unwrap()
                .iter()
            {
                let rset = self.gen_packet(
                    packet.source_id,
                    child_id,
                    child_id,
                    packet.channel_id,
                    "rset".to_string(),
                );

                res.push(rset);
            }
        } else {
            return (false, res);
        }

        (true, res)
    }

    fn dynamic_rec_in_update(&mut self, option: Option<&UpdateOption>) {
        for (&channel_id, _) in self.rec_info.clone().iter() {
            let rec_info = self.rec_info.get_mut(&channel_id).unwrap();
            if rec_info.rec_cnt >= MAX_REC_CNT {
                rec_info.rec_cnt = 0;

                let init_node_id = rec_info.init_node_id;

                // 全ての子供の探索が終わったら
                if rec_info.children.is_empty() {
                    // failメッセージ
                    let packet = self.gen_packet(
                        self.id,
                        init_node_id,
                        init_node_id,
                        channel_id,
                        "fl".to_string(),
                    );
                    self.send_packet_buffer.push_back(packet);

                    continue;
                }

                let next_id = rec_info.children.iter().next().cloned().unwrap(); // emptyかは確認したのでunwrapして良い
                rec_info.children.remove(&next_id);
                rec_info.end();

                let packet = self.gen_packet(
                    self.id,
                    next_id,
                    next_id,
                    channel_id,
                    "tf".to_string(), // transfer
                );
                self.send_packet_buffer.push_back(packet);

                continue;
            }
            let mut rng = rand::thread_rng();
            let p: f64 = rng.gen();

            if p < 0.1 && rec_info.is_rec {
                rec_info.rec_cnt += 1;
                let init_node_id = rec_info.init_node_id;
                // 一定の確率で探索をブロードキャスト
                let new_packet = self.gen_packet(
                    self.id,
                    BROADCAST_ID,
                    BROADCAST_ID,
                    channel_id,
                    "R".to_string() + init_node_id.to_string().as_str(),
                );

                self.send_packet_buffer.push_back(new_packet);
            }
        }

        // システム分離があった時の処理
        if let Some(option) = option {
            // 全ての親を見ていき，optionに含まれていたらその親を削除
            // 動的再構成と同じように経路を探す
            // 経路が見つかったらルートノードのidバトルを行い敗者のツリーは勝者に取り込まれる
            // 勝者のツリーはルートノードの探索を続ける
            // ルートノードが見つからなかったら，自然と勝者のツリーのルートが全体のルートノードとなる
            // ただし再構成フラグのブロードキャストの不備によりIDバトルの勝敗が逆になってしまった場合でもそのまま続行する

            for &dc_node_id in option.vids.iter() {
                eprintln!("{} {} {:?}", self.id, dc_node_id, self.parent_ids);
                for index in 0..self.channel_num {
                    let parent_id = self.parent_ids[index as usize];

                    if parent_id != dc_node_id {
                        continue;
                    }

                    self.parent_ids[index as usize] = 0; // 親の削除

                    let rec_info = RecInfo::new(
                        index,
                        &self.children_ids.get(&index).unwrap(),
                        self.id,
                        None,
                    );

                    self.rec_info.insert(index, rec_info); // 再構成情報の追加

                    self.root_ids.insert(index as usize, self.id); // ルートノードの更新

                    // 再構成を開始
                    let children_ids = self.children_ids.get(&index).cloned().unwrap();

                    for &child_id in children_ids.iter() {
                        let packet =
                            self.gen_packet(self.id, child_id, child_id, index, "rec".to_string());
                        self.send_packet_buffer.push_back(packet);
                    }

                    if children_ids.is_empty() {
                        let new_packet = self.gen_packet(
                            self.id,
                            BROADCAST_ID,
                            BROADCAST_ID,
                            index,
                            "R".to_string() + self.id.to_string().as_str(),
                        );
                    }
                }

                // children_idsをfor文でみる
                for (channel_id, children) in self.children_ids.table.iter_mut() {
                    if children.contains(&dc_node_id) {
                        children.remove(&dc_node_id);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct RecInfo {
    channel_id: u8,
    is_rec: bool,
    rec_cnt: u8,
    children: HashSet<u32>,
    init_node_id: u32,
    old_parent_id: Option<u32>,
}
impl RecInfo {
    fn new(
        channel_id: u8,
        children: &HashSet<u32>,
        init_node_id: u32,
        old_parent_id: Option<u32>,
    ) -> Self {
        Self {
            channel_id,
            is_rec: false,
            rec_cnt: 0,
            children: children.clone(),
            init_node_id,
            old_parent_id,
        }
    }

    fn end(&mut self) {
        if self.is_rec {
            self.is_rec = false;
            self.rec_cnt = 0;
        } else {
        }
    }

    fn begin(&mut self) {
        if !self.is_rec {
            self.is_rec = true;
            self.rec_cnt = 0;
        } else {
            self.rec_cnt += 1;
        }
    }
}

#[derive(Clone, Debug)]
struct ChildrenTable {
    table: HashMap<u8, HashSet<u32>>,
}
impl ChildrenTable {
    fn new(channel_num: u8) -> Self {
        ChildrenTable {
            table: (0..channel_num)
                .map(|x| (x, HashSet::new()))
                .collect::<HashMap<u8, HashSet<u32>>>(),
        }
    }

    fn insert(&mut self, channel_id: u8, node_id: u32) {
        self.table.get_mut(&channel_id).unwrap().insert(node_id);
    }

    fn pop(&mut self, channel_id: u8) -> Option<u32> {
        // channel_id番目のHashSetから一つ取り出し，その要素を削除する
        let set = self.table.get_mut(&channel_id).unwrap();
        let res = set.iter().next().copied();

        if let Some(res) = res {
            set.remove(&res);
        }

        res
    }

    fn contains(&self, channel_id: u8, node_id: u32) -> bool {
        self.table.get(&channel_id).unwrap().contains(&node_id)
    }

    fn get(&self, channel_id: &u8) -> Option<&HashSet<u32>> {
        self.table.get(channel_id)
    }

    fn is_empty(&self, channel_id: &u8) -> bool {
        let set = self.table.get(channel_id);

        set.is_some() && set.unwrap().is_empty()
    }
}

fn print_packet(cur: u32, packet: &DynamicPacket) {
    let cur_id = get_pid(cur).unwrap();
    if packet.message.starts_with('R') || packet.message.starts_with('P') {
        let first = packet.message.chars().next().unwrap();
        let init_id = packet
            .message
            .chars()
            .skip(1)
            .collect::<String>()
            .parse::<u32>()
            .unwrap();
        recsimu_dbg!(
            "{}: {{source{},dest{},prev{},channel{},message:{}}}",
            cur_id,
            get_pid(packet.source_id).unwrap(),
            get_pid(packet.dest_id).unwrap(),
            get_pid(packet.prev_id).unwrap(),
            packet.channel_id,
            first.to_string() + get_pid(init_id).unwrap().as_str(),
        );
    } else {
        recsimu_dbg!(
            "{}: {{source{},dest{},prev{},channel{},message:{}}}",
            cur_id,
            get_pid(packet.source_id).unwrap(),
            get_pid(packet.dest_id).unwrap(),
            get_pid(packet.prev_id).unwrap(),
            packet.channel_id,
            packet.message
        );
    }
}
