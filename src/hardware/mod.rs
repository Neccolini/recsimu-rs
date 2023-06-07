pub(crate) mod constants;
pub mod flit;
pub mod state;

use self::flit::{AckFlit, Flit};
use self::state::State;
pub struct Hardware {
    pub state: State,
    retransmission_buffer: Flit,
    ack_buffer: Flit,
}

impl Hardware {
    pub fn new() -> Self {
        Self {
            state: State::default(),
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

    pub fn update(&mut self) {}

    pub fn ack_gen(&mut self, flit: &Flit) -> Result<Flit, Box<dyn std::error::Error>> {
        if let Flit::Ack(_) = flit {
            return Err("ack_gen: flit is ack".into());
        }

        // flitの中身を取り出す
        let (rcv_source_id, rcv_dest_id, rcv_packet_id, flit_num) = match flit {
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
                panic!("ack_gen: flit is not header or data");
            }
        };

        self.ack_buffer = Flit::Ack(AckFlit {
            source_id: rcv_dest_id,
            dest_id: rcv_source_id,
            packet_id: rcv_packet_id,
            flit_num,
        });

        Ok(self.ack_buffer.clone())
    }

    pub fn receive_ack(&mut self, flit: &Flit) -> Result<Flit, Box<dyn std::error::Error>> {
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
                Err(
                    "receive_ack: ack is not matched {ack_flit:?} {self.retransmission_buffer:?}"
                        .into(),
                )
            }
        } else {
            panic!("receive_ack: flit is not ack {flit:?}");
        }
    }
}

impl Default for Hardware {
    fn default() -> Self {
        Self::new()
    }
}
