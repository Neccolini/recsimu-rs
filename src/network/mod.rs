pub mod flit;
pub mod flit_buffer;
pub mod protocols;
pub mod vid;

use self::flit::{data_to_flits, flits_to_data};
use self::flit_buffer::FlitBuffer;
use self::protocols::packets::DefaultPacket;
use self::protocols::packets::InjectionPacket;
use self::protocols::NetworkProtocol;
use self::vid::*;
use crate::log::{
    get_packet_log, post_new_packet_log, update_packet_log, NewPacketLogInfo, UpdatePacketLogInfo,
};
use crate::network::flit::Flit;
use crate::network::protocols::packets::GeneralPacket;
use crate::sim::node_type::NodeType;
use std::collections::HashMap;

pub type ChannelId = u32;

pub struct Network {
    id: String,
    cur_cycle: u32,
    pub routing: NetworkProtocol,
    pub sending_flit_buffer: HashMap<ChannelId, FlitBuffer>,
    pub receiving_flit_buffer: HashMap<ChannelId, FlitBuffer>,
}

impl Network {
    pub fn new(id: String, vc_num: ChannelId, rf_kind: String, node_type: NodeType) -> Self {
        // sending_flit_bufferとreceiving_flit_bufferを初期化する
        // 0...vc_num-1のchannel_idを持つFlitBufferを生成する
        let mut sending_flit_buffer = HashMap::new();
        let mut receiving_flit_buffer = HashMap::new();

        for i in 0..vc_num {
            sending_flit_buffer.insert(i, FlitBuffer::new());
            receiving_flit_buffer.insert(i, FlitBuffer::new());
        }
        let routing = NetworkProtocol::new(rf_kind, node_type);
        let vid = routing.get_id();

        add_to_vid_table(vid, id.clone());

        Self {
            id,
            cur_cycle: 0,
            routing,
            sending_flit_buffer,
            receiving_flit_buffer,
        }
    }

    pub fn send_flit(&mut self, channel_id: ChannelId) -> Option<Flit> {
        // sending_flit_bufferのchannel_id番目のFlitBufferからpopする
        self.sending_flit_buffer.get_mut(&channel_id).unwrap().pop()
    }

    pub fn update(&mut self, cur_cycle: u32) {
        self.cur_cycle = cur_cycle;

        self.routing.update();

        // 送信待ちのパケットを取りに行く
        if let Some(packet) = self.routing.send_packet() {
            // packetをフリットに変換する
            let flits = data_to_flits(
                packet.data.clone(),
                packet.source_id.clone(),
                packet.dest_id.clone(),
                packet.next_id.clone(),
                packet.prev_id.clone(),
                packet.packet_id,
                packet.channel_id,
            );
            self.log_handler(&packet);

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
        if let Flit::Ack(_) = flit {
            // ack flitは受け取らない
            return;
        }

        // receiving_flit_bufferのchannel_id番目のFlitBufferにpushする
        self.receiving_flit_buffer
            .get_mut(&channel_id)
            .unwrap()
            .push(flit.clone());

        if let Flit::Tail(tail_flit) = flit {
            if tail_flit.dest_id == "broadcast"
                || get_vid(tail_flit.dest_id.clone()).unwrap() == self.routing.get_id()
            {
                // receiving_flit_bufferのchannel_id番目のFlitBufferからtail_flit.flit_num個のフリットを取り出す
                let mut flits = Vec::new();
                for _ in 0..tail_flit.flit_num {
                    // todo 複数のdataフリットからなる場合，順番が正しく並べられるかを確かめるべき
                    if let Some(flit) = self
                        .receiving_flit_buffer
                        .get_mut(&channel_id)
                        .unwrap()
                        .pop()
                    {
                        flits.push(flit);
                    }
                }

                let data = flits_to_data(&flits);

                let packet = GeneralPacket {
                    data,
                    source_id: tail_flit.source_id.clone(),
                    dest_id: tail_flit.dest_id.clone(),
                    next_id: tail_flit.next_id.clone(),
                    prev_id: tail_flit.prev_id.clone(),
                    packet_id: tail_flit.packet_id,
                    channel_id: tail_flit.channel_id,
                };

                self.routing.receive_packet(&packet);

                self.log_handler(&packet);
            }
        }
    }

    pub fn send_new_packet(&mut self, packet: &InjectionPacket) {
        self.routing.push_new_packet(packet);
    }
}

impl Network {
    fn log_handler(&self, packet: &GeneralPacket) {
        let packet_id = packet.source_id.to_string() + "_" + &packet.packet_id.to_string();

        if get_packet_log(&packet_id).is_none() {
            let log = NewPacketLogInfo {
                packet_id,
                from_id: packet.source_id.clone(),
                dest_id: packet.dest_id.clone(),
                send_cycle: self.cur_cycle,
                message: self.get_message(packet),
            };

            let _ = post_new_packet_log(log);
        } else {
            let update_log = UpdatePacketLogInfo {
                last_receive_cycle: Some(self.cur_cycle),
                route_info: Some(self.id.clone()),
                is_delivered: Some(true),
                flit_log: None,
            };

            let _ = update_packet_log(packet_id, update_log);
        }
    }

    fn get_message(&self, packet: &GeneralPacket) -> String {
        match self.routing {
            NetworkProtocol::DefaultFunction(_) => {
                let p = DefaultPacket::from_general(packet);
                p.message
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::flit::HeaderFlit;
    use crate::network::protocols::packets::InjectionPacket;

    #[test]
    fn test_send_flit() {
        add_to_vid_table(u32::MAX, "broadcast".to_string());
        let mut network = Network::new(
            "test".to_string(),
            1,
            "default".to_string(),
            NodeType::Router,
        );
        let packet = InjectionPacket {
            message: "hello world".to_string(),
            dest_id: "broadcast".to_string(),
            source_id: "test".to_string(),
        };
        network.send_new_packet(&packet);
        network.update(0);

        // preq
        network.send_flit(0).unwrap();
        network.send_flit(0).unwrap();

        network.update(1);
        let flit = network.send_flit(0).unwrap();
        assert_eq!(
            flit,
            Flit::Header(HeaderFlit {
                source_id: "test".to_string(),
                dest_id: "broadcast".to_string(),
                packet_id: 0,
                next_id: "broadcast".to_string(),
                prev_id: "test".to_string(),
                flits_len: 2,
                channel_id: 0,
            })
        );

        network.update(2);
        let flit = network.send_flit(0).unwrap();
        match flit {
            Flit::Tail(tail_flit) => {
                assert_eq!(tail_flit.dest_id, "broadcast".to_string());
                assert_eq!(tail_flit.source_id, "test".to_string());
                assert_eq!(tail_flit.packet_id, 0);
                assert_eq!(tail_flit.next_id, "broadcast".to_string());
                assert_eq!(tail_flit.flit_num, 2);
                assert_eq!(tail_flit.channel_id, 0);
            }
            _ => panic!(),
        }
    }
}
