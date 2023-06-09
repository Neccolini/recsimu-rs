use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::error;
use std::sync::Mutex;

static LOG: Lazy<Mutex<Log>> = Lazy::new(|| Mutex::new(Log::new()));

#[derive(Debug, Clone, Default)]
pub struct Log {
    pub packets_info: HashMap<String, PacketLog>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            packets_info: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketLog {
    pub packet_id: String,
    pub from_id: String,
    pub dist_id: String,
    pub send_cycle: u32,
    pub last_receive_cycle: Option<u32>,
    pub route_info: Vec<String>,
    pub flit_logs: Vec<FlitLog>,
    pub is_delivered: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlitLog {
    pub received_cycle: u32,
    pub from_id: String,
    pub dist_id: String,
}

#[derive(Debug, Clone)]
pub struct NewPacketLogInfo {
    pub packet_id: String,
    pub from_id: String,
    pub dist_id: String,
    pub send_cycle: u32,
}

pub fn post_new_packet_log(
    packet_info: NewPacketLogInfo,
) -> Result<PacketLog, Box<dyn error::Error>> {
    let id = packet_info.packet_id.clone();
    let packet_log = PacketLog {
        packet_id: packet_info.packet_id,
        from_id: packet_info.from_id.clone(),
        dist_id: packet_info.dist_id,
        send_cycle: packet_info.send_cycle,
        last_receive_cycle: None,
        route_info: vec![packet_info.from_id],
        flit_logs: Vec::new(),
        is_delivered: false,
    };
    LOG.lock()
        .unwrap()
        .packets_info
        .insert(id, packet_log.clone());
    Ok(packet_log)
}

pub struct FlitLogInfo {
    pub received_cycle: u32,
    pub from_id: String,
    pub dist_id: String,
    pub flit_num: u32,
}

impl FlitLogInfo {
    // NewFlitInfoをFlitInfoに変換する
    pub fn to_flit_log(&self) -> FlitLog {
        FlitLog {
            received_cycle: self.received_cycle,
            from_id: self.from_id.clone(),
            dist_id: self.dist_id.clone(),
        }
    }
}

pub struct UpdatePacketLogInfo {
    pub last_receive_cycle: Option<u32>,
    pub route_info: Option<String>,
    pub is_delivered: Option<bool>,
    pub flit_log: Option<FlitLogInfo>,
}

pub fn update_packet_log(
    packet_id: String,
    update_packet_log: UpdatePacketLogInfo,
) -> Result<PacketLog, Box<dyn error::Error>> {
    let mut log = LOG.lock().unwrap();
    let packet_log = log.packets_info.get_mut(&packet_id).unwrap();

    if let Some(last_receive_cycle) = &update_packet_log.last_receive_cycle {
        packet_log.last_receive_cycle = Some(*last_receive_cycle);
    }
    if let Some(route_info) = &update_packet_log.route_info {
        packet_log.route_info.push(route_info.clone());
    }
    if let Some(is_delivered) = &update_packet_log.is_delivered {
        packet_log.is_delivered = *is_delivered;
    }

    Ok(packet_log.clone())
}

pub fn get_packet_log(packet_id: &String) -> Option<PacketLog> {
    let log = LOG.lock().unwrap();
    log.packets_info.get(packet_id).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_new_packet_log() {
        let packet_info = NewPacketLogInfo {
            packet_id: "packet_id".to_string(),
            from_id: "from_id".to_string(),
            dist_id: "dist_id".to_string(),
            send_cycle: 0,
        };
        let packet_log = post_new_packet_log(packet_info).unwrap();
        assert_eq!(packet_log.packet_id, "packet_id");
        assert_eq!(packet_log.from_id, "from_id");
        assert_eq!(packet_log.dist_id, "dist_id");
        assert_eq!(packet_log.send_cycle, 0);
        assert_eq!(packet_log.last_receive_cycle, None);
        assert_eq!(packet_log.route_info, vec!["from_id"]);
        assert_eq!(packet_log.flit_logs, Vec::<FlitLog>::new());
        assert_eq!(packet_log.is_delivered, false);
    }

    #[test]
    fn test_update_packet_log() {
        let packet_info = NewPacketLogInfo {
            packet_id: "packet_id".to_string(),
            from_id: "from_id".to_string(),
            dist_id: "dist_id".to_string(),
            send_cycle: 0,
        };
        let packet_log = post_new_packet_log(packet_info).unwrap();

        let update_packet_log_info = UpdatePacketLogInfo {
            last_receive_cycle: Some(1),
            route_info: Some("route_info".to_string()),
            is_delivered: Some(true),
            flit_log: None,
        };
        let packet_log =
            update_packet_log(packet_log.packet_id.clone(), update_packet_log_info).unwrap();
        assert_eq!(packet_log.packet_id, "packet_id");
        assert_eq!(packet_log.from_id, "from_id");
        assert_eq!(packet_log.dist_id, "dist_id");
        assert_eq!(packet_log.send_cycle, 0);
        assert_eq!(packet_log.last_receive_cycle, Some(1));
        assert_eq!(packet_log.route_info, vec!["from_id", "route_info"]);
        assert_eq!(packet_log.flit_logs, Vec::<FlitLog>::new());
        assert_eq!(packet_log.is_delivered, true);
    }

    #[test]
    fn test_get_packet_log() {
        let packet_info = NewPacketLogInfo {
            packet_id: "packet_id".to_string(),
            from_id: "from_id".to_string(),
            dist_id: "dist_id".to_string(),
            send_cycle: 0,
        };
        let packet_log = post_new_packet_log(packet_info).unwrap();

        let get_packet_log = get_packet_log(&packet_log.packet_id).unwrap();
        assert_eq!(get_packet_log.packet_id, "packet_id");
        assert_eq!(get_packet_log.from_id, "from_id");
        assert_eq!(get_packet_log.dist_id, "dist_id");
        assert_eq!(get_packet_log.send_cycle, 0);
        assert_eq!(get_packet_log.last_receive_cycle, None);
        assert_eq!(get_packet_log.route_info, vec!["from_id"]);
        assert_eq!(get_packet_log.flit_logs, Vec::<FlitLog>::new());
        assert_eq!(get_packet_log.is_delivered, false);
    }
}
