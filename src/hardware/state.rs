#[derive(PartialEq)]
pub enum State {
    Idle(Idle),
    Sending,
    Receiving,
    Waiting,
    WaitingAck,
    ReplyAck,
}

impl Default for State {
    fn default() -> Self {
        Self::Idle(Idle::default())
    }
}

// Idle state
#[derive(PartialEq)]
pub struct Idle {
    pub cycles: u32,
}

impl Idle {
    pub fn new() -> Self {
        Self { cycles: 0 }
    }

    pub fn next(&mut self) {}
}
impl Default for Idle {
    fn default() -> Self {
        Self::new()
    }
}
