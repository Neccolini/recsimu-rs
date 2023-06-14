pub mod flit_buffer;
pub mod protocols;

use self::flit_buffer::FlitBuffer;
use self::protocols::NetworkProtocol;
use crate::hardware::flit::Flit;
use crate::network::protocols::packets::GeneralPacket;
use std::collections::HashMap;

pub type ChannelId = u32;

pub struct Network {
    pub routing: NetworkProtocol,
    pub sending_flit_buffer: HashMap<ChannelId, FlitBuffer>,
    pub receiving_flit_buffer: HashMap<ChannelId, FlitBuffer>,
}

impl Network {
    pub fn new(vc_num: ChannelId) -> Self {
        // sending_flit_bufferとreceiving_flit_bufferを初期化する
        // 0...vc_num-1のchannel_idを持つFlitBufferを生成する
        let mut sending_flit_buffer = HashMap::new();
        let mut receiving_flit_buffer = HashMap::new();

        for i in 0..vc_num {
            sending_flit_buffer.insert(i, FlitBuffer::new());
            receiving_flit_buffer.insert(i, FlitBuffer::new());
        }

        Self {
            routing: NetworkProtocol::default(),
            sending_flit_buffer,
            receiving_flit_buffer,
        }
    }

    pub fn send_flit(&mut self, channel_id: ChannelId) -> Option<Flit> {
        // sending_flit_bufferのchannel_id番目のFlitBufferからpopする
        self.sending_flit_buffer.get_mut(&channel_id).unwrap().pop()
    }

    pub fn update(&mut self) {
        // 送信待ちのパケットを取りに行く
        if let Some(flits) = self.routing.send_packet() {
            // 送信待ちのパケットがあったら
            // sending_flit_bufferのchannel_id番目のFlitBufferにpushする
            for flit in flits {
                let channel_id = match flit.clone() {
                    Flit::Header(head) => head.channel_id,
                    Flit::Data(data) => data.channel_id,
                    Flit::Ack(ack) => ack.channel_id,
                    Flit::Empty => panic!("Empty flit is not allowed to be sent"),
                };

                self.sending_flit_buffer
                    .get_mut(&channel_id)
                    .unwrap()
                    .push(flit);
            }
        }
    }

    pub fn receive_flit(&mut self, flit: &Flit, channel_id: ChannelId) {
        // receiving_flit_bufferのchannel_id番目のFlitBufferにpushする
        self.receiving_flit_buffer
            .get_mut(&channel_id)
            .unwrap()
            .push(flit.clone());
    }

    pub fn send_new_packet(&mut self, packet: &GeneralPacket) {
        self.routing.push_new_packet(packet);
    }
}
