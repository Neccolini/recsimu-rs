pub mod node;
pub(crate) mod node_type;
pub mod nodes;

use crate::file::InputFile;
use crate::log::Log;
use crate::network::protocols::packets::InjectionPacket;
use std::collections::HashMap;
use std::{error, path::PathBuf};

use self::node::Node;
use self::node_type::NodeType;
use self::nodes::Nodes;

pub struct SimBuilder {
    pub path: PathBuf,
    pub verbose: bool,
}

impl SimBuilder {
    pub fn new(path: PathBuf, verbose: bool) -> Self {
        Self { path, verbose }
    }
    pub fn build(&self) -> Result<Sim, Box<dyn error::Error>> {
        let input = InputFile::new(self.path.clone());

        let nodes: Vec<Node> = input
            .nodes
            .iter()
            .map(|node| {
                let packets = input
                    .packets
                    .iter()
                    .filter(|packet| packet.src_id == node.node_id)
                    .map(|packet| {
                        (
                            packet.cycle_num,
                            InjectionPacket {
                                source_id: packet.src_id.clone(),
                                dest_id: packet.dest_id.clone(),
                                message: packet.msg.clone(),
                            },
                        )
                    })
                    .collect::<HashMap<u32, InjectionPacket>>();

                Node::new(
                    node.node_id.clone(),
                    input.channel_num,
                    "default".to_string(),
                    NodeType::new(&node.node_type),
                    packets,
                )
            })
            .collect();

        Ok(Sim {
            node_num: input.node_num,
            debug: self.verbose,
            nodes: Nodes::new(nodes, input.neighbors),
            total_cycles: input.total_cycles,
            vc_num: input.channel_num,
            log: Log::new(),
            cur_cycles: 0,
        })
    }
}

pub struct Sim {
    pub node_num: u32,
    pub total_cycles: u32,
    pub cur_cycles: u32,
    pub vc_num: u32,
    pub nodes: Nodes,
    pub debug: bool,
    pub log: Log,
}

impl Sim {
    pub fn run(&mut self) {
        // シミュレーションを実行する
        while self.cur_cycles < self.total_cycles {
            self.nodes.run_cycle(self.cur_cycles);
            self.cur_cycles += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_build() {
        let path = PathBuf::from("tests/run/success1.json");
        let sim_builder = SimBuilder::new(path, false);
        let sim = sim_builder.build().unwrap();

        assert_eq!(sim.node_num, 2);
        assert_eq!(sim.total_cycles, 10);
        assert_eq!(sim.vc_num, 1);
        assert_eq!(sim.cur_cycles, 0);
        assert_eq!(sim.debug, false);
        assert_eq!(sim.nodes.nodes.len(), 2);
        assert_eq!(sim.nodes.nodes[0].id, "node1");
        assert_eq!(sim.nodes.nodes[0].node_type, NodeType::Coordinator);
        assert_eq!(sim.nodes.nodes[0].packets.len(), 1);
        assert_eq!(sim.nodes.nodes[0].packets[&1].source_id, "node1");
        assert_eq!(sim.nodes.nodes[0].packets[&1].dest_id, "node2");
        assert_eq!(sim.nodes.nodes[0].packets[&1].message, "Hello, World!");
        assert_eq!(sim.nodes.nodes[1].id, "node2");
        assert_eq!(sim.nodes.nodes[1].node_type, NodeType::Router);
        assert_eq!(sim.nodes.nodes[1].packets.len(), 0);
    }
}
