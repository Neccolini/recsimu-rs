use crate::network::vid::get_vid;

#[derive(Debug, Clone)]
pub struct UpdateOption {
    // 隣接ノードのうち接続が切れたノードのID
    pub pids: Vec<String>,
    pub vids: Vec<u32>,
}

impl UpdateOption {
    pub fn new(old_neighbors: Vec<String>, new_neighbors: Vec<String>) -> Self {
        let mut pids = vec![];
        let mut vids = vec![];

        for old_neighbor in old_neighbors.iter() {
            if !new_neighbors.contains(old_neighbor) {
                pids.push(old_neighbor.clone());
                vids.push(get_vid(old_neighbor).unwrap());
            }
        }

        Self { pids, vids }
    }
}
