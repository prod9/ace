use crate::state::State;

pub struct Session<'a> {
    pub state: &'a mut State,
}
