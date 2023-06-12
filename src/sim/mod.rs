pub mod node;
pub(crate) mod node_type;
pub mod nodes;

use crate::file::InputFile;
use crate::log::Log;
use crate::network::protocols::packets::GeneralPacket;
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
                            GeneralPacket {
                                source_id: packet.src_id.clone(),
                                dest_id: packet.dest_id.clone(),
                                message: packet.msg.clone(),
                            },
                        )
                    })
                    .collect::<HashMap<u32, GeneralPacket>>();

                Node::new(
                    node.node_id.clone(),
                    NodeType::new(&node.node_type),
                    packets,
                )
            })
            .collect();

        Ok(Sim {
            node_num: input.node_num,
            debug: self.verbose,
            nodes: Nodes::new(nodes),
            total_cycles: 0,
            log: Log::new(),
            cur_cycles: 0,
        })
    }
}

pub struct Sim {
    pub node_num: u32,
    pub total_cycles: u32,
    pub cur_cycles: u32,
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
