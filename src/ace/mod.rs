use crate::session::Session;
use crate::state::State;
use crate::ui::{StdoutUI, UI};

pub struct Ace {
    state: State,
    ui: Box<dyn UI>,
}

impl Ace {
    pub fn new() -> Self {
        let state = State::empty();
        let ui = Box::new(StdoutUI);
        Self { state, ui }
    }

    pub fn session(&mut self) -> Session<'_> {
        Session {
            state: &mut self.state,
            ui: &*self.ui,
        }
    }
}
