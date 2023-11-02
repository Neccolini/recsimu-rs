pub mod flit;
pub mod flit_buffer;
pub mod protocols;
pub mod vid;

use self::flit::data_to_flits;
use self::flit_buffer::{FlitBuffer, ReceivedFlitsBuffer};
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
    received_flits_buffer: ReceivedFlitsBuffer,
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
            received_flits_buffer: ReceivedFlitsBuffer::new(),
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
        // ルーティングするフリットの処理
        let channel_ids: Vec<ChannelId> = self.receiving_flit_buffer.keys().cloned().collect();
        // receiving_flit_bufferからsending_flit_bufferへフリットを転送する
        for channel_id in channel_ids {
            self.forward_flits(channel_id);
        }
    }

    pub fn receive_flit(&mut self, flit: &Flit, channel_id: ChannelId) {
        if let Flit::Ack(_) = flit {
            // ack flitは受け取らない
            return;
        }

        // 自分が最終的な宛先なら
        if flit.get_dest_id().unwrap() == self.id || flit.get_dest_id().unwrap() == "broadcast" {
            // received_flits_bufferにpushする
            self.received_flits_buffer.push_flit(flit);

            if flit.is_tail() || (flit.is_header() && flit.get_flits_len().unwrap_or(0) == 1) {
                let packet = self
                    .received_flits_buffer
                    .pop_packet(
                        &flit.get_source_id().unwrap(),
                        flit.get_packet_id().unwrap(),
                    )
                    .unwrap();

                self.routing.receive_packet(&packet);

                self.log_handler(&packet);
            }
        } else if flit.get_next_id().unwrap() == self.id {
            // receiving_flit_bufferのchannel_id番目のFlitBufferにpushする
            self.receiving_flit_buffer
                .get_mut(&channel_id)
                .unwrap()
                .push(flit.clone());
        }
    }

    pub fn send_new_packet(&mut self, packet: &InjectionPacket) {
        self.routing.push_new_packet(packet);
    }

    pub fn forward_flits(&mut self, channel_id: ChannelId) {
        // receiving_flit_bufferからsending_flit_bufferへフリットを転送する
        if let Some(flit) = self
            .receiving_flit_buffer
            .get_mut(&channel_id)
            .unwrap()
            .pop()
        {
            let new_flit = self.routing.forward_flit(&flit);
            let channel_id = new_flit.get_channel_id().unwrap();

            self.sending_flit_buffer
                .get_mut(&channel_id)
                .unwrap()
                .push(new_flit);
        }
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

        network.update(1);
        let flit = network.send_flit(0).unwrap();

        assert_eq!(flit.get_source_id().unwrap(), "test".to_string());
        assert_eq!(flit.get_dest_id().unwrap(), "broadcast".to_string());
        assert_eq!(flit.get_packet_id().unwrap(), 0);
        assert_eq!(flit.get_next_id().unwrap(), "broadcast".to_string());
        assert_eq!(flit.get_prev_id().unwrap(), "test".to_string());
        assert_eq!(flit.get_flits_len().unwrap(), 1);
        assert_eq!(flit.get_channel_id().unwrap(), 0);
    }
}
