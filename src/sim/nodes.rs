use crate::hardware::state::State;
use crate::network::flit::Flit;
use crate::sim::node::{Node, NodeId};
use std::collections::HashMap;

pub struct Nodes {
    pub nodes: Vec<Node>,
    pub flit_buffers: HashMap<NodeId, Vec<Flit>>,
    // ノードの隣接情報を保持するHashMap
    pub neighbors: HashMap<NodeId, Vec<NodeId>>,
}

impl Nodes {
    pub fn new(nodes: Vec<Node>, neighbors: HashMap<NodeId, Vec<NodeId>>) -> Self {
        Self {
            nodes,
            flit_buffers: HashMap::new(),
            neighbors,
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
                    if let Some(receiver_id) = flit.get_next_id() {
                        // broadcastの場合はneighborsを見て，nodeに隣接するノードすべてに配信する
                        if receiver_id == "broadcast" {
                            let neighbors = self.neighbors.get(&node.id).unwrap();
                            neighbors.iter().for_each(|neighbor| {
                                let buffer = self
                                    .flit_buffers
                                    .entry(neighbor.clone())
                                    .or_insert(Vec::new());
                                buffer.push(flit.clone());
                            });
                        } else if let Some(neighbor_list) = self.neighbors.get(&node.id) {
                            if neighbor_list.contains(&receiver_id) {
                                let buffer = self
                                    .flit_buffers
                                    .entry(receiver_id.clone())
                                    .or_insert(Vec::new());
                                buffer.push(flit);
                            }
                        }
                    }
                }
                State::ReplyAck => {
                    let ack = node.send_ack().unwrap();
                    // flit_buffersに追加
                    if let Some(receiver_id) = ack.get_next_id() {
                        let buffer = self.flit_buffers.entry(receiver_id).or_insert(Vec::new());
                        buffer.push(ack);
                    }
                }
                _ => {}
            }
        }
        dbg!("flit_buffers: {:?}", &self.flit_buffers);
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
            let _ = node.update(cur_cycle).map_err(|e| {
                panic!(
                    "node: {}, cur_cycle: {} update error: {:?}",
                    node.id, e, cur_cycle
                );
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::protocols::packets::GeneralPacket;
    use crate::sim::NodeType;
    #[test]
    fn test_run_cycle() {
        // node1からnode2へのパケットを作成
        let packet = GeneralPacket {
            source_id: "node1".to_string(),
            dest_id: "broadcast".to_string(),
            message: "hello".to_string(),
            packet_id: 0,
        };
        let mut packets = HashMap::new();
        packets.insert(0, packet);

        let mut neighbors = HashMap::new();
        neighbors.insert("node1".to_string(), vec!["node2".to_string()]);
        neighbors.insert("node2".to_string(), vec!["node1".to_string()]);

        let mut nodes = Nodes::new(
            vec![
                Node::new("node1".to_string(), NodeType::Coordinator, 1, packets),
                Node::new("node2".to_string(), NodeType::Router, 1, HashMap::new()),
            ],
            neighbors,
        );

        nodes.run_cycle(0);

        // nodeの状態を見る
        assert_eq!(*nodes.nodes[0].hardware.state.get(), State::Sending);
        assert_eq!(*nodes.nodes[1].hardware.state.get(), State::Receiving);

        nodes.run_cycle(1);

        nodes.run_cycle(2);
    }
}
