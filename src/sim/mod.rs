pub mod node;
pub(crate) mod node_type;
pub mod nodes;

use crate::file::InputFile;
use crate::hardware::switching::Switching;
use crate::log::{aggregate_log, get_all_log};
use crate::network::core_functions::packets::InjectionPacket;
use crate::recsimu_dbg;
use std::collections::HashMap;
use std::{error, path::Path, path::PathBuf};

use self::node::Node;
use self::node_type::NodeType;
use self::nodes::Nodes;

use crate::network::vid::add_to_vid_table;

pub struct SimBuilder {
    pub path: PathBuf,
}

impl SimBuilder {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
    pub fn build(&self) -> Result<Sim, Box<dyn error::Error>> {
        let input = InputFile::new(self.path.clone());

        let switching = input.switching.parse::<Switching>()?;
        let routing = input.routing.unwrap_or("default".to_string());

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
                    &node.node_id,
                    input.channel_num,
                    &switching.clone(),
                    &routing,
                    &NodeType::new(&node.node_type),
                    &packets,
                )
            })
            .collect();

        add_to_vid_table(u32::MAX, "broadcast");
        // print_vid_table();
        Ok(Sim {
            node_num: input.node_num,
            nodes: Nodes::new(&nodes, &input.neighbors),
            total_cycles: input.total_cycles,
            channel_num: input.channel_num,
            cur_cycles: 0,
        })
    }
}

pub struct Sim {
    pub node_num: u32,
    pub total_cycles: u32,
    pub cur_cycles: u32,
    pub channel_num: u8,
    pub nodes: Nodes,
}

impl Sim {
    pub fn run(&mut self) {
        // シミュレーションを実行する
        while self.cur_cycles < self.total_cycles {
            recsimu_dbg!("cycle: {}", self.cur_cycles);
            self.nodes.run_cycle(self.cur_cycles);
            self.cur_cycles += 1;
        }

        get_all_log().iter().for_each(|log| {
            recsimu_dbg!("{:?}", log);
        });

        println!("{:?}", aggregate_log());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_build() {
        let path = PathBuf::from("tests/run/auto/1_c.json");
        let sim_builder = SimBuilder::new(&path);
        let sim = sim_builder.build().unwrap();

        assert_eq!(sim.node_num, 2);
        assert_eq!(sim.total_cycles, 50);
        assert_eq!(sim.channel_num, 1);
        assert_eq!(sim.cur_cycles, 0);
        assert_eq!(sim.nodes.nodes.len(), 2);
        assert_eq!(sim.nodes.nodes[0].id, "node1");
        assert_eq!(sim.nodes.nodes[0].node_type, NodeType::Coordinator);
        assert_eq!(sim.nodes.nodes[0].packets.len(), 1);
        assert_eq!(sim.nodes.nodes[0].packets[&40].source_id, "node1");
        assert_eq!(sim.nodes.nodes[0].packets[&40].dest_id, "node2");
        assert_eq!(sim.nodes.nodes[0].packets[&40].message, "Hello, World!");
        assert_eq!(sim.nodes.nodes[1].id, "node2");
        assert_eq!(sim.nodes.nodes[1].node_type, NodeType::Router);
        assert_eq!(sim.nodes.nodes[1].packets.len(), 0);
    }
}
