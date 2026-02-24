pub mod prompt;

use crate::events::{Event, OwnedSink};
use crate::state::State;

pub struct Session<'a> {
    pub state: &'a mut State,
    sink: OwnedSink,
}

impl<'a> Session<'a> {
    pub fn new(state: &'a mut State, sink: OwnedSink) -> Self {
        Self { state, sink }
    }

    pub fn progress(&mut self, msg: &str) {
        self.sink.handle(Event::Progress(msg.to_string()));
    }

    pub fn done(&mut self, msg: &str) {
        self.sink.handle(Event::Done(msg.to_string()));
    }

    pub fn warn(&mut self, msg: &str) {
        self.sink.handle(Event::Warn(msg.to_string()));
    }
}
