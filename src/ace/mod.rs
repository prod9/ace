use std::path::Path;

use crate::config;
use crate::events::{EventSink, NoopSink, OwnedSink};
use crate::session::Session;
use crate::state::State;
use crate::term_ui::sink::TermSink;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("{0}")]
    Path(#[from] config::paths::PathError),
    #[error("{0}")]
    Config(#[from] config::tree::LoadError),
}

pub struct Ace {
    state: State,
    sink: Option<OwnedSink>,
}

impl Ace {
    pub fn term_sink() -> Box<dyn EventSink> {
        Box::new(TermSink::new())
    }

    pub fn new(sink: Box<dyn EventSink>) -> Self {
        let state = State::empty();
        Self { state, sink: Some(OwnedSink::new(sink)) }
    }

    pub fn load(project_dir: &Path, sink: Box<dyn EventSink>) -> Result<Self, LoadError> {
        let paths = config::paths::resolve(project_dir)?;
        let tree = config::tree::Tree::load(&paths)?;
        let state = State::resolve(tree);
        Ok(Self { state, sink: Some(OwnedSink::new(sink)) })
    }

    pub fn with_state(state: State, sink: Box<dyn EventSink>) -> Self {
        Self { state, sink: Some(OwnedSink::new(sink)) }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn session(&mut self) -> Session<'_> {
        let sink = self.sink.take()
            .unwrap_or_else(|| OwnedSink::new(Box::new(NoopSink)));
        Session::new(&mut self.state, sink)
    }
}
