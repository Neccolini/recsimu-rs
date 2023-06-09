use crate::hardware::constants::DATA_BYTE_PER_FLIT;
use crate::utils::div_ceil;
use uuid::Uuid;
pub type PacketId = Uuid;
pub type NodeId = String;

#[derive(Clone, Default, Debug)]
pub enum Flit {
    Header(HeaderFlit),
    Data(DataFlit),
    Ack(AckFlit),
    #[default]
    Empty,
}

#[derive(Clone, Debug)]
pub struct HeaderFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub next_id: NodeId,
    pub packet_id: PacketId,
    pub flits_len: u32,
}

#[derive(Clone, Debug)]
pub struct DataFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub next_id: NodeId,
    pub packet_num: u32,
    pub flit_num: u32,
    pub resend_num: u32,
    pub data: Vec<u8>,
    // parity: u8,
    pub packet_id: PacketId,
}

#[derive(Clone, Debug)]
pub struct AckFlit {
    pub source_id: NodeId,
    pub dest_id: NodeId,
    pub packet_id: PacketId,
    pub flit_num: u32,
}

pub fn data_to_flits(
    data: Vec<u8>,
    source_id: NodeId,
    dest_id: NodeId,
    next_id: NodeId,
    packet_num: u32,
    packet_id: PacketId,
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
    }));

    // DATA_BYTE_PER_FLITでdataを分割する
    for (flit_num, data_chunk) in data.chunks(DATA_BYTE_PER_FLIT as usize).enumerate() {
        flits.push(Flit::Data(DataFlit {
            source_id,
            dest_id,
            next_id,
            packet_num,
            flit_num: flit_num as u32,
            resend_num: 0,
            data: data_chunk.to_vec(),
            packet_id,
        }));
    }
    flits
}
