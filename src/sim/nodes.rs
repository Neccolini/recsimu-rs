use crate::hardware::state::State;
use crate::log::{post_collision_info, NewCollisionInfo};
use crate::network::flit::Flit;
use crate::sim::node::{Node, NodeId};
use crate::sim::rec::RecTable;
use std::collections::HashMap;
pub struct Nodes {
    pub nodes: Vec<Node>,
    pub flit_buffers: HashMap<NodeId, Vec<Flit>>,
    // ノードの隣接情報を保持するHashMap
    pub neighbors: HashMap<NodeId, Vec<NodeId>>,
    pub rec_table: RecTable,
}

impl Nodes {
    pub fn new(
        nodes: &[Node],
        neighbors: &HashMap<NodeId, Vec<NodeId>>,
        rec_table: &RecTable,
    ) -> Self {
        Self {
            nodes: nodes.to_owned(),
            flit_buffers: HashMap::new(),
            neighbors: neighbors.clone(),
            rec_table: rec_table.clone(),
        }
    }

    pub fn run_cycle(&mut self, cur_cycle: u32) {
        // 各ノードの状態を更新する
        self.update_nodes(cur_cycle);

        // 各ノードのメッセージを処理する
        self.message_handle(cur_cycle);
    }

    fn message_handle(&mut self, cur_cycle: u32) {
        // 送信状態のノードはflitを送信
        for node in self.nodes.iter_mut() {
            // todo 後でnodeに切り出す
            match node.hardware.state.get() {
                State::Sending => {
                    let flit = node.send_flit().unwrap();
                    // flit_buffersに追加
                    if let Some(receiver_id) = flit.get_next_id() {
                        // broadcastの場合はneighborsを見て，nodeに隣接するノードすべてに配信する
                        if receiver_id == "broadcast" {
                            let neighbors = self.neighbors.get(&node.id).unwrap();
                            neighbors.iter().for_each(|neighbor| {
                                let buffer = self.flit_buffers.entry(neighbor.clone()).or_default();
                                buffer.push(flit.clone());
                            });
                        } else if let Some(neighbor_list) = self.neighbors.get(&node.id) {
                            if neighbor_list.contains(&receiver_id) {
                                let buffer =
                                    self.flit_buffers.entry(receiver_id.clone()).or_default();
                                buffer.push(flit);
                            }
                        }
                    }
                }
                State::ReplyAck => {
                    let ack = node.send_ack().unwrap();
                    if ack.is_ack() {
                        // flit_buffersに追加
                        if let Some(receiver_id) = ack.get_dest_id() {
                            let buffer = self.flit_buffers.entry(receiver_id).or_default();
                            buffer.push(ack);
                        }
                    }
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
                    let _ = node.receive_flit(flit).map_err(|e| {
                        panic!(
                            "node: {}, cur_cycle: {} receive_flit error: {:?}",
                            node.id, cur_cycle, e
                        );
                    });
                }
            } else if flits.len() >= 2 {
                post_collision_info(&NewCollisionInfo {
                    cycle: cur_cycle,
                    from_ids: flits.iter().map(|f| f.get_prev_id().unwrap()).collect(),
                    dest_id: node.id.clone(),
                });
            }
        }

        // flit_buffersをクリア
        self.flit_buffers.clear();
    }
    fn update_nodes(&mut self, cur_cycle: u32) {
        // update system topology
        if let Some(update) = self.rec_table.clone().table.get(&cur_cycle) {
            self.update_system(&update.new_neighbors);
        }

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

    fn update_system(&mut self, new_neighbors: &HashMap<String, Vec<String>>) {
        self.neighbors = new_neighbors.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::switching::Switching;
    use crate::network::core_functions::packets::InjectionPacket;
    use crate::network::vid::add_to_vid_table;
    use crate::sim::NodeType;

    #[test]
    fn test_run_cycle() {
        // node1からnode2へのパケットを作成
        add_to_vid_table(u32::MAX, "broadcast");

        let packets: HashMap<u32, InjectionPacket> = HashMap::new();

        let mut neighbors = HashMap::new();
        neighbors.insert("node1".to_string(), vec!["node2".to_string()]);
        neighbors.insert("node2".to_string(), vec!["node1".to_string()]);

        let rec_table = RecTable {
            table: HashMap::new(),
        };

        let mut nodes = Nodes::new(
            &vec![
                Node::new(
                    "node1",
                    1,
                    &Switching::StoreAndForward,
                    "default",
                    &NodeType::Coordinator,
                    &packets,
                ),
                Node::new(
                    "node2",
                    1,
                    &Switching::StoreAndForward,
                    "default",
                    &NodeType::Router,
                    &HashMap::new(),
                ),
            ],
            &neighbors,
            &rec_table,
        );

        nodes.run_cycle(0);

        // nodeの状態を見る
        assert_eq!(*nodes.nodes[0].hardware.state.get(), State::Receiving);
        assert_eq!(*nodes.nodes[1].hardware.state.get(), State::Sending);

        nodes.run_cycle(1);

        // nodes.run_cycle(2);
    }
}
