use crate::config::Config;
use crate::config::school::School;

pub struct Session<'a> {
    pub config: &'a Config,
    pub school: &'a School,
}
