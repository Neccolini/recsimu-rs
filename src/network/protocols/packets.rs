use crate::hardware::flit::Flit;

pub struct GeneralPacket {
    message: String,
    dest_id: String,
    source_id: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct DefaultPacket {
    pub(crate) message: String,
    pub(crate) dest_id: String,
    pub(crate) source_id: String,
}

impl DefaultPacket {
    pub(crate) fn new(gp: &GeneralPacket) -> Self {
        DefaultPacket {
            message: gp.message.clone(),
            dest_id: gp.dest_id.clone(),
            source_id: gp.source_id.clone(),
        }
    }

    pub(crate) fn to_flits(&self) -> Vec<Flit> {
        todo!();
    }
}
