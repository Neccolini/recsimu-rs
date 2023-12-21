use std::collections::HashMap;
use std::error;
use std::sync::Mutex;

use crate::recsimu_dbg;

cfg_if::cfg_if!(
    if #[cfg(not(test))]
    {
        use once_cell::sync::Lazy;

        static LOG: Lazy<Mutex<Log>> = Lazy::new(|| Mutex::new(Log::new()));
    }
    else {
        use std::cell::Cell;
        thread_local! {
            static LOCAL_LOG: Cell<Option<&'static Mutex<Log>>> = Cell::new(None);
        }

        struct LogProxy;

        impl std::ops::Deref for LogProxy {
            type Target = Mutex<Log>;

            #[inline]
            fn deref (&self) -> &Self::Target {
                LOCAL_LOG.with(|log| {
                    if log.get().is_none() {
                        let l = Mutex::new(Log::new());
                        let b = Box::new(l);
                        let static_ref = Box::leak(b);
                        log.set(Some(static_ref));
                    }
                    log.get().unwrap()
                }
            )
            }
        }

        static LOG: LogProxy = LogProxy;
    }
);

#[derive(Debug, Clone, Default)]
struct Log {
    packets_info: HashMap<String, PacketLog>,
    collision_info: Vec<CollisionInfo>,
}

impl Log {
    fn new() -> Self {
        Self {
            packets_info: HashMap::new(),
            collision_info: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PacketLog {
    packet_id: String,
    from_id: String,
    dest_id: String,
    send_cycle: Option<u32>,
    last_receive_cycle: Option<u32>,
    flits_len: u32,
    route_info: Vec<String>,
    flit_logs: Vec<FlitLog>,
    is_delivered: bool,
    message: String,
    channel_id: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlitLog {
    received_cycle: u32,
    from_id: String,
    dest_id: String,
}

#[derive(Debug, Clone)]
pub struct NewPacketLogInfo {
    pub packet_id: String,
    pub from_id: String,
    pub dest_id: String,
    pub flits_len: u32,
    pub message: String,
    pub channel_id: u8,
}

pub fn post_new_packet_log(
    packet_info: &NewPacketLogInfo,
) -> Result<PacketLog, Box<dyn error::Error>> {
    let id = packet_info.packet_id.clone();

    let packet_log = PacketLog {
        packet_id: packet_info.packet_id.clone(),
        from_id: packet_info.from_id.clone(),
        dest_id: packet_info.dest_id.clone(),
        send_cycle: None,
        last_receive_cycle: None,
        flits_len: packet_info.flits_len,
        route_info: vec![packet_info.from_id.clone()],
        flit_logs: Vec::new(),
        is_delivered: false,
        message: packet_info.message.clone(),
        channel_id: packet_info.channel_id,
    };

    LOG.lock()
        .expect("failed to lock log")
        .packets_info
        .insert(id, packet_log.clone());

    Ok(packet_log)
}

pub struct FlitLogInfo {
    pub received_cycle: u32,
    pub from_id: String,
    pub dest_id: String,
    pub flit_num: u32,
}

impl FlitLogInfo {
    // NewFlitInfoをFlitInfoに変換する
    fn to_flit_log(&self) -> FlitLog {
        FlitLog {
            received_cycle: self.received_cycle,
            from_id: self.from_id.clone(),
            dest_id: self.dest_id.clone(),
        }
    }
}

pub struct UpdatePacketLogInfo {
    pub send_cycle: Option<u32>,
    pub last_receive_cycle: Option<u32>,
    pub route_info: Option<String>,
    pub is_delivered: Option<bool>,
    pub flit_log: Option<FlitLogInfo>,
}

pub fn update_packet_log(
    packet_id: &str,
    update_packet_log: &UpdatePacketLogInfo,
) -> Result<PacketLog, Box<dyn error::Error>> {
    let mut log = LOG.lock().expect("failed to lock log");

    let packet_log = log
        .packets_info
        .get_mut(packet_id)
        .expect("specified packet not found");

    if let Some(send_cycle) = &update_packet_log.send_cycle {
        packet_log.send_cycle = Some(*send_cycle);
    }

    if let Some(last_receive_cycle) = &update_packet_log.last_receive_cycle {
        packet_log.last_receive_cycle = Some(*last_receive_cycle);
    }

    if let Some(route_info) = &update_packet_log.route_info {
        packet_log.route_info.push(route_info.clone());
    }

    if let Some(is_delivered) = &update_packet_log.is_delivered {
        packet_log.is_delivered = *is_delivered;
    }

    if let Some(flit_log) = &update_packet_log.flit_log {
        packet_log.flit_logs.push(flit_log.to_flit_log());
    }

    Ok(packet_log.clone())
}

pub fn get_packet_log(packet_id: &str) -> Option<PacketLog> {
    let log = LOG.lock().expect("failed to lock log");
    log.packets_info.get(packet_id).cloned()
}

pub fn packet_is_received(packet_id: &str) -> bool {
    let log = LOG.lock().expect("failed to lock log");
    log.packets_info
        .get(packet_id)
        .expect("specified packet not found")
        .last_receive_cycle
        .is_some()
}

pub fn get_all_log() -> Vec<PacketLog> {
    let log = LOG.lock().expect("failed to lock log");

    log.packets_info.values().cloned().collect()
}

pub fn clear_log() {
    let mut log = LOG.lock().expect("failed to lock log");
    log.packets_info.clear();
}

#[allow(unused)]
#[derive(Debug, Clone)]
struct CollisionInfo {
    cycle: u32,
    from_ids: Vec<String>,
    dest_id: String,
}

pub struct NewCollisionInfo {
    pub cycle: u32,
    pub from_ids: Vec<String>,
    pub dest_id: String,
}

pub fn post_collision_info(info: &NewCollisionInfo) {
    let mut log = LOG.lock().expect("failed to lock log");

    let collision_info = CollisionInfo {
        cycle: info.cycle,
        from_ids: info.from_ids.clone(),
        dest_id: info.dest_id.clone(),
    };

    log.collision_info.push(collision_info);
}

// ログの集計
pub fn aggregate_log(begin: u32, end: u32) -> HashMap<String, f64> {
    // 必要な情報は，パケットの送信にかかった平均サイクル数
    let log = LOG.lock().expect("failed to lock log");

    let mut sum = 0.0;
    let mut count = 0;
    let mut undelivered_count = 0;

    let mut packet_count = 0;
    let mut flits_count = 0;

    for (_, packet_log) in log.packets_info.iter() {
        if packet_log.send_cycle.is_none() {
            continue;
        }

        if packet_log.send_cycle.unwrap() < begin || packet_log.send_cycle.unwrap() >= end {
            continue;
        }
        if packet_log.is_delivered {
            if packet_log.last_receive_cycle.unwrap() < packet_log.send_cycle.unwrap() {
                panic!("{:?}", packet_log);
            }

            sum += (packet_log.last_receive_cycle.unwrap() - packet_log.send_cycle.unwrap()) as f64
                / packet_log.flits_len as f64;
            count += 1;

            recsimu_dbg!("{:?}", packet_log);
        } else {
            undelivered_count += 1;
        }
        packet_count += 1;
        flits_count += packet_log.flits_len;
    }

    let mut result = HashMap::new();

    // assert!(jack_max_cycle == 0);

    result.insert("average_cycle".to_string(), sum / count as f64);
    result.insert("undelivered_packets".to_string(), undelivered_count as f64);
    result.insert("total_packets".to_string(), packet_count as f64);
    result.insert("total_flits".to_string(), flits_count as f64);
    result.insert(
        "average_flits_len".to_string(),
        flits_count as f64 / packet_count as f64,
    );
    result.insert(
        "collision_count".to_string(),
        log.collision_info.len() as f64,
    );

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_new_packet_log() {
        let packet_info = NewPacketLogInfo {
            packet_id: "packet_id".to_string(),
            from_id: "from_id".to_string(),
            dest_id: "dest_id".to_string(),
            flits_len: 2,
            message: "test".to_string(),
            channel_id: 0,
        };
        let packet_log = post_new_packet_log(&packet_info).unwrap();
        assert_eq!(packet_log.packet_id, "packet_id");
        assert_eq!(packet_log.from_id, "from_id");
        assert_eq!(packet_log.dest_id, "dest_id");
        assert_eq!(packet_log.flits_len, 2);
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
            dest_id: "dest_id".to_string(),
            flits_len: 3,
            message: "test".to_string(),
            channel_id: 0,
        };
        let packet_log = post_new_packet_log(&packet_info).unwrap();

        let update_packet_log_info = UpdatePacketLogInfo {
            send_cycle: None,
            last_receive_cycle: Some(1),
            route_info: Some("route_info".to_string()),
            is_delivered: Some(true),
            flit_log: None,
        };
        let packet_log = update_packet_log(&packet_log.packet_id, &update_packet_log_info).unwrap();
        assert_eq!(packet_log.packet_id, "packet_id");
        assert_eq!(packet_log.from_id, "from_id");
        assert_eq!(packet_log.dest_id, "dest_id");
        assert_eq!(packet_log.flits_len, 3);
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
            dest_id: "dest_id".to_string(),
            flits_len: 1,
            message: "test".to_string(),
            channel_id: 0,
        };
        let packet_log = post_new_packet_log(&packet_info).unwrap();

        let get_packet_log = get_packet_log(&packet_log.packet_id).unwrap();
        assert_eq!(get_packet_log.packet_id, "packet_id");
        assert_eq!(get_packet_log.from_id, "from_id");
        assert_eq!(get_packet_log.dest_id, "dest_id");
        assert_eq!(get_packet_log.flits_len, 1);
        assert_eq!(get_packet_log.last_receive_cycle, None);
        assert_eq!(get_packet_log.route_info, vec!["from_id"]);
        assert_eq!(get_packet_log.flit_logs, Vec::<FlitLog>::new());
        assert_eq!(get_packet_log.is_delivered, false);
    }
}
