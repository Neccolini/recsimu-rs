use crate::hardware::constants::DATA_BYTE_PER_FLIT;
use crate::network::ChannelId;
use crate::utils::div_ceil;
use std::error;

#[derive(Clone, Default, Debug, PartialEq)]
pub enum Flit {
    Header(HeaderFlit),
    Data(DataFlit),
    Tail(TailFlit),
    Ack(AckFlit),
    #[default]
    Empty,
}

impl Flit {
    pub fn is_empty(&self) -> bool {
        matches!(self, Flit::Empty)
    }

    pub fn is_ack(&self) -> bool {
        matches!(self, Flit::Ack(_))
    }

    pub fn is_data(&self) -> bool {
        matches!(self, Flit::Data(_))
    }
    pub fn is_header(&self) -> bool {
        matches!(self, Flit::Header(_))
    }
    pub fn is_tail(&self) -> bool {
        matches!(self, Flit::Tail(_))
    }

    // next_idがbroadcastならtrueを返す
    pub fn is_broadcast(&self) -> bool {
        let dest_id = match self {
            Flit::Header(flit) => flit.dest_id.clone(),
            Flit::Data(flit) => flit.dest_id.clone(),
            Flit::Tail(flit) => flit.dest_id.clone(),
            Flit::Ack(flit) => flit.dest_id.clone(),
            Flit::Empty => {
                dbg!("flit is empty");
                return false;
            }
        };

        dest_id == "broadcast"
    }

    pub fn clear(&mut self) {
        *self = Flit::Empty;
    }

    pub fn set_next_id(&mut self, next_id: &str) -> Result<(), Box<dyn error::Error>> {
        match self {
            Flit::Header(flit) => flit.next_id = next_id.to_string(),
            Flit::Data(flit) => flit.next_id = next_id.to_string(),
            Flit::Tail(flit) => flit.next_id = next_id.to_string(),
            Flit::Ack(flit) => flit.dest_id = next_id.to_string(),
            Flit::Empty => {
                dbg!("flit is empty");
                return Err("flit is empty".into());
            }
        }
        Ok(())
    }

    pub fn set_prev_id(&mut self, prev_id: &str) -> Result<(), Box<dyn error::Error>> {
        match self {
            Flit::Header(flit) => flit.prev_id = prev_id.to_string(),
            Flit::Data(flit) => flit.prev_id = prev_id.to_string(),
            Flit::Tail(flit) => flit.prev_id = prev_id.to_string(),
            Flit::Ack(flit) => flit.source_id = prev_id.to_string(),
            Flit::Empty => {
                dbg!("flit is empty");
                return Err("flit is empty".into());
            }
        }
        Ok(())
    }

    pub fn get_source_id(&self) -> Option<String> {
        match self {
            Flit::Header(flit) => Some(flit.source_id.clone()),
            Flit::Data(flit) => Some(flit.source_id.clone()),
            Flit::Tail(flit) => Some(flit.source_id.clone()),
            Flit::Ack(flit) => Some(flit.source_id.clone()),
            Flit::Empty => None,
        }
    }

    pub fn get_prev_id(&self) -> Option<String> {
        match self {
            Flit::Header(flit) => Some(flit.prev_id.clone()),
            Flit::Data(flit) => Some(flit.prev_id.clone()),
            Flit::Tail(flit) => Some(flit.prev_id.clone()),
            Flit::Ack(ack) => Some(ack.source_id.clone()),
            Flit::Empty => None,
        }
    }

    pub fn get_next_id(&self) -> Option<String> {
        match self {
            Flit::Header(flit) => Some(flit.next_id.clone()),
            Flit::Data(flit) => Some(flit.next_id.clone()),
            Flit::Tail(flit) => Some(flit.next_id.clone()),
            Flit::Ack(ack) => Some(ack.dest_id.clone()),
            Flit::Empty => None,
        }
    }

    pub fn get_dest_id(&self) -> Option<String> {
        match self {
            Flit::Header(flit) => Some(flit.dest_id.clone()),
            Flit::Data(flit) => Some(flit.dest_id.clone()),
            Flit::Tail(flit) => Some(flit.dest_id.clone()),
            Flit::Ack(flit) => Some(flit.dest_id.clone()),
            Flit::Empty => None,
        }
    }

    pub fn get_channel_id(&self) -> Option<ChannelId> {
        match self {
            Flit::Header(flit) => Some(flit.channel_id),
            Flit::Data(flit) => Some(flit.channel_id),
            Flit::Tail(flit) => Some(flit.channel_id),
            Flit::Ack(flit) => Some(flit.channel_id),
            Flit::Empty => None,
        }
    }

    pub fn get_packet_id(&self) -> Option<u32> {
        match self {
            Flit::Header(flit) => Some(flit.packet_id),
            Flit::Data(flit) => Some(flit.packet_id),
            Flit::Tail(flit) => Some(flit.packet_id),
            Flit::Ack(flit) => Some(flit.packet_id),
            Flit::Empty => None,
        }
    }

    pub fn get_flit_num(&self) -> Option<u32> {
        match self {
            Flit::Header(_) => Some(0),
            Flit::Data(flit) => Some(flit.flit_num),
            Flit::Tail(flit) => Some(flit.flit_num),
            Flit::Ack(flit) => Some(flit.flit_num),
            Flit::Empty => None,
        }
    }

    pub fn get_flits_len(&self) -> Option<u32> {
        match self {
            Flit::Header(flit) => Some(flit.flits_len),
            Flit::Data(_) => None,
            Flit::Tail(_) => None,
            Flit::Ack(_) => None,
            Flit::Empty => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderFlit {
    pub source_id: String,
    pub dest_id: String,
    pub next_id: String,
    pub prev_id: String,
    pub packet_id: u32,
    pub flits_len: u32,
    pub data: Vec<u8>,
    pub channel_id: ChannelId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TailFlit {
    pub source_id: String,
    pub dest_id: String,
    pub next_id: String,
    pub prev_id: String,
    pub flit_num: u32,
    pub resend_num: u8,
    pub data: Vec<u8>,
    pub packet_id: u32,
    pub channel_id: ChannelId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataFlit {
    pub source_id: String,
    pub dest_id: String,
    pub next_id: String,
    pub prev_id: String,
    pub flit_num: u32,
    pub resend_num: u8,
    pub data: Vec<u8>,
    pub packet_id: u32,
    pub channel_id: ChannelId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AckFlit {
    pub source_id: String,
    pub dest_id: String,
    pub packet_id: u32,
    pub flit_num: u32,
    pub channel_id: ChannelId,
}

pub fn data_to_flits(
    data: Vec<u8>,
    source_id: &str,
    dest_id: &str,
    next_id: &str,
    prev_id: &str,
    packet_id: u32,
    channel_id: ChannelId,
) -> Vec<Flit> {
    let mut flits = Vec::new();
    let flits_len = div_ceil(data.len() as u32, DATA_BYTE_PER_FLIT);

    // DATA_BYTE_PER_FLITでdataを分割する
    for (flit_num, data_chunk) in data.chunks(DATA_BYTE_PER_FLIT as usize).enumerate() {
        if flit_num == 0 {
            flits.push(Flit::Header(HeaderFlit {
                source_id: source_id.to_string(),
                dest_id: dest_id.to_string(),
                next_id: next_id.to_string(),
                prev_id: prev_id.to_string(),
                packet_id,
                data: data_chunk.to_vec(),
                flits_len,
                channel_id,
            }));
            continue;
        } else if flit_num == flits_len as usize - 1 {
            flits.push(Flit::Tail(TailFlit {
                source_id: source_id.to_string(),
                dest_id: dest_id.to_string(),
                next_id: next_id.to_string(),
                prev_id: prev_id.to_string(),
                flit_num: flit_num as u32 + 1,
                resend_num: 0,
                data: data_chunk.to_vec(),
                packet_id,
                channel_id,
            }));
            break;
        }
        flits.push(Flit::Data(DataFlit {
            source_id: source_id.to_string(),
            dest_id: dest_id.to_string(),
            next_id: next_id.to_string(),
            prev_id: prev_id.to_string(),
            flit_num: flit_num as u32 + 1,
            resend_num: 0,
            data: data_chunk.to_vec(),
            packet_id,
            channel_id,
        }));
    }
    flits
}

pub fn flits_to_data(flits: &Vec<Flit>) -> Vec<u8> {
    let mut data = Vec::new();
    for flit in flits {
        match flit {
            Flit::Header(flit) => data.extend_from_slice(&flit.data),
            Flit::Data(flit) => data.extend_from_slice(&flit.data),
            Flit::Tail(flit) => data.extend_from_slice(&flit.data),
            _ => {}
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_data_to_flits() {
        let flits = data_to_flits(vec![0; 100], "source", "dest", "next", "next", 0, 0);
        match DATA_BYTE_PER_FLIT {
            32 => {
                assert_eq!(flits.len(), 5);
                assert!(flits[0].is_header());
                assert!(flits[1].is_data());
                assert!(flits[2].is_data());
            }
            64 => {
                assert_eq!(flits.len(), 2);
                assert!(flits[0].is_header());
                assert!(flits[1].is_tail());
            }
            _ => {
                dbg!(flits.len());
            }
        }
    }
    #[test]
    fn test_flits_to_data() {
        let flits = data_to_flits(vec![0; 100], "source", "dest", "next", "next", 0, 0);

        let data = flits_to_data(&flits);
        assert_eq!(data.len(), 100);
    }
}
