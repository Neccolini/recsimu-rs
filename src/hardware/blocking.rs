use crate::network::flit::Flit;

use super::switching::Switching;

pub const BLOCK_FLIT: bool = false;
pub const RECEIVE_FLIT: bool = true;

pub struct Blocking {
    switching: Switching,
    is_receiving: bool,
    receiving_packet_next_id: String,
    receiving_packet_id: u32,
    cur_flit_num: u32,
}

impl Blocking {
    pub fn new(switching: Switching) -> Self {
        Self {
            switching,
            is_receiving: false,
            receiving_packet_next_id: "".to_string(),
            receiving_packet_id: 0,
            cur_flit_num: 0,
        }
    }

    pub fn check_received_flit(&mut self, flit: &Flit) -> bool {
        if !self.is_block_mode() {
            return RECEIVE_FLIT;
        }

        match flit {
            Flit::Header(_) => {
                // ほかのパケットを受信中ならブロックする
                if self.is_receiving {
                    return BLOCK_FLIT;
                } else if flit.get_flits_len().unwrap() > 1 {
                    self.is_receiving = true;
                    self.receiving_packet_next_id = flit.get_next_id().unwrap();
                    self.receiving_packet_id = flit.get_packet_id().unwrap();
                    self.cur_flit_num = 0;
                }
                RECEIVE_FLIT
            }
            Flit::Data(data_flit) => {
                if self.is_receiving {
                    let next_id = data_flit.next_id.clone();
                    let packet_id = data_flit.packet_id;

                    if self.receiving_packet_next_id == next_id
                        && self.receiving_packet_id == packet_id
                        && self.cur_flit_num + 1 == data_flit.flit_num
                    {
                        self.cur_flit_num += 1;

                        return RECEIVE_FLIT;
                    }
                }
                BLOCK_FLIT
            }
            Flit::Tail(tail_flit) => {
                if self.is_receiving {
                    let next_id = tail_flit.next_id.clone();
                    let packet_id = tail_flit.packet_id;

                    if self.receiving_packet_next_id == next_id
                        && self.receiving_packet_id == packet_id
                        && self.cur_flit_num + 1 == tail_flit.flit_num
                    {
                        self.reset();

                        return RECEIVE_FLIT;
                    }
                }
                BLOCK_FLIT
            }
            Flit::Ack(_) => {
                // ほかのパケットを受信中ならブロックする
                if self.is_receiving {
                    return BLOCK_FLIT;
                }
                RECEIVE_FLIT
            }
            Flit::Empty => BLOCK_FLIT,
        }
    }

    fn reset(&mut self) {
        self.is_receiving = false;
        self.receiving_packet_next_id = "".to_string();
        self.receiving_packet_id = 0;
        self.cur_flit_num = 0;
    }

    fn is_block_mode(&self) -> bool {
        match self.switching {
            Switching::CutThrough => false,
            Switching::StoreAndForward => true,
        }
    }
}

impl Default for Blocking {
    fn default() -> Self {
        Self::new(Switching::StoreAndForward)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::flit::{AckFlit, DataFlit, HeaderFlit, TailFlit};

    #[test]
    fn test_check_received_flit() {
        let mut blocking = Blocking::new(Switching::StoreAndForward);

        let header_flit = Flit::Header(HeaderFlit {
            source_id: "0".to_string(),
            dest_id: "1".to_string(),
            next_id: "1".to_string(),
            prev_id: "0".to_string(),
            packet_id: 0,
            flits_len: 3,
            channel_id: 0,
            data: vec![0; 64],
        });

        let data_flit = Flit::Data(DataFlit {
            source_id: "0".to_string(),
            dest_id: "1".to_string(),
            next_id: "1".to_string(),
            prev_id: "0".to_string(),
            packet_id: 0,
            flit_num: 1,
            resend_num: 0,
            channel_id: 0,
            data: vec![0; 64],
        });

        let block_data_flit = Flit::Data(DataFlit {
            source_id: "5".to_string(),
            dest_id: "1".to_string(),
            next_id: "1".to_string(),
            prev_id: "0".to_string(),
            packet_id: 1,
            flit_num: 1,
            resend_num: 0,
            channel_id: 0,
            data: vec![0; 64],
        });

        let tail_flit = Flit::Tail(TailFlit {
            source_id: "0".to_string(),
            dest_id: "1".to_string(),
            next_id: "1".to_string(),
            prev_id: "0".to_string(),
            packet_id: 0,
            flit_num: 2,
            resend_num: 0,
            channel_id: 0,
            data: vec![0; 64],
        });

        let ack_flit = Flit::Ack(AckFlit {
            source_id: "0".to_string(),
            dest_id: "1".to_string(),
            packet_id: 0,
            flit_num: 4,
            channel_id: 0,
        });

        assert_eq!(blocking.check_received_flit(&header_flit), RECEIVE_FLIT);
        assert_eq!(blocking.check_received_flit(&block_data_flit), BLOCK_FLIT);
        assert_eq!(blocking.check_received_flit(&data_flit), RECEIVE_FLIT);
        assert_eq!(blocking.check_received_flit(&tail_flit), RECEIVE_FLIT);
        assert_eq!(blocking.check_received_flit(&ack_flit), RECEIVE_FLIT);
    }
}
