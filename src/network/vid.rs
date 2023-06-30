use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static VID_TABLE: Lazy<Mutex<VIDTable>> = Lazy::new(|| Mutex::new(VIDTable::new()));

#[derive(Debug, Default)]
struct VIDTable {
    v_to_p: HashMap<u32, String>,
    p_to_v: HashMap<String, u32>,
}

impl VIDTable {
    fn new() -> Self {
        Self {
            v_to_p: HashMap::new(),
            p_to_v: HashMap::new(),
        }
    }
}

pub fn add_to_vid_table(vid: u32, pid: String) {
    let mut table = VID_TABLE.lock().unwrap();
    table.v_to_p.insert(vid, pid.clone());
    table.p_to_v.insert(pid, vid);
}

pub fn remove_from_vid_table(vid: u32, pid: String) {
    let mut table = VID_TABLE.lock().unwrap();
    table.v_to_p.remove(&vid);
    table.p_to_v.remove(&pid);
}

pub fn get_vid(pid: String) -> Option<u32> {
    let table = VID_TABLE.lock().unwrap();
    table.p_to_v.get(&pid).cloned()
}

pub fn get_pid(vid: u32) -> Option<String> {
    let table = VID_TABLE.lock().unwrap();
    table.v_to_p.get(&vid).cloned()
}

pub fn print_vid_table() {
    let table = VID_TABLE.lock().unwrap();
    println!("VID Table: {:?}", table);
}
