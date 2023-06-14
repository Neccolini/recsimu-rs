use crate::hardware::flit::Flit;
use crate::hardware::state::State;
use crate::sim::node::{Node, NodeId};
use std::collections::HashMap;

pub struct Nodes {
    pub nodes: Vec<Node>,
    pub flit_buffers: HashMap<NodeId, Vec<Flit>>,
}

impl Nodes {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self {
            nodes,
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
            // todo 後でnodeに切り出す
            dbg!(&node.id, node.hardware.state.get());
            match node.hardware.state.get() {
                State::Sending => {
                    let flit = node.send_flit().unwrap();
                    // flit_buffersに追加
                    let buffer = self.flit_buffers.get_mut(&node.id).unwrap();
                    buffer.push(flit);
                }
                State::ReplyAck => {
                    let ack = node.send_ack().unwrap();
                    // flit_buffersに追加
                    let buffer = self.flit_buffers.get_mut(&node.id).unwrap();
                    buffer.push(ack);
                }
                _ => {}
            }
        }
        // バッファにあるメッセージを受信
        for node in self.nodes.iter_mut() {
            // todo nodeに切り出す
            let flits = self.flit_buffers.get(&node.id);
            if flits.is_none() {
                continue;
            }
            let flits = flits.unwrap();

            // 衝突がなければ受信
            if flits.len() == 1 {
                let flit = &flits[0];

                if let State::Idle | State::Waiting(_) = node.hardware.state.get() {
                    // 状態を受信中に変更
                    let _ = node.receive_flit(flit);
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
