use crate::hardware::flit::Flit;
use crate::hardware::state::State;
use crate::sim::node::{Node, NodeId};
use std::collections::HashMap;

pub struct Nodes {
    nodes: Vec<Node>,
    flit_buffers: HashMap<NodeId, Vec<Flit>>,
}

impl Nodes {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            flit_buffers: HashMap::new(),
        }
    }
    pub fn run_cycle(&mut self, cur_cycle: u32) {
        // 各ノードの状態を更新する
        self.update_nodes(cur_cycle);

        // 各ノードのメッセージを処理する
        self.message_handle(cur_cycle);
    }
    fn message_handle(&mut self, _cur_cycle: u32) {
        // 送信状態のノードはflitを送信
        for node in self.nodes.iter_mut() {
            if let State::Sending | State::ReplyAck = node.hardware.state {
                let flit = node.send_flit().unwrap();
                // flit_buffersに追加
                let buffer = self.flit_buffers.get_mut(&node.id).unwrap();
                buffer.push(flit);
            }
        }
        // バッファにあるメッセージを受信
        for node in self.nodes.iter_mut() {
            // バッファにあるメッセージを受信
            let flits = self.flit_buffers.get(&node.id).unwrap();
            // バッファのメッセージが衝突したら、衝突したノードのメッセージを破棄
            if flits.len() == 1 {
                let flit = &flits[0];

                if let State::Idle(_) | State::Receiving | State::Waiting = node.hardware.state {
                    // 受信成功
                    // メッセージを受信
                    node.receive_flit(flit);
                }
            }
        }
        // flit_buffersをクリア
        self.flit_buffers.clear();
    }
    fn update_nodes(&mut self, cur_cycle: u32) {
        // 各ノードの状態を更新する
        for node in self.nodes.iter_mut() {
            node.update(cur_cycle);
        }
    }
}

impl Default for Nodes {
    fn default() -> Self {
        Self::new()
    }
}
