pub mod node;
pub(crate) mod node_type;
pub mod nodes;
pub mod simulation;


use crate::log::Log;

use std::{error, path::PathBuf};

use self::nodes::Nodes;

pub struct SimBuilder {
    pub path: PathBuf,
}

impl SimBuilder {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn build(&self) -> Result<Sim, Box<dyn error::Error>> {
        Ok(Sim {
            node_num: 0,
            debug: false,
            nodes: Nodes::new(),
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
