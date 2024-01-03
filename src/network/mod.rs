pub mod core_functions;
pub mod flit;
pub mod flit_buffer;
pub mod option;
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
use crate::network::option::UpdateOption;
use crate::sim::node_type::NodeType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Network {
    id: String,
    cur_cycle: u32,
    switching: Switching,
    core: CoreFunction,
    sending_flit_buffer: HashMap<u8, FlitBuffer>,
    receiving_flit_buffer: HashMap<u8, FlitBuffer>,
    received_flits_buffer: ReceivedFlitsBuffer,
    send_history: (bool, Flit),
    channel_num: u8,
}

impl Network {
    pub fn new(
        id: &str,
        vc_num: u8,
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
        let core = CoreFunction::new(rf_kind, node_type, vc_num);
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
            send_history: (false, Flit::default()),
            channel_num: vc_num,
        }
    }

    pub fn send_flit(&mut self) -> Option<Flit> {
        // sending_flit_bufferのchannel_id番目のFlitBufferからpopする
        let channel_id = self.select_channel();
        if let Some(flit) = self.sending_flit_buffer.get_mut(&channel_id).unwrap().pop() {
            // log
            self.log_handler(Some(&flit), None);

            // history
            self.send_history.1 = flit.clone();
            self.send_history.0 = !flit.is_last();

            return Some(flit);
        }

        None
    }

    pub fn update(&mut self, cur_cycle: u32, option: Option<&UpdateOption>) {
        self.cur_cycle = cur_cycle;

        self.core.update(option);

        // 送信待ちのパケットを取りに行く
        if let Some(packet) = self.core.send_packet() {
            // packetをフリットに変換する
            let flits = packet_to_flits(&packet);

            // log
            self.log_handler(None, Some(&packet));

            // 送信待ちのパケットがあったら
            // sending_flit_bufferのchannel_id番目のFlitBufferにpushする
            for flit in flits {
                let mut channel_id = flit.get_channel_id().unwrap();

                if channel_id > self.channel_num {
                    channel_id = 0;
                }

                self.sending_flit_buffer
                    .get_mut(&channel_id)
                    .unwrap()
                    .push(&flit);
            }
        }

        if self.switching == Switching::CutThrough {
            // ルーティングするフリットの処理
            let channel_ids: Vec<u8> = self.receiving_flit_buffer.keys().cloned().collect();
            // receiving_flit_bufferからsending_flit_bufferへフリットを転送する
            for channel_id in channel_ids {
                self.forward_flits(channel_id);
            }
        }
    }

    pub fn receive_flit(&mut self, flit: &Flit, channel_id: u8) {
        if let Flit::Ack(_) = flit {
            // ack flitは受け取らない
            return;
        }

        let channel_id = if channel_id > self.channel_num {
            0
        } else {
            channel_id
        };

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
            if self.switching == Switching::StoreAndForward {
                // ルーティングするフリットの処理
                self.received_flits_buffer.push_flit(flit);

                if flit.is_last() {
                    if let Some(packet) = self.received_flits_buffer.pop_packet(
                        &flit.get_source_id().unwrap(),
                        flit.get_packet_id().unwrap(),
                    ) {
                        eprintln!("receive routing packet {:?}", packet);
                        self.core.receive_packet(&packet);

                        // log
                        self.log_handler(Some(flit), None);
                    }
                }
            }

            if self.switching == Switching::CutThrough {
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
    }

    pub fn send_new_packet(&mut self, packet: &InjectionPacket) {
        self.core.push_new_packet(packet);
    }

    pub fn is_joined(&self) -> bool {
        self.core.is_joined()
    }

    pub fn get_pid(&self) -> String {
        self.id.clone()
    }

    pub fn get_parent_pid(&self) -> Vec<Option<String>> {
        self.core
            .get_parent_id()
            .iter()
            .map(|id| {
                if id.is_some() {
                    get_pid(id.unwrap())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Network {
    fn forward_flits(&mut self, channel_id: u8) {
        // ここでCutThroughならフリットをそのまま転送する
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

    fn select_channel(&self) -> u8 {
        // historyを見て現在送信中のパケットがあったらそれが優先
        if !self.send_history.1.is_empty() && self.send_history.0 {
            return self.send_history.1.get_channel_id().unwrap();
        }

        // 送信待ちのフリットがあるchannel_idを返す
        let channel_ids: Vec<u8> = self.sending_flit_buffer.keys().cloned().collect();
        for channel_id in channel_ids {
            if !self
                .sending_flit_buffer
                .get(&channel_id)
                .unwrap()
                .is_empty()
            {
                return channel_id;
            }
        }

        // 送信待ちのフリットがない場合は適当に0を返す
        0
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
                route_info: if flit.is_header() {
                    Some(self.id.clone())
                } else {
                    None
                },
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
                channel_id: packet.channel_id,
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
        network.update(0, None);

        // preq
        network.send_flit().unwrap();

        network.update(1, None);
        let flit = network.send_flit().unwrap();

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
        network.update(0, None);

        // preq
        network.send_flit().unwrap();

        network.update(1, None);
        let flit = network.send_flit().unwrap();

        assert_eq!(flit.get_source_id().unwrap(), "test".to_string());
        assert_eq!(flit.get_dest_id().unwrap(), "broadcast".to_string());
        assert_eq!(flit.get_packet_id().unwrap(), 0);
        assert_eq!(flit.get_next_id().unwrap(), "broadcast".to_string());
        assert_eq!(flit.get_prev_id().unwrap(), "test".to_string());
        assert_eq!(flit.get_flits_len().unwrap(), 1);
        assert_eq!(flit.get_channel_id().unwrap(), 0);
    }
}
