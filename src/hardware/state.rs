#[derive(Default)]
pub struct NodeState {
    state: State,
    resend_times: u8,
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            resend_times: 0,
        }
    }

    // transitionのvalidateを行う
    pub fn next(&mut self, new_state: &State) {
        match self.state {
            State::Idle => match new_state {
                State::Sending | State::Receiving => {
                    self.state = new_state.clone();
                }
                _ => {
                    panic!("Invalid state transition from idle to {new_state:?}");
                }
            },

            State::Sending => match new_state {
                State::Waiting(_) => {
                    self.state = new_state.clone();
                }
                _ => {
                    panic!("Invalid state transition from sending to {new_state:?}");
                }
            },

            State::Receiving => match new_state {
                State::ReplyAck => {
                    self.state = new_state.clone();
                }
                State::Idle => {
                    self.state = new_state.clone();
                }
                _ => {
                    panic!("Invalid state transition from receiving to {new_state:?}");
                }
            },

            State::Waiting(ref mut cur_waiting) => match new_state {
                State::Waiting(next_waiting) => {
                    assert!(
                        cur_waiting.remaining_cycles == next_waiting.remaining_cycles + 1,
                        "Invalid state transition from waiting to waiting: cycle mismatch"
                    );

                    assert!(cur_waiting.remaining_cycles > 0, "Invalid state transition from waiting to waiting: remaining_cycles must be positive");

                    self.state = new_state.clone();
                }
                State::Idle | State::Sending | State::Receiving => {
                    self.state = new_state.clone();
                }
                _ => {
                    panic!("Invalid state transition from waiting to {new_state:?}");
                }
            },

            State::ReplyAck => match new_state {
                State::Idle => {
                    self.state = new_state.clone();
                }
                _ => {
                    panic!("Invalid state transition from reply_ack to {new_state:?}");
                }
            },
        }
    }

    pub fn get(&self) -> &State {
        &self.state
    }

    pub fn set_resend_times(&mut self, times: u8) {
        self.resend_times = times;
    }

    pub fn get_resend_times(&self) -> u8 {
        self.resend_times
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub enum State {
    #[default]
    Idle,
    Sending,
    Receiving,
    Waiting(Waiting),
    ReplyAck,
}

impl State {
    pub fn waiting_state(remaining_cycles: u32) -> Self {
        Self::Waiting(Waiting { remaining_cycles })
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Waiting {
    pub remaining_cycles: u32,
}
