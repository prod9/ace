use std::path::Path;

use crate::config;
use crate::session::Session;
use crate::state::State;

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("{0}")]
    Path(#[from] config::paths::PathError),
    #[error("{0}")]
    Config(#[from] config::tree::LoadError),
}

pub struct Ace {
    state: State,
}

impl Ace {
    pub fn new() -> Self {
        let state = State::empty();
        Self { state }
    }

    pub fn load(project_dir: &Path) -> Result<Self, LoadError> {
        let paths = config::paths::resolve(project_dir)?;
        let tree = config::tree::Tree::load(&paths)?;
        let state = State::resolve(tree);
        Ok(Self { state })
    }

    pub fn with_state(state: State) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn session(&mut self) -> Session<'_> {
        Session {
            state: &mut self.state,
        }
    }
}
