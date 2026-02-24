use std::path::Path;

use crate::config;
use crate::config::ConfigError;
use crate::events::{Event, EventSink, OwnedSink};
use crate::state::State;
use crate::term_ui::sink::TermSink;

pub struct Ace {
    pub state: State,
    sink: OwnedSink,
}

impl Ace {
    pub fn term_sink() -> Box<dyn EventSink> {
        Box::new(TermSink::new())
    }

    pub fn new(sink: Box<dyn EventSink>) -> Self {
        Self { state: State::empty(), sink: OwnedSink::new(sink) }
    }

    pub fn load(project_dir: &Path, sink: Box<dyn EventSink>) -> Result<Self, ConfigError> {
        let paths = config::paths::resolve(project_dir)?;
        let tree = config::tree::Tree::load(&paths)?;
        let state = State::resolve(tree);
        Ok(Self { state, sink: OwnedSink::new(sink) })
    }

    pub fn with_state(state: State, sink: Box<dyn EventSink>) -> Self {
        Self { state, sink: OwnedSink::new(sink) }
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

    pub fn error(&mut self, msg: &str) {
        self.sink.handle(Event::Error(msg.to_string()));
    }

    pub fn data(&mut self, msg: &str) {
        self.sink.handle(Event::Data(msg.to_string()));
    }
}
