use crate::ace::Ace;
use crate::config::ace_toml::AceToml;

use super::CmdError;

pub async fn run(ace: &mut Ace) {
    let result = run_inner(ace);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace) -> Result<(), CmdError> {
    ace.require_state()?;
    let state = ace.state();

    let effective = AceToml {
        school: state.school_specifier.clone().unwrap_or_default(),
        backend: Some(state.backend),
        role: if state.role.is_empty() { None } else { Some(state.role.clone()) },
        description: if state.description.is_empty() { None } else { Some(state.description.clone()) },
        session_prompt: if state.session_prompt.is_empty() {
            None
        } else {
            Some(state.session_prompt.clone())
        },
        env: state.env.clone(),
    };

    let output = toml::to_string_pretty(&effective)
        .map_err(|e| CmdError::Other(e.to_string()))?;
    print!("{output}");

    if let Some(school) = &ace.state().school {
        let school_output = toml::to_string_pretty(school)
            .map_err(|e| CmdError::Other(e.to_string()))?;
        println!("\n# school.toml");
        print!("{school_output}");
    }

    Ok(())
}
