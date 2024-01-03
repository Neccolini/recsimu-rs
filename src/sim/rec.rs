use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RecTable {
    pub(crate) table: HashMap<u32, Update>, // cycle, update
}

#[derive(Clone, Debug, Deserialize)]
pub struct Update {
    pub(crate) new_neighbors: HashMap<String, Vec<String>>,
}
