use crate::network::protocols::packets::DefaultPacket;
use std::collections::VecDeque;

pub struct DefaultProtocol {
    send_packet_buffer: VecDeque<DefaultPacket>,
    received_packet_buffer: VecDeque<DefaultPacket>,
}

impl DefaultProtocol {
    pub fn new() -> Self {
        DefaultProtocol {
            send_packet_buffer: VecDeque::new(),
            received_packet_buffer: VecDeque::new(),
        }
    }
}

impl Default for DefaultProtocol {
    fn default() -> Self {
        Self::new()
    }
}
