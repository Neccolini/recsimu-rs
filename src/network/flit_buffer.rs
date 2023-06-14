use std::collections::VecDeque;

use crate::hardware::flit::Flit;

#[derive(Debug)]
pub struct FlitBuffer {
    flit_buffer: VecDeque<Flit>,
}

impl FlitBuffer {
    pub fn new() -> Self {
        FlitBuffer {
            flit_buffer: VecDeque::new(),
        }
    }

    pub fn push(&mut self, flit: Flit) {
        self.flit_buffer.push_back(flit);
    }

    pub fn pop(&mut self) -> Option<Flit> {
        self.flit_buffer.pop_front()
    }
}

impl Default for FlitBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::flit::{Flit, HeaderFlit};
    #[test]
    fn test_flit_buffer() {
        let mut flit_buffer = FlitBuffer::new();

        let flit0 = Flit::Header(HeaderFlit {
            channel_id: 0,
            packet_id: 0,
            dest_id: "".to_string(),
            source_id: "".to_string(),
            next_id: "".to_string(),
            flits_len: 0,
        });
        flit_buffer.push(flit0.clone());

        let flit1 = Flit::Header(HeaderFlit {
            channel_id: 0,
            packet_id: 1,
            dest_id: "".to_string(),
            source_id: "".to_string(),
            next_id: "".to_string(),
            flits_len: 0,
        });
        flit_buffer.push(flit1.clone());

        let flit2 = Flit::Header(HeaderFlit {
            channel_id: 0,
            packet_id: 2,
            dest_id: "".to_string(),
            source_id: "".to_string(),
            next_id: "".to_string(),
            flits_len: 0,
        });

        flit_buffer.push(flit2.clone());

        assert_eq!(flit_buffer.pop(), Some(flit0));
        assert_eq!(flit_buffer.pop(), Some(flit1));
        assert_eq!(flit_buffer.pop(), Some(flit2));
        assert_eq!(flit_buffer.pop(), None);
    }
}
