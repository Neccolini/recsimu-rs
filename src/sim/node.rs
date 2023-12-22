use std::collections::HashMap;

use crate::hardware::state::State;
use crate::hardware::switching::Switching;
use crate::hardware::Hardware;
use crate::network::core_functions::packets::InjectionPacket;
use crate::network::flit::Flit;
use crate::network::Network;
use crate::sim::node_type::NodeType;

pub type NodeId = String;
pub type CycleNum = u32;

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub network: Network,
    pub hardware: Hardware,
    pub packets: HashMap<CycleNum, InjectionPacket>,
    cur_cycle: u32,
}

impl Node {
    pub fn new(
        id: &str,
        vc_num: u8,
        switching: &Switching,
        rf_kind: &str,
        node_type: &NodeType,
        packets: &HashMap<CycleNum, InjectionPacket>,
    ) -> Self {
        Self {
            id: id.to_string(),
            node_type: node_type.clone(),
            network: Network::new(id, vc_num, switching, rf_kind, node_type),
            hardware: Hardware::new(id, switching),
            packets: packets.clone(),
            cur_cycle: 0,
        }
    }

    pub fn send_flit(&mut self) -> Result<Flit, Box<dyn std::error::Error>> {
        // retransmission_bufferから取り出し，送信する
        let x = self.hardware.retransmission_buffer.clone();
        if x.is_empty() {
            return Err("retransmission_buffer is empty".into());
        }

        if x.is_broadcast() {
            self.hardware.retransmission_buffer.clear();
        }
        Ok(x)
    }

    pub fn send_ack(&mut self) -> Result<Flit, Box<dyn std::error::Error>> {
        let ack = self.hardware.send_ack().map_err(|e| {
            dbg!("error occured while sending a flit: {e:?}");
            e
        })?;

        Ok(ack)
    }

    pub fn update(&mut self, cur_cycle: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.cur_cycle = cur_cycle;

        // packetsにcur_cycleが含まれていたら
        if let Some(packet) = self.packets.get(&cur_cycle) {
            // 新規パケットを生成
            self.network.send_new_packet(packet);
        }

        self.network.update(cur_cycle);

        // retransmission_bufferが空なら
        if self.hardware.retransmission_buffer.is_empty() {
            // network.send_flit_bufferからフリットを取り出す
            if let Some(flit) = self.network.send_flit() {
                self.hardware.send_flit(&flit).map_err(|e| {
                    dbg!("error occured while sending a flit: {e:?}");
                    e
                })?;
            }
        }

        // ハードウェアの状態を更新
        self.hardware.update_state()?;

        Ok(())
    }

    pub fn receive_flit(&mut self, flit: &Flit) -> Result<(), Box<dyn std::error::Error>> {
        self.hardware.set_state(&State::Receiving);

        if let Some(flit) = self.hardware.receive_flit(flit)? {
            self.network.receive_flit(&flit, 0);
        }
        Ok(())
    }

    pub fn shutdown(&mut self) {}

    pub fn reboot(&mut self) {}
}
