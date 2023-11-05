use std::collections::HashMap;
use std::sync::Mutex;

cfg_if::cfg_if!(
    if #[cfg(not(test))]
    {
        use once_cell::sync::Lazy;

        static VID_TABLE: Lazy<Mutex<VIDTable>> = Lazy::new(|| Mutex::new(VIDTable::new()));
    }
    else {
        use std::cell::Cell;
        thread_local! {
            static LOCAL_VID_TABLE: Cell<Option<&'static Mutex<VIDTable>>> = Cell::new(None);
        }

        struct VIDTableProxy;

        impl std::ops::Deref for VIDTableProxy {
            type Target = Mutex<VIDTable>;

            #[inline]
            fn deref (&self) -> &Self::Target {
                LOCAL_VID_TABLE.with(|vt| {
                    if vt.get().is_none() {
                        let l = Mutex::new(VIDTable::new());
                        let b = Box::new(l);
                        let static_ref = Box::leak(b);
                        vt.set(Some(static_ref));
                    }
                    vt.get().unwrap()
                }
            )
            }
        }

        static VID_TABLE: VIDTableProxy = VIDTableProxy;
    }
);

// broadcastのvidはu32::MAX，事前にVID_TABLEに登録しておく

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

pub fn add_to_vid_table(vid: u32, pid: &str) {
    let mut table = VID_TABLE.lock().expect("failed to lock VID_TABLE");
    table.v_to_p.insert(vid, pid.to_string());
    table.p_to_v.insert(pid.to_string(), vid);
}

pub fn remove_from_vid_table(vid: u32, pid: &str) {
    let mut table = VID_TABLE.lock().expect("failed to lock VID_TABLE");
    table.v_to_p.remove(&vid);
    table.p_to_v.remove(pid);
}

pub fn update_vid_table(vid: u32, pid: &str) {
    remove_from_vid_table(vid, pid);
    add_to_vid_table(vid, pid);
}

pub fn clear_vid_table() {
    let mut table = VID_TABLE.lock().expect("failed to lock VID_TABLE");
    table.v_to_p.clear();
    table.p_to_v.clear();
}

pub fn get_vid(pid: &str) -> Option<u32> {
    let table = VID_TABLE.lock().expect("failed to lock VID_TABLE");
    eprintln!("{:?}", table);
    table.p_to_v.get(pid).cloned()
}

pub fn get_pid(vid: u32) -> Option<String> {
    let table = VID_TABLE.lock().expect("failed to lock VID_TABLE");
    table.v_to_p.get(&vid).cloned()
}

pub fn print_vid_table() {
    let table = VID_TABLE.lock().expect("failed to lock VID_TABLE");
    eprintln!("VID Table: {:?}", table);
}
