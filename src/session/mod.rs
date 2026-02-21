use crate::state::State;
use crate::ui::UI;

pub struct Session<'a> {
    pub state: &'a mut State,
    pub ui: &'a dyn UI,
}
