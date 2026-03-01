use std::path::Path;

use crate::config;
use crate::config::ConfigError;
use crate::events::OutputMode;
use crate::git::Git;
use crate::state::State;
use crate::term_ui::sink::EventSink;

pub struct Ace {
    pub state: State,
    sink: EventSink,
    mode: OutputMode,
}

impl Ace {
    pub fn new(mode: OutputMode) -> Self {
        Self { state: State::empty(), sink: EventSink::new(mode), mode }
    }

    pub fn load(project_dir: &Path, mode: OutputMode) -> Result<Self, ConfigError> {
        let paths = config::paths::resolve(project_dir)?;
        let tree = config::tree::Tree::load(&paths)?;
        let state = State::resolve(tree);
        Ok(Self { state, sink: EventSink::new(mode), mode })
    }

    pub fn with_state(state: State, mode: OutputMode) -> Self {
        Self { state, sink: EventSink::new(mode), mode }
    }

    pub fn output_mode(&self) -> OutputMode {
        self.mode
    }

    pub fn git<'a>(&self, repo: &'a Path) -> Git<'a> {
        Git::new(repo, self.mode)
    }

    pub fn progress(&mut self, msg: &str) {
        self.sink.progress(msg);
    }

    pub fn done(&mut self, msg: &str) {
        self.sink.done(msg);
    }

    pub fn warn(&mut self, msg: &str) {
        self.sink.warn(msg);
    }

    pub fn error(&mut self, msg: &str) {
        self.sink.error(msg);
    }

    pub fn data(&mut self, msg: &str) {
        self.sink.data(msg);
    }
}
