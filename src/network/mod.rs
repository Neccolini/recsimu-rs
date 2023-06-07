pub mod protocols;
pub mod routing_function;

use self::routing_function::RoutingFunction;
use crate::hardware::flit::Flit;

pub struct Network {
    pub routing_function: RoutingFunction,
    pub sending_flit_buffer: Vec<Flit>,
    pub receiving_flit_buffer: Vec<Flit>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            routing_function: RoutingFunction::default(),
            sending_flit_buffer: Vec::new(),
            receiving_flit_buffer: Vec::new(),
        }
    }

    pub fn send_flit(&mut self) -> Option<Flit> {
        self.sending_flit_buffer.pop()
    }

    pub fn update(&mut self) {}

    pub fn receive_flit(&mut self, flit: &Flit) {
        self.receiving_flit_buffer.push(flit.clone());
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}
