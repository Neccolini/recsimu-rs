use crate::network::flit::flits_to_data;
use crate::network::flit::Flit;
use std::collections::HashMap;
use std::collections::VecDeque;

use super::protocols::packets::GeneralPacket;

#[derive(Debug, Clone)]
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

    pub fn clear(&mut self) {
        self.flit_buffer.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.flit_buffer.is_empty()
    }

    fn remove_duplicate_and_sort(&mut self) {
        let mut unique_elements: VecDeque<Flit> = VecDeque::new();
        for elem in self.flit_buffer.iter() {
            if !unique_elements.contains(elem) {
                unique_elements.push_back(elem.clone());
            }
        }
        self.flit_buffer.clear();
        self.flit_buffer.extend(unique_elements);

        self.flit_buffer
            .make_contiguous()
            .sort_by_key(|a| a.get_flit_num());
    }
}

impl Default for FlitBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// 最終的に到達したフリットを格納するバッファ
#[derive(Debug, Clone, Default)]
pub struct ReceivedFlitsBuffer {
    buffer: HashMap<String, FlitBuffer>,
}

impl ReceivedFlitsBuffer {
    pub fn new() -> Self {
        ReceivedFlitsBuffer {
            buffer: HashMap::new(),
        }
    }

    pub fn push_flit(&mut self, flit: &Flit) {
        let from_id = flit.get_source_id().unwrap();
        let packet_id = flit.get_packet_id().unwrap();
        let key = format!("{}-{}", from_id, packet_id);

        if !self.buffer.contains_key(&key) {
            self.buffer.insert(key.clone(), FlitBuffer::new());
        }

        self.buffer.get_mut(&key).unwrap().push(flit.clone());
    }

    pub fn pop_packet(&mut self, from_id: &str, packet_id: u32) -> Option<GeneralPacket> {
        let key = format!("{}-{}", from_id, packet_id);
        if !self.buffer.contains_key(&key) {
            return None;
        }

        self.buffer
            .get_mut(&key)
            .unwrap()
            .remove_duplicate_and_sort();

        let flits: Vec<Flit> = self
            .buffer
            .get_mut(&key)
            .unwrap()
            .flit_buffer
            .clone()
            .into();

        let tail_flit = flits.last().unwrap();
        assert!(tail_flit.is_tail());

        let data = flits_to_data(&flits);

        Some(GeneralPacket {
            data,
            source_id: tail_flit.get_source_id().unwrap(),
            dest_id: tail_flit.get_dest_id().unwrap(),
            next_id: tail_flit.get_next_id().unwrap(),
            prev_id: tail_flit.get_prev_id().unwrap(),
            packet_id: tail_flit.get_packet_id().unwrap(),
            channel_id: tail_flit.get_channel_id().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::flit::{Flit, HeaderFlit};
    use crate::network::vid::add_to_vid_table;

    #[test]
    fn test_flit_buffer() {
        add_to_vid_table(u32::MAX, "broadcast".to_string());
        let mut flit_buffer = FlitBuffer::new();

        let flit0 = Flit::Header(HeaderFlit {
            channel_id: 0,
            packet_id: 0,
            dest_id: "".to_string(),
            source_id: "".to_string(),
            prev_id: "".to_string(),
            next_id: "".to_string(),
            flits_len: 0,
        });
        flit_buffer.push(flit0.clone());

        let flit1 = Flit::Header(HeaderFlit {
            channel_id: 0,
            packet_id: 0,
            dest_id: "".to_string(),
            source_id: "".to_string(),
            prev_id: "".to_string(),
            next_id: "".to_string(),
            flits_len: 0,
        });
        flit_buffer.push(flit1.clone());

        let flit2 = Flit::Header(HeaderFlit {
            channel_id: 0,
            packet_id: 0,
            dest_id: "".to_string(),
            source_id: "".to_string(),
            prev_id: "".to_string(),
            next_id: "".to_string(),
            flits_len: 0,
        });

        flit_buffer.push(flit2.clone());

        assert_eq!(flit_buffer.pop(), Some(flit0));
        assert_eq!(flit_buffer.pop(), Some(flit1));
        assert_eq!(flit_buffer.pop(), Some(flit2));
        assert_eq!(flit_buffer.pop(), None);
    }

    #[test]
    fn test_remove_duplicate_and_sort() {
        add_to_vid_table(u32::MAX, "broadcast".to_string());
        let mut flit_buffer = FlitBuffer::new();

        let header_flit = Flit::Header(HeaderFlit {
            channel_id: 0,
            packet_id: 0,
            dest_id: "".to_string(),
            source_id: "".to_string(),
            prev_id: "".to_string(),
            next_id: "".to_string(),
            flits_len: 0,
        });
        flit_buffer.push(header_flit.clone());
        flit_buffer.push(header_flit.clone());
        flit_buffer.remove_duplicate_and_sort();

        assert_eq!(flit_buffer.pop(), Some(header_flit));
        assert_eq!(flit_buffer.pop(), None);

        for i in [4, 5, 2, 9, 7, 1, 8, 3, 6, 1, 0, 4] {
            let data_flit = Flit::Data(crate::network::flit::DataFlit {
                channel_id: 0,
                packet_id: 0,
                dest_id: "".to_string(),
                source_id: "".to_string(),
                prev_id: "".to_string(),
                next_id: "".to_string(),
                flit_num: i,
                resend_num: 0,
                data: vec![],
            });
            flit_buffer.push(data_flit.clone());
        }

        flit_buffer.remove_duplicate_and_sort();
        assert_eq!(flit_buffer.flit_buffer.len(), 10);
        let mut prev_flit_num = 0;
        while let Some(flit) = flit_buffer.pop() {
            assert!(flit.get_flit_num().unwrap() >= prev_flit_num);
            prev_flit_num = flit.get_flit_num().unwrap();
        }
    }
}
