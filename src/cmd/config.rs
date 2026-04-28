use std::collections::{HashMap, HashSet};

use clap::Subcommand;

use crate::ace::Ace;
use crate::backend::Kind;
use crate::config::ace_toml::{self, AceToml, Trust};
use crate::config::tree::Tree;
use crate::config::{ConfigKey, Scope};

use super::CmdError;

#[derive(Subcommand)]
pub enum Command {
    /// Print the resolved value of a config key
    Get {
        /// Key to read (school, backend, trust, resume, session_prompt, env.KEY)
        key: String,
    },
    /// Set a config value in the appropriate layer
    Set {
        /// Key to write (school, backend, trust, resume, session_prompt, env.KEY)
        key: String,
        /// Value to set
        value: String,
    },
}

pub fn run(ace: &mut Ace, command: Option<Command>) {
    let result = run_inner(ace, command);
    super::exit_on_err(ace, result);
}

fn run_inner(ace: &mut Ace, command: Option<Command>) -> Result<(), CmdError> {
    match command {
        None => show(ace),
        Some(Command::Get { key }) => get(ace, &key),
        Some(Command::Set { key, value }) => set(ace, &key, &value),
    }
}

/// Bare `ace config` — print effective resolved configuration.
///
/// Reads the merged `Resolved` only; does not bind the backend. A stale
/// `backend = "..."` selector (one not in the registry) still prints the
/// configured name without erroring — recovery is the bare `ace` command's job.
fn show(ace: &mut Ace) -> Result<(), CmdError> {
    let r = ace.require_resolved()?;
    let backend_name = r.backend_name.value.clone();

    let session_prompt_value = r.session_prompt.value.clone();
    let env_flat: HashMap<String, String> = r
        .env
        .iter()
        .map(|(k, v)| (k.clone(), v.value.clone()))
        .collect();
    let effective = AceToml {
        school: r.school_specifier.value.clone().unwrap_or_default(),
        backend: Some(backend_name),
        session_prompt: if session_prompt_value.is_empty() {
            None
        } else {
            Some(session_prompt_value)
        },
        env: env_flat,
        trust: r.trust.value,
        resume: if r.resume.value { None } else { Some(false) },
        skip_update: if r.skip_update.value { Some(true) } else { None },
        ..AceToml::default()
    };

    let output = toml::to_string_pretty(&effective)
        .map_err(|e| CmdError::Other(e.to_string()))?;
    print!("{output}");

    let school_output = ace.school()?
        .map(toml::to_string_pretty)
        .transpose()
        .map_err(|e| CmdError::Other(e.to_string()))?;
    if let Some(s) = school_output {
        println!("\n# school.toml");
        print!("{s}");
    }

    Ok(())
}

/// `ace config get <key>` — print resolved value for a single key.
fn get(ace: &mut Ace, key: &str) -> Result<(), CmdError> {
    let config_key = ConfigKey::parse(key)
        .ok_or_else(|| CmdError::Other(format!("unknown config key: {key}")))?;

    let r = ace.require_resolved()?;

    let value = match config_key {
        ConfigKey::School => r.school_specifier.value.clone().unwrap_or_default(),
        ConfigKey::Backend => r.backend_name.value.clone(),
        ConfigKey::Trust => match r.trust.value {
            Trust::Default => "default".to_string(),
            Trust::Auto => "auto".to_string(),
            Trust::Yolo => "yolo".to_string(),
        },
        ConfigKey::Resume => r.resume.value.to_string(),
        ConfigKey::SkipUpdate => r.skip_update.value.to_string(),
        ConfigKey::SessionPrompt => r.session_prompt.value.clone(),
        ConfigKey::Env(ref env_key) => {
            r.env.get(env_key).map(|v| v.value.clone()).unwrap_or_default()
        }
    };

    ace.data(&value);
    Ok(())
}

/// `ace config set <key> <value>` — write a field to the appropriate layer.
fn set(ace: &mut Ace, key: &str, value: &str) -> Result<(), CmdError> {
    let config_key = ConfigKey::parse(key)
        .ok_or_else(|| CmdError::Other(format!("unknown config key: {key}")))?;

    let scope = ace.scope_override()
        .unwrap_or_else(|| Scope::default_for_key(config_key.scope_key()));

    let paths = ace.require_paths()?;
    let target = scope.path_in(&paths);

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut config = ace_toml::load_or_default(target)?;

    match config_key {
        ConfigKey::School => config.school = value.to_string(),
        ConfigKey::Backend => {
            let known = known_backend_names(ace.require_tree()?);
            if !known.contains(value) {
                let mut listed: Vec<&str> = known.iter().map(String::as_str).collect();
                listed.sort();
                return Err(CmdError::Other(format!(
                    "unknown backend: {value} (known: {})",
                    listed.join(", "),
                )));
            }
            config.backend = Some(value.to_string());
        }
        ConfigKey::Trust => {
            let trust = parse_trust(value)?;
            config.trust = trust;
            config.yolo = false; // clear deprecated field
        }
        ConfigKey::Resume => {
            let resume = parse_bool(value)?;
            config.resume = Some(resume);
        }
        ConfigKey::SkipUpdate => {
            config.skip_update = Some(parse_bool(value)?);
        }
        ConfigKey::SessionPrompt => {
            config.session_prompt = Some(value.to_string());
        }
        ConfigKey::Env(env_key) => {
            config.env.insert(env_key, value.to_string());
        }
    }

    ace_toml::save(target, &config)?;
    ace.done(&format!("{key} = {value}"));
    Ok(())
}

/// Names that resolve as a backend selector: built-ins + any `[[backends]]`
/// declarations across school/user/project/local layers. Used by `ace config
/// set backend` for early validation; resolve-time errors still apply.
fn known_backend_names(tree: &Tree) -> HashSet<String> {
    let mut names: HashSet<String> = Kind::ALL.iter().map(|k| k.name().to_string()).collect();
    if let Some(st) = &tree.school {
        for d in &st.backends {
            names.insert(d.name.clone());
        }
    }
    for layer in [&tree.user, &tree.project, &tree.local].iter().filter_map(|o| o.as_ref()) {
        for d in &layer.backends {
            names.insert(d.name.clone());
        }
    }
    names
}

fn parse_trust(value: &str) -> Result<Trust, CmdError> {
    match value {
        "default" => Ok(Trust::Default),
        "auto" => Ok(Trust::Auto),
        "yolo" => Ok(Trust::Yolo),
        _ => Err(CmdError::Other(format!(
            "unknown trust mode: {value} (expected default, auto, yolo)"
        ))),
    }
}

fn parse_bool(value: &str) -> Result<bool, CmdError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(CmdError::Other(format!(
            "expected true or false, got: {value}"
        ))),
    }
}
