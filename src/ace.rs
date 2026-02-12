use crate::config::Config;
use crate::ui::UI;

pub struct Ace {
    config: Config,
    ui: Box<dyn UI>,
}

impl Ace {
    pub fn new(config: Config, ui: Box<dyn UI>) -> Self {
        Self { config, ui }
    }

    pub fn ui(&self) -> &dyn UI {
        &*self.ui
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
