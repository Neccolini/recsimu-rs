#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Coordinator,
    Router,
    EndDevice,
    UserType(String),
}
