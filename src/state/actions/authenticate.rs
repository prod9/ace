use crate::ace::Ace;
use crate::config::school_toml::ServiceDecl;
use super::prepare::PrepareError;

pub struct Authenticate<'a> {
    pub service: &'a ServiceDecl,
}

impl Authenticate<'_> {
    pub async fn run(&self, _ace: &mut Ace) -> Result<(), PrepareError> {
        // TODO: run PKCE OAuth flow for service
        Ok(())
    }
}
