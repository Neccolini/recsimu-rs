pub mod flit;
pub mod flit_buffer;
pub mod protocols;

use self::flit::flits_to_data;
use self::flit_buffer::FlitBuffer;
use self::protocols::packets::encode_id;
use self::protocols::NetworkProtocol;
use crate::network::flit::Flit;
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
                let channel_id = flit.get_channel_id().unwrap();

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

        // 自分 or broadcast宛かつ最後のフリットなら

        if let Flit::Tail(tail_flit) = flit {
            if tail_flit.dest_id == self.routing.get_id() || tail_flit.dest_id == "broadcast" {
                // receiving_flit_bufferのchannel_id番目のFlitBufferからtail_flit.flit_num個のフリットを取り出す
                let mut flits = Vec::new();
                for _ in 1..tail_flit.flit_num {
                    flits.push(
                        self.receiving_flit_buffer
                            .get_mut(&channel_id)
                            .unwrap()
                            .pop()
                            .unwrap(),
                    );
                }
                // パケットをデコードする
                let data = flits_to_data(&flits);

                // dataを文字列にデコードする
                let message = String::from_utf8(data).unwrap(); // todo エラー処理

                let packet = GeneralPacket {
                    message,
                    source_id: tail_flit.source_id.clone(),
                    dest_id: tail_flit.dest_id.clone(),
                    packet_id: encode_id(tail_flit.packet_id, &tail_flit.source_id),
                };

                // パケットを受信したことを通知する
                self.routing.receive_packet(&packet);
            }
        }
    }

    pub fn send_new_packet(&mut self, packet: &GeneralPacket) {
        self.routing.push_new_packet(packet);
    }
}

#[cfg(test)]
mod tests {
    use crate::network::flit::{HeaderFlit, TailFlit};

    use super::*;

    #[test]
    fn test_send_flit() {
        let mut network = Network::new(1);
        let packet = GeneralPacket {
            message: "".to_string(),
            source_id: "test".to_string(),
            dest_id: "broadcast".to_string(),
            packet_id: "test_0".to_string(),
        };
        network.send_new_packet(&packet);
        network.update();
        let flit = network.send_flit(0).unwrap();

        assert_eq!(
            flit,
            Flit::Header(HeaderFlit {
                source_id: "test".to_string(),
                dest_id: "broadcast".to_string(),
                packet_id: 0,
                next_id: "broadcast".to_string(),
                flits_len: 2,
                channel_id: 0,
            })
        );
        let flit = network.send_flit(0).unwrap();
        assert_eq!(
            flit,
            Flit::Tail(TailFlit {
                source_id: "test".to_string(),
                dest_id: "broadcast".to_string(),
                next_id: "broadcast".to_string(),
                data: vec![
                    0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 98, 114, 111, 97, 100, 99, 97,
                    115, 116, 4, 0, 0, 0, 0, 0, 0, 0, 116, 101, 115, 116, 0, 0, 0, 0
                ],
                resend_num: 0,
                packet_id: 0,
                flit_num: 2,
                channel_id: 0,
            })
        );
    }
}
