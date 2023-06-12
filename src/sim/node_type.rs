#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Coordinator,
    Router,
    EndDevice,
    UserType(String),
}

impl NodeType {
    pub fn new(node_type: &str) -> Self {
        match node_type {
            "coordinator" => NodeType::Coordinator,
            "router" => NodeType::Router,
            "end_device" => NodeType::EndDevice,
            _ => NodeType::UserType(node_type.to_string()),
        }
    }
}
