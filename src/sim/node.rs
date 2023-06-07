use crate::hardware::flit::Flit;
use crate::hardware::Hardware;
use crate::network::Network;
use crate::sim::node_type::NodeType;

pub type NodeId = String;

pub struct Node {
    pub id: NodeId,
    pub node_type: NodeType,
    pub alive: bool,
    pub network: Network,
    pub hardware: Hardware,
}

impl Node {
    pub fn new(id: String, node_type: NodeType) -> Self {
        Self {
            id,
            node_type,
            alive: true,
            network: Network::new(),
            hardware: Hardware::new(),
        }
    }

    pub fn send_flit(&mut self) -> Result<Flit, Box<dyn std::error::Error>> {
        // フリットを送信する
        // ネットワーク層からフリットを受け取る
        let flit = self.network.send_flit().unwrap();
        // hardwareでフリットを送信する
        self.hardware.send_flit(flit)
    }

    pub fn update(&mut self, _cur_cycle: u32) {
        self.network.update();
        self.hardware.update();
    }

    pub fn receive_flit(&mut self, flit: &Flit) {
        // Data, Header Flitの場合はackを生成する
        // Ack Flitの場合はtransmission_bufferを更新する
        match flit {
            Flit::Data(_) | Flit::Header(_) => {
                self.hardware.ack_gen(flit);
            }
            Flit::Ack(_) => {
                self.hardware.receive_ack(flit);
            }
            _ => {
                panic!("receive_flit: flit is not header, data, or ack {flit:?}");
            }
        }
        self.network.receive_flit(flit);
    }
}
