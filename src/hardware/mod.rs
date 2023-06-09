pub(crate) mod constants;
pub mod state;

use self::state::{NodeState, State};
use crate::network::flit::{AckFlit, Flit};
#[derive(Default)]
pub struct Hardware {
    pub id: String,
    pub state: NodeState,
    pub retransmission_buffer: Flit,
    pub ack_buffer: Flit,
}

impl Hardware {
    pub fn new(id: String) -> Self {
        Self {
            id,
            state: NodeState::default(),
            retransmission_buffer: Flit::default(),
            ack_buffer: Flit::default(),
        }
    }
}

// nodeが使用するAPI
impl Hardware {
    pub fn send_flit(&mut self, flit: &Flit) -> Result<Flit, Box<dyn std::error::Error>> {
        self.retransmission_buffer = flit.clone();
        Ok(flit.clone())
    }

    pub fn send_ack(&mut self) -> Result<Flit, Box<dyn std::error::Error>> {
        let ack = self.ack_buffer.clone();
        assert!(
            ack.is_ack(),
            "send_ack: flit in the buffer is not ack flit: {0:?}",
            ack
        );

        self.ack_buffer.clear();

        Ok(ack)
    }

    pub fn receive_flit(
        &mut self,
        flit: &Flit,
    ) -> Result<Option<Flit>, Box<dyn std::error::Error>> {
        // Data, Header Flitの場合はackを生成する
        // Ack Flitの場合はtransmission_bufferを更新する
        let next_id = flit.get_next_id().unwrap();
        if next_id != self.id && next_id != "broadcast" {
            return Ok(None);
        }

        match flit {
            Flit::Data(_) | Flit::Header(_) | Flit::Tail(_) => {
                let _ack = self.ack_gen(flit)?;
                Ok(Some(flit.clone()))
            }
            Flit::Ack(_) => {
                let ack = self.receive_ack(flit)?;
                self.ack_buffer = ack;
                Ok(Some(flit.clone()))
            }
            _ => {
                panic!("receive_flit: flit is not header, data, or ack {flit:?}");
            }
        }
    }

    pub fn update_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // stateを更新する
        match self.state.get() {
            State::Idle => {
                // retransmission_bufferが空でない場合は送信状態へ遷移
                if let Flit::Data(_) | Flit::Header(_) = self.retransmission_buffer {
                    self.state.next(&State::Sending);
                }
            }
            State::Receiving => {
                self.state.next(&State::ReplyAck);
            }
            State::ReplyAck => {
                self.state.next(&State::Idle);
            }
            State::Sending => {
                self.state.next(&State::waiting_state(0)); // todo 0を変数にする
            }
            State::Waiting(_) => {
                let remaining_cycles = match self.state.get() {
                    State::Waiting(state::Waiting { remaining_cycles }) => *remaining_cycles,
                    _ => panic!("update_state: state is not waiting"),
                };

                if remaining_cycles == 0 {
                    self.state.next(&State::Sending);
                } else {
                    self.state.next(&State::waiting_state(remaining_cycles - 1))
                }
            }
        }
        Ok(())
    }

    pub fn set_state(&mut self, state: &State) {
        self.state.next(state);
    }

    pub fn check_flit(&self, _flit: &Flit) -> Result<Option<Flit>, Box<dyn std::error::Error>> {
        unimplemented!();
    }
}

// 外部に公開しない関数
impl Hardware {
    fn ack_gen(&mut self, flit: &Flit) -> Result<Flit, Box<dyn std::error::Error>> {
        if let Flit::Ack(_) = flit {
            return Err("ack_gen: flit is ack".into());
        }

        // flitの中身を取り出す
        let (rcv_source_id, rcv_dest_id, rcv_packet_id, flit_num, channel_id) = match flit {
            Flit::Header(header_flit) => {
                let header_flit = header_flit.clone();
                (
                    header_flit.source_id,
                    header_flit.dest_id,
                    header_flit.packet_id,
                    0,
                    header_flit.channel_id,
                )
            }
            Flit::Data(data_flit) => {
                let data_flit = data_flit.clone();
                (
                    data_flit.source_id,
                    data_flit.dest_id,
                    data_flit.packet_id,
                    data_flit.flit_num,
                    data_flit.channel_id,
                )
            }
            Flit::Tail(tail_flit) => {
                let tail_flit = tail_flit.clone();
                (
                    tail_flit.source_id,
                    tail_flit.dest_id,
                    tail_flit.packet_id,
                    tail_flit.flit_num,
                    tail_flit.channel_id,
                )
            }
            _ => {
                panic!("ack_gen: flit is not header or data");
            }
        };

        self.ack_buffer = Flit::Ack(AckFlit {
            source_id: rcv_dest_id,
            dest_id: rcv_source_id,
            packet_id: rcv_packet_id,
            flit_num,
            channel_id,
        });

        Ok(self.ack_buffer.clone())
    }

    fn receive_ack(&mut self, flit: &Flit) -> Result<Flit, Box<dyn std::error::Error>> {
        // 受信したackの中身を取り出す
        if let Flit::Ack(ack_flit) = flit {
            let ack_flit = ack_flit.clone();

            let (src_id, dest_id, packet_id, flit_num) = match &self.retransmission_buffer {
                Flit::Header(header_flit) => {
                    let header_flit = header_flit.clone();
                    (
                        header_flit.source_id,
                        header_flit.dest_id,
                        header_flit.packet_id,
                        0,
                    )
                }
                Flit::Data(data_flit) => {
                    let data_flit = data_flit.clone();
                    (
                        data_flit.source_id,
                        data_flit.dest_id,
                        data_flit.packet_id,
                        data_flit.flit_num,
                    )
                }
                _ => {
                    panic!("receive_ack: retransmission_buffer is not header or data");
                }
            };

            if ack_flit.dest_id == src_id
                && ack_flit.source_id == dest_id
                && ack_flit.packet_id == packet_id
                && ack_flit.flit_num == flit_num
            {
                // ackを受信したのでretransmission_bufferをクリアする
                self.retransmission_buffer = Flit::default();
                Ok(flit.clone())
            } else {
                Err(format!(
                    "receive_ack: ack is not matched. {ack_flit:?} and {:?}",
                    self.retransmission_buffer
                )
                .into())
            }
        } else {
            panic!("receive_ack: flit is not ack {flit:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::flit::DataFlit;
    #[test]
    fn test_send_flit() {
        let mut hardware = Hardware::new("source_id".to_string());

        // データフリットを送信する
        let flit = Flit::Data(DataFlit {
            source_id: "source_id".to_string(),
            dest_id: "dest_id".to_string(),
            next_id: "next_id".to_string(),
            resend_num: 0,
            packet_id: 0,
            flit_num: 0,
            channel_id: 0,
            data: vec![0; 8],
        });

        let sended_flit = hardware.send_flit(&flit).unwrap();
        assert_eq!(sended_flit, flit.clone());
        assert_eq!(hardware.retransmission_buffer, flit.clone());
        // assert_eq!(hardware.state.get(), State::Sending);
    }

    #[test]
    fn test_receive_flit() {
        let mut hardware = Hardware::new("dest_id".to_string());

        // データフリットを受信する
        let flit = Flit::Data(DataFlit {
            source_id: "source_id".to_string(),
            dest_id: "dest_id".to_string(),
            next_id: "dest_id".to_string(),
            resend_num: 0,
            packet_id: 0,
            flit_num: 0,
            channel_id: 0,
            data: vec![0; 8],
        });

        let received_flit = hardware.receive_flit(&flit).unwrap();
        assert_eq!(received_flit, Some(flit.clone()));
        assert_eq!(hardware.ack_buffer.is_empty(), false);
        assert_eq!(hardware.ack_buffer.is_ack(), true);
    }
}
