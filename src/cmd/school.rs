use clap::Subcommand;

use crate::ace::Ace;
use crate::config::school_toml::ServiceDecl;
use crate::state::actions::add_service::AddService;
use crate::state::actions::school_init::SchoolInit;
use crate::state::actions::school_update::{SchoolUpdate, SchoolUpdateResult};
use crate::term_ui::{Tui, Workflow};

use super::CmdError;

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new school repository
    Init {
        /// School display name
        #[arg(long)]
        name: Option<String>,
        /// Overwrite existing school.toml
        #[arg(long)]
        force: bool,
    },
    /// Re-fetch all imported skills from their sources
    Update,
    /// Add a service (OAuth provider) to school.toml
    AddService {
        /// Service name (e.g. "github")
        #[arg(long)]
        name: Option<String>,
        /// OAuth authorize URL
        #[arg(long)]
        authorize_url: Option<String>,
        /// OAuth token URL
        #[arg(long)]
        token_url: Option<String>,
        /// OAuth client ID
        #[arg(long)]
        client_id: Option<String>,
        /// Comma-separated scopes
        #[arg(long, value_delimiter = ',')]
        scopes: Option<Vec<String>>,
    },
}

pub async fn run(ace: &mut Ace, command: Command) {
    match command {
        Command::Init { name, force } => {
            let result = run_init(ace, name, force);
            super::exit_on_err(ace, result);
        }
        Command::Update => {
            let result = run_update(ace);
            super::exit_on_err(ace, result);
        }
        Command::AddService { name, authorize_url, token_url, client_id, scopes } => {
            let result = run_add_service(ace, name, authorize_url, token_url, client_id, scopes);
            super::exit_on_err(ace, result);
        }
    }
}

fn run_init(ace: &mut Ace, name: Option<String>, force: bool) -> Result<(), CmdError> {
    match name {
        Some(name) => {
            let project_dir = ace.project_dir().to_path_buf();
            SchoolInit {
                name: &name,
                project_dir: &project_dir,
                force,
            }
            .run(ace)?;
        }
        None => {
            Tui::new(ace).run(Workflow::SchoolInit { force })?;
        }
    }
    Ok(())
}

fn run_update(ace: &mut Ace) -> Result<(), CmdError> {
    let school_root = ace.require_school()?.root.clone();

    let result = SchoolUpdate { school_root: &school_root }.run(ace)?;
    match result {
        SchoolUpdateResult::NoImports => ace.warn("no imports to update"),
        SchoolUpdateResult::Updated { .. } => {}
    }
    Ok(())
}

fn run_add_service(
    ace: &mut Ace,
    name: Option<String>,
    authorize_url: Option<String>,
    token_url: Option<String>,
    client_id: Option<String>,
    scopes: Option<Vec<String>>,
) -> Result<(), CmdError> {
    let school_root = ace.require_school()?.root.clone();

    if let (Some(name), Some(authorize_url), Some(token_url), Some(client_id)) =
        (name, authorize_url, token_url, client_id)
    {
        let service = ServiceDecl {
            name,
            authorize_url,
            token_url,
            client_id,
            scopes: scopes.unwrap_or_default(),
        };
        AddService { school_root: &school_root, service }.run(ace)?;
    } else {
        Tui::new(ace).run(Workflow::AddService { school_root })?;
    }
    Ok(())
}
