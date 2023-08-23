use crate::hardware::constants::DATA_BYTE_PER_FLIT;
use crate::network::ChannelId;
use crate::utils::div_ceil;
use uuid::Uuid;

pub type PacketId = Uuid;
pub type NodeId = String;

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
    pub fn clear(&mut self) {
        *self = Flit::Empty;
    }

    pub fn get_next_id(&self) -> Option<NodeId> {
        match self {
            Flit::Header(flit) => Some(flit.next_id.clone()),
            Flit::Data(flit) => Some(flit.next_id.clone()),
            Flit::Tail(flit) => Some(flit.next_id.clone()),
            Flit::Ack(flit) => Some(flit.source_id.clone()),
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub next_id: NodeId,
    pub prev_id: NodeId,
    pub packet_id: u32,
    pub flits_len: u32,
    pub channel_id: ChannelId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TailFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub next_id: NodeId,
    pub prev_id: NodeId,
    pub flit_num: u32,
    pub resend_num: u8,
    pub data: Vec<u8>,
    pub packet_id: u32,
    pub channel_id: ChannelId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub next_id: NodeId,
    pub prev_id: NodeId,
    pub flit_num: u32,
    pub resend_num: u8,
    pub data: Vec<u8>,
    pub packet_id: u32,
    pub channel_id: ChannelId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AckFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub packet_id: u32,
    pub flit_num: u32,
    pub channel_id: ChannelId,
}

pub fn data_to_flits(
    data: Vec<u8>,
    source_id: NodeId,
    dest_id: NodeId,
    next_id: NodeId,
    prev_id: NodeId,
    packet_id: u32,
    channel_id: ChannelId,
) -> Vec<Flit> {
    let mut flits = Vec::new();
    let flits_len = div_ceil(data.len() as u32, DATA_BYTE_PER_FLIT) + 1;

    // header flit
    flits.push(Flit::Header(HeaderFlit {
        source_id: source_id.clone(),
        dest_id: dest_id.clone(),
        next_id: next_id.clone(),
        prev_id: prev_id.clone(),
        packet_id,
        flits_len,
        channel_id,
    }));

    // DATA_BYTE_PER_FLITでdataを分割する
    for (flit_num, data_chunk) in data.chunks(DATA_BYTE_PER_FLIT as usize).enumerate() {
        // 最後はtail flit
        if flit_num == flits_len as usize - 2 {
            flits.push(Flit::Tail(TailFlit {
                source_id,
                dest_id,
                next_id,
                prev_id,
                flit_num: flit_num as u32 + 2,
                resend_num: 0,
                data: data_chunk.to_vec(),
                packet_id,
                channel_id,
            }));
            break;
        }
        flits.push(Flit::Data(DataFlit {
            source_id: source_id.clone(),
            dest_id: dest_id.clone(),
            next_id: next_id.clone(),
            prev_id: prev_id.clone(),
            flit_num: flit_num as u32 + 2,
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
        let data = vec![0; 100];
        let source_id = "source".to_string();
        let dest_id = "dest".to_string();
        let next_id = "next".to_string();
        let prev_id = next_id.clone();
        let packet_id = 0;
        let channel_id = 0;
        let flits = data_to_flits(
            data,
            source_id.clone(),
            dest_id.clone(),
            next_id.clone(),
            prev_id.clone(),
            packet_id,
            channel_id,
        );
        match DATA_BYTE_PER_FLIT {
            32 => {
                assert_eq!(flits.len(), 5);
                assert!(flits[0].is_header());
                assert!(flits[1].is_data());
                assert!(flits[2].is_data());
            }
            64 => {
                assert_eq!(flits.len(), 3);
                assert!(flits[0].is_header());
                assert!(flits[1].is_data());
                assert!(flits[2].is_tail());
            }
            _ => {
                dbg!(flits.len());
            }
        }
    }
    #[test]
    fn test_flits_to_data() {
        let data = vec![0; 100];
        let source_id = "source".to_string();
        let dest_id = "dest".to_string();
        let next_id = "next".to_string();
        let prev_id = next_id.clone();
        let packet_id = 0;
        let channel_id = 0;
        let flits = data_to_flits(
            data,
            source_id.clone(),
            dest_id.clone(),
            next_id.clone(),
            prev_id.clone(),
            packet_id,
            channel_id,
        );

        let data = flits_to_data(&flits);
        assert_eq!(data.len(), 100);
    }
}
