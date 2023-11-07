pub mod core_functions;
pub mod flit;
pub mod flit_buffer;
pub mod vid;

use self::core_functions::packets::InjectionPacket;
use self::core_functions::CoreFunction;
use self::flit::packet_to_flits;
use self::flit_buffer::{FlitBuffer, ReceivedFlitsBuffer};
use self::vid::*;
use crate::hardware::switching::Switching;

use crate::log::{post_new_packet_log, update_packet_log, NewPacketLogInfo, UpdatePacketLogInfo};
use crate::network::core_functions::packets::Packet;
use crate::network::flit::Flit;
use crate::sim::node_type::NodeType;
use std::collections::HashMap;

pub type ChannelId = u32;

#[derive(Debug, Clone)]
pub struct Network {
    id: String,
    cur_cycle: u32,
    switching: Switching,
    core: CoreFunction,
    sending_flit_buffer: HashMap<ChannelId, FlitBuffer>,
    receiving_flit_buffer: HashMap<ChannelId, FlitBuffer>,
    received_flits_buffer: ReceivedFlitsBuffer,
}

impl Network {
    pub fn new(
        id: &str,
        vc_num: ChannelId,
        switching: &Switching,
        rf_kind: &str,
        node_type: &NodeType,
    ) -> Self {
        // sending_flit_bufferとreceiving_flit_bufferを初期化する
        // 0...vc_num-1のchannel_idを持つFlitBufferを生成する
        let mut sending_flit_buffer = HashMap::new();
        let mut receiving_flit_buffer = HashMap::new();

        for i in 0..vc_num {
            sending_flit_buffer.insert(i, FlitBuffer::new());
            receiving_flit_buffer.insert(i, FlitBuffer::new());
        }
        let core = CoreFunction::new(rf_kind, node_type);
        let vid = core.get_id();

        add_to_vid_table(vid, id);

        Self {
            id: id.to_string(),
            cur_cycle: 0,
            switching: switching.clone(),
            core,
            sending_flit_buffer,
            receiving_flit_buffer,
            received_flits_buffer: ReceivedFlitsBuffer::new(),
        }
    }

    pub fn send_flit(&mut self, channel_id: ChannelId) -> Option<Flit> {
        // sending_flit_bufferのchannel_id番目のFlitBufferからpopする
        if let Some(flit) = self.sending_flit_buffer.get_mut(&channel_id).unwrap().pop() {
            // log
            self.log_handler(Some(&flit), None);

            return Some(flit);
        }

        None
    }

    pub fn update(&mut self, cur_cycle: u32) {
        self.cur_cycle = cur_cycle;

        self.core.update();

        // 送信待ちのパケットを取りに行く
        if let Some(packet) = self.core.send_packet() {
            // packetをフリットに変換する
            let flits = packet_to_flits(&packet);

            // log
            self.log_handler(None, Some(&packet));

            // 送信待ちのパケットがあったら
            // sending_flit_bufferのchannel_id番目のFlitBufferにpushする
            for flit in flits {
                let channel_id = flit.get_channel_id().unwrap();

                self.sending_flit_buffer
                    .get_mut(&channel_id)
                    .unwrap()
                    .push(&flit);
            }
        }

        if self.switching == Switching::CutThrough {
            // ルーティングするフリットの処理
            let channel_ids: Vec<ChannelId> = self.receiving_flit_buffer.keys().cloned().collect();
            // receiving_flit_bufferからsending_flit_bufferへフリットを転送する
            for channel_id in channel_ids {
                self.forward_flits(channel_id);
            }
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

            if flit.is_last() {
                if let Some(packet) = self.received_flits_buffer.pop_packet(
                    &flit.get_source_id().unwrap(),
                    flit.get_packet_id().unwrap(),
                ) {
                    self.core.receive_packet(&packet);

                    // log
                    self.log_handler(Some(flit), None);
                }
            }
        } else if flit.get_next_id().unwrap() == self.id {
            // receiving_flit_bufferのchannel_id番目のFlitBufferにpushする
            self.receiving_flit_buffer
                .get_mut(&channel_id)
                .unwrap()
                .push(flit);

            if flit.is_last() {
                while !self
                    .receiving_flit_buffer
                    .get(&channel_id)
                    .unwrap()
                    .is_empty()
                {
                    self.forward_flits(channel_id);
                }
            }
        }
    }

    pub fn send_new_packet(&mut self, packet: &InjectionPacket) {
        self.core.push_new_packet(packet);
    }

    pub fn is_joined(&self) -> bool {
        self.core.is_joined()
    }
}

impl Network {
    fn forward_flits(&mut self, channel_id: ChannelId) {
        // receiving_flit_bufferからsending_flit_bufferへフリットを転送する
        if let Some(flit) = self
            .receiving_flit_buffer
            .get_mut(&channel_id)
            .unwrap()
            .pop()
        {
            let new_flit = self.core.forward_flit(&flit);
            let channel_id = new_flit.get_channel_id().unwrap();

            self.sending_flit_buffer
                .get_mut(&channel_id)
                .unwrap()
                .push(&new_flit);
        }
    }

    fn log_handler(&self, flit: Option<&Flit>, packet: Option<&Packet>) {
        // 雑なassertion
        assert!(flit.is_some() || packet.is_some());
        assert!(flit.is_none() || packet.is_none());

        // 更新
        if let Some(flit) = flit {
            let packet_id =
                flit.get_source_id().unwrap() + "_" + &flit.get_packet_id().unwrap().to_string();

            let send_init = flit.get_source_id().unwrap() == self.id && flit.is_header();
            let is_delivered = flit.get_dest_id().unwrap() == self.id && flit.is_last();

            let update_log = UpdatePacketLogInfo {
                send_cycle: if send_init {
                    Some(self.cur_cycle)
                } else {
                    None
                },
                last_receive_cycle: if is_delivered {
                    Some(self.cur_cycle)
                } else {
                    None
                },
                route_info: Some(self.id.clone()),
                is_delivered: Some(is_delivered),
                flit_log: None,
            };

            let _ = update_packet_log(&packet_id, &update_log);
        }

        // 最初のパケット登録
        if let Some(packet) = packet {
            let packet_id = packet.source_id.clone() + "_" + &packet.packet_id.to_string();

            let log = NewPacketLogInfo {
                packet_id,
                from_id: packet.source_id.clone(),
                dest_id: packet.dest_id.clone(),
                flits_len: packet.get_flits_len(),
                message: self.core.get_message(packet),
            };

            let _ = post_new_packet_log(&log);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::core_functions::packets::InjectionPacket;

    #[test]
    fn test_send_flit1() {
        add_to_vid_table(u32::MAX, "broadcast");
        let mut network = Network::new(
            "test",
            1,
            &Switching::StoreAndForward,
            "default",
            &NodeType::Router,
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

    #[test]
    fn test_send_flit2() {
        add_to_vid_table(u32::MAX, "broadcast");
        let mut network = Network::new(
            "test",
            1,
            &Switching::CutThrough,
            "default",
            &NodeType::Router,
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
