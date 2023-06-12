use std::collections::HashMap;

use crate::hardware::flit::Flit;
use crate::hardware::Hardware;
use crate::hardware::state::State;
use crate::network::protocols::packets::GeneralPacket;
use crate::network::Network;
use crate::sim::node_type::NodeType;

pub type NodeId = String;
pub type CycleNum = u32;
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub alive: bool,
    pub network: Network,
    pub hardware: Hardware,
    pub packets: HashMap<CycleNum, GeneralPacket>,
}

impl Node {
    pub fn new(id: String, node_type: NodeType, packets: HashMap<CycleNum, GeneralPacket>) -> Self {
        Self {
            id,
            node_type,
            alive: true,
            network: Network::new(),
            hardware: Hardware::new(),
            packets,
        }
    }

    pub fn send_flit(&mut self) -> Result<Flit, Box<dyn std::error::Error>> {
        // フリットを送信する
        // ネットワーク層からフリットを受け取る
        let flit = self.network.send_flit(0).unwrap(); // todo error handling

        self.hardware.send_flit(&flit).map_err(|e| {
            dbg!("error occured while sending a flit: {e:?}");
            e
        })?;

        Ok(flit)
    }

    pub fn send_ack(&mut self) -> Result<Flit, Box<dyn std::error::Error>> {
        let ack = self.hardware.send_ack().map_err(|e| {
            dbg!("error occured while sending a flit: {e:?}");
            e
        })?;

        Ok(ack)
    }

    pub fn update(&mut self, cur_cycle: u32) {
        // packetsにcur_cycleが含まれていたら
        if let Some(packet) = self.packets.get(&cur_cycle) {
            // 新規パケットを生成
            self.network.send_new_packet(packet);
        }

        self.network.update();
    }

    pub fn receive_flit(&mut self, flit: &Flit) -> Result<(), Box<dyn std::error::Error>> {
        self.hardware.set_state(&State::Receiving);

        if let Some(flit) = self.hardware.receive_flit(flit)? {
            self.network.receive_flit(&flit, 0);
        }
        Ok(())
    }
}
