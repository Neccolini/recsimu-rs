use crate::network::flit::Flit;

pub const BLOCK_FLIT: bool = false;
pub const RECEIVE_FLIT: bool = true;

pub struct Blocking {
    is_block_mode: bool,
    is_receiving: bool,
    receiving_packet_next_id: String,
    receiving_packet_id: u32,
    cur_flit_num: u32,
}

impl Blocking {
    pub fn new(is_block_mode: bool) -> Self {
        Self {
            is_block_mode,
            is_receiving: false,
            receiving_packet_next_id: "".to_string(),
            receiving_packet_id: 0,
            cur_flit_num: 0,
        }
    }

    pub fn check_received_flit(&mut self, flit: &Flit) -> bool {
        if !self.is_block_mode {
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
                return RECEIVE_FLIT;
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
                return BLOCK_FLIT;
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
                return BLOCK_FLIT;
            }
            Flit::Ack(_) => {
                // ほかのパケットを受信中ならブロックする
                if self.is_receiving {
                    return BLOCK_FLIT;
                }
                return RECEIVE_FLIT;
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
}

impl Default for Blocking {
    fn default() -> Self {
        Self::new(false)
    }
}
