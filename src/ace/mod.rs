use crate::session::Session;
use crate::state::State;

pub struct Ace {
    state: State,
}

impl Ace {
    pub fn new() -> Self {
        let state = State::empty();
        Self { state }
    }

    pub fn session(&mut self) -> Session<'_> {
        Session {
            state: &mut self.state,
        }
    }
}
