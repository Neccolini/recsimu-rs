use std::collections::VecDeque;

use crate::hardware::flit::Flit;

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
