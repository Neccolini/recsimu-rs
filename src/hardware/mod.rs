mod blocking;
pub(crate) mod constants;
pub mod state;
pub mod switching;
extern crate rand;
use self::{
    state::{NodeState, State},
    switching::Switching,
};
use crate::network::flit::{AckFlit, Flit};
use rand::Rng;

#[derive(Default)]
pub struct Hardware {
    pub id: String,
    pub state: NodeState,
    pub retransmission_buffer: Flit,
    pub ack_buffer: Flit,
    received_msg_is_broadcast: bool,
    received_msg_is_ack: bool,
    blocking: blocking::Blocking,
}

impl Hardware {
    pub fn new(id: String, switching: Switching) -> Self {
        Self {
            id,
            state: NodeState::default(),
            retransmission_buffer: Flit::default(),
            ack_buffer: Flit::default(),
            received_msg_is_broadcast: false,
            received_msg_is_ack: false,
            blocking: blocking::Blocking::new(switching),
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

        if ack.is_ack() {
            self.ack_buffer.clear();

            return Ok(ack);
        }

        Ok(Flit::default())
    }

    pub fn receive_flit(
        &mut self,
        flit: &Flit,
    ) -> Result<Option<Flit>, Box<dyn std::error::Error>> {
        // Data, Header Flitの場合はackを生成する
        // Ack Flitの場合はtransmission_bufferを更新する
        if let Some(next_id) = flit.get_next_id() {
            if next_id != self.id && next_id != "broadcast" {
                return Ok(None);
            }

            self.received_msg_is_broadcast = next_id == "broadcast";
        }

        if self.blocking.check_received_flit(flit) == blocking::BLOCK_FLIT {
            return Ok(None);
        }

        match flit {
            Flit::Header(_) => {
                let _ack = self.ack_gen(flit)?;

                self.received_msg_is_ack = false;

                Ok(Some(flit.clone()))
            }
            Flit::Data(_) => {
                let _ack = self.ack_gen(flit)?;

                self.received_msg_is_ack = false;

                Ok(Some(flit.clone()))
            }
            Flit::Tail(_) => {
                let _ack = self.ack_gen(flit)?;

                self.received_msg_is_ack = false;

                Ok(Some(flit.clone()))
            }
            Flit::Ack(_) => {
                let _ack = self.receive_ack(flit)?;
                // self.ack_buffer = ack; // todo 右辺はNoneでは？

                self.received_msg_is_ack = true;

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
                if let Flit::Data(_) | Flit::Header(_) | Flit::Tail(_) = self.retransmission_buffer
                {
                    self.state.next(&State::Sending);
                }
            }
            State::Receiving => {
                if self.received_msg_is_ack {
                    self.state.set_resend_times(0);

                    self.state.next(&State::Idle);
                } else {
                    self.state.next(&State::ReplyAck);
                }
            }
            State::ReplyAck => {
                self.state.next(&State::Idle);
            }
            State::Sending => {
                // resend_times
                let resend_times = self.state.get_resend_times();
                if resend_times < constants::MAX_RESEND_TIMES {
                    self.state
                        .next(&State::waiting_state(self.calc_wait_cycles()));
                    self.state.set_resend_times(resend_times + 1);
                } else {
                    self.state.set_resend_times(0);
                    self.state.next(&State::waiting_state(0));
                }
            }
            State::Waiting(_) => {
                let remaining_cycles = match self.state.get() {
                    State::Waiting(state::Waiting { remaining_cycles }) => *remaining_cycles,
                    _ => panic!("update_state: state is not waiting"),
                };
                if remaining_cycles == 0 {
                    if let Flit::Data(_) | Flit::Header(_) | Flit::Tail(_) =
                        self.retransmission_buffer
                    {
                        self.state.next(&State::Sending);
                    } else {
                        self.state.next(&State::Idle);
                    }
                } else {
                    self.state.next(&State::waiting_state(remaining_cycles - 1));
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
        let (prev_id, next_id, packet_id, flit_num, channel_id) = (
            flit.get_prev_id().unwrap(),
            flit.get_next_id().unwrap(),
            flit.get_packet_id().unwrap(),
            flit.get_flit_num().unwrap(),
            flit.get_channel_id().unwrap(),
        );

        if next_id == self.id {
            self.ack_buffer = Flit::Ack(AckFlit {
                source_id: next_id,
                dest_id: prev_id,
                packet_id,
                flit_num,
                channel_id,
            });

            return Ok(self.ack_buffer.clone());
        }

        Ok(Flit::default())
    }

    fn receive_ack(&mut self, flit: &Flit) -> Result<Flit, Box<dyn std::error::Error>> {
        // 受信したackの中身を取り出す
        if let Flit::Ack(ack_flit) = flit {
            let ack_flit = ack_flit.clone();

            let (prev_id, next_id, packet_id, flit_num) = (
                self.retransmission_buffer.get_prev_id().unwrap(),
                self.retransmission_buffer.get_next_id().unwrap(),
                self.retransmission_buffer.get_packet_id().unwrap(),
                self.retransmission_buffer.get_flit_num().unwrap(),
            );

            if ack_flit.dest_id == prev_id
                && ack_flit.source_id == next_id
                && ack_flit.packet_id == packet_id
                && ack_flit.flit_num == flit_num
            {
                // ackを受信したのでretransmission_bufferをクリアする
                self.retransmission_buffer = Flit::default();
                self.set_state(&State::Idle);

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

    fn calc_wait_cycles(&self) -> u32 {
        let resend_times = self.state.get_resend_times();

        if resend_times == 0 {
            return constants::WAIT_ACK_CYCLES;
        }

        let mut rng = rand::thread_rng();
        let begin = 2i32.pow(resend_times as u32 - 1);
        let end = 2i32.pow(resend_times as u32 + 1);

        let random_backoff = rng.gen_range(begin..end) as u32;

        constants::WAIT_ACK_CYCLES + random_backoff
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::flit::{DataFlit, HeaderFlit};
    #[test]
    fn test_send_flit() {
        let mut hardware = Hardware::new("source_id".to_string(), Switching::StoreAndForward);

        // データフリットを送信する
        let flit = Flit::Data(DataFlit {
            source_id: "source_id".to_string(),
            dest_id: "dest_id".to_string(),
            next_id: "next_id".to_string(),
            prev_id: "prev_id".to_string(),
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
        let mut hardware = Hardware::new("dest_id".to_string(), Switching::StoreAndForward);

        // データフリットを受信する
        let flit = Flit::Header(HeaderFlit {
            source_id: "source_id".to_string(),
            dest_id: "dest_id".to_string(),
            next_id: "dest_id".to_string(),
            prev_id: "prev_id".to_string(),
            packet_id: 0,
            flits_len: 1,
            channel_id: 0,
            data: vec![0; 8],
        });

        let received_flit = hardware.receive_flit(&flit).unwrap();
        assert_eq!(received_flit, Some(flit.clone()));
        assert_eq!(hardware.ack_buffer.is_empty(), false);
        assert_eq!(hardware.ack_buffer.is_ack(), true);
    }

    // calc_wait_cyclesのテスト
    #[test]
    fn test_calc_wait_cycles() {
        let mut hardware = Hardware::new("dest_id".to_string(), Switching::StoreAndForward);
        hardware.state.set_resend_times(0);
        assert_eq!(hardware.calc_wait_cycles(), 2);

        for _ in 0..100 {
            hardware.state.set_resend_times(1);
            let val = hardware.calc_wait_cycles();
            assert!(3 <= val && val <= 5);

            hardware.state.set_resend_times(2);
            let val = hardware.calc_wait_cycles();
            assert!(4 <= val && val <= 9);

            hardware.state.set_resend_times(3);
            let val = hardware.calc_wait_cycles();
            assert!(6 <= val && val <= 17);
        }
    }
}
