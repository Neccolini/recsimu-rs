use crate::network::protocols::default::DefaultProtocol;

pub enum RoutingFunction {
    DefaultFunction(DefaultProtocol),
    RoutingFunctionVer202306,
}

impl RoutingFunction {
    pub fn new(rf_kind: String) -> Self {
        #[allow(clippy::match_single_binding)]
        match rf_kind.as_str() {
            _ => RoutingFunction::DefaultFunction(DefaultProtocol::new()),
        }
    }
    pub fn send_packet() {}
}

impl Default for RoutingFunction {
    fn default() -> Self {
        Self::new("".to_string())
    }
}
