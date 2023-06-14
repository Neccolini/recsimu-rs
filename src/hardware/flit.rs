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

    pub fn clear(&mut self) {
        *self = Flit::Empty;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeaderFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub next_id: NodeId,
    pub packet_id: u32,
    pub flits_len: u32,
    pub channel_id: ChannelId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub next_id: NodeId,
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
    packet_id: u32,
    channel_id: ChannelId,
) -> Vec<Flit> {
    let mut flits = Vec::new();
    let flits_len = div_ceil(data.len() as u32, DATA_BYTE_PER_FLIT);

    // header flit
    flits.push(Flit::Header(HeaderFlit {
        source_id: source_id.clone(),
        dest_id: dest_id.clone(),
        next_id: next_id.clone(),
        packet_id,
        flits_len,
        channel_id,
    }));

    // DATA_BYTE_PER_FLITでdataを分割する
    for (flit_num, data_chunk) in data.chunks(DATA_BYTE_PER_FLIT as usize).enumerate() {
        flits.push(Flit::Data(DataFlit {
            source_id: source_id.clone(),
            dest_id: dest_id.clone(),
            next_id: next_id.clone(),
            flit_num: flit_num as u32,
            resend_num: 0,
            data: data_chunk.to_vec(),
            packet_id,
            channel_id,
        }));
    }
    flits
}
