use crate::config::school_toml::ServiceDecl;
use crate::session::Session;
use crate::state::setup::SetupError;

pub struct Authenticate<'a> {
    pub service: &'a ServiceDecl,
}

impl Authenticate<'_> {
    pub async fn run(&self, _session: &mut Session<'_>) -> Result<(), SetupError> {
        // TODO: run PKCE OAuth flow for service
        Ok(())
    }
}
