use std::collections::HashMap;

use crate::hardware::state::State;
use crate::hardware::Hardware;
use crate::network::flit::Flit;
use crate::network::protocols::packets::InjectionPacket;
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
    pub packets: HashMap<CycleNum, InjectionPacket>,
    cur_cycle: u32,
}

impl Node {
    pub fn new(
        id: String,
        vc_num: u32,
        rf_kind: String,
        node_type: NodeType,
        packets: HashMap<CycleNum, InjectionPacket>,
    ) -> Self {
        Self {
            id: id.clone(),
            node_type: node_type.clone(),
            alive: true,
            network: Network::new(id, vc_num, rf_kind, node_type),
            hardware: Hardware::new(),
            packets,
            cur_cycle: 0,
        }
    }

    pub fn send_flit(&mut self) -> Result<Flit, Box<dyn std::error::Error>> {
        // フリットを送信する
        // ネットワーク層からフリットを受け取る
        // flitがNoneならOkを返す
        if let Some(flit) = self.network.send_flit(0) {
            self.hardware.send_flit(&flit).map_err(|e| {
                dbg!("error occured while sending a flit: {e:?}");
                e
            })?;
            return Ok(flit);
        }

        Ok(Flit::Empty)
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

        self.network.update();

        // retransmission_bufferが空なら
        if self.hardware.retransmission_buffer.is_empty() {
            // 仮想channelを選択
            let channel = self.select_vc();
            // network.send_flit_bufferからフリットを取り出す
            if let Some(flit_buffer) = self.network.sending_flit_buffer.get_mut(&channel) {
                //flit_bufferからフリットを取り出し，送信
                let flit = flit_buffer.pop();
                if flit.is_some() {
                    self.hardware.send_flit(&flit.unwrap())?;
                }
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
}

impl Node {
    fn select_vc(&self) -> u32 {
        0 // todo 複数チャネルに対応
    }
}
