use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ace::Ace;

#[derive(Debug, thiserror::Error)]
pub enum SchoolProposeError {
    #[error("no school linked, run ace setup first")]
    NoSchool,
    #[error("school cache not found at {0}")]
    NoCacheDir(String),
    #[error("no changes to propose")]
    NoChanges,
    #[error("git: {0}")]
    Git(String),
    #[error("github api: {0}")]
    Api(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] crate::config::ConfigError),
}

pub struct SchoolPropose<'a> {
    pub project_dir: &'a Path,
    pub token: &'a str,
}

impl SchoolPropose<'_> {
    pub fn run(&self, ace: &mut Ace) -> Result<String, SchoolProposeError> {
        let specifier = ace
            .state
            .school_specifier
            .as_deref()
            .ok_or(SchoolProposeError::NoSchool)?;

        let school_paths = crate::config::school_paths::resolve(self.project_dir, specifier)?;
        let cache = school_paths
            .cache
            .as_deref()
            .ok_or_else(|| SchoolProposeError::NoCacheDir("embedded school".to_string()))?;

        if !cache.join(".git").exists() {
            return Err(SchoolProposeError::NoCacheDir(cache.display().to_string()));
        }

        let git = ace.git(cache);
        if !git.is_dirty().map_err(|e| SchoolProposeError::Git(e.to_string()))? {
            return Err(SchoolProposeError::NoChanges);
        }

        let (owner, repo) = parse_owner_repo(specifier)?;

        let branch = format!("ace/propose-{}", timestamp());
        let git = ace.git(cache);
        git.checkout_new_branch(&branch)
            .map_err(|e| SchoolProposeError::Git(e.to_string()))?;
        git.add_all()
            .map_err(|e| SchoolProposeError::Git(e.to_string()))?;
        git.commit("Propose school changes from ace")
            .map_err(|e| SchoolProposeError::Git(e.to_string()))?;
        git.push_new_branch("origin", &branch)
            .map_err(|e| SchoolProposeError::Git(e.to_string()))?;

        let pr_url = create_pr(owner, repo, &branch, self.token)?;

        // Reset cache back to origin/main so future updates work cleanly
        let git = ace.git(cache);
        git.checkout("main")
            .map_err(|e| SchoolProposeError::Git(e.to_string()))?;
        git.reset_hard("origin/main")
            .map_err(|e| SchoolProposeError::Git(e.to_string()))?;

        Ok(pr_url)
    }
}

fn parse_owner_repo(specifier: &str) -> Result<(&str, &str), SchoolProposeError> {
    let repo_part = specifier.split_once(':').map_or(specifier, |(repo, _)| repo);
    repo_part
        .split_once('/')
        .ok_or_else(|| SchoolProposeError::Git(format!("invalid specifier: {specifier}")))
}

#[derive(Serialize)]
struct CreatePrRequest<'a> {
    title: &'a str,
    head: &'a str,
    base: &'a str,
    body: &'a str,
}

#[derive(Deserialize)]
struct CreatePrResponse {
    html_url: String,
}

fn create_pr(owner: &str, repo: &str, branch: &str, token: &str) -> Result<String, SchoolProposeError> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/pulls");

    let request_body = CreatePrRequest {
        title: "Propose school changes",
        head: branch,
        base: "main",
        body: "Changes proposed via `ace school propose`.",
    };

    let response: CreatePrResponse = ureq::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "ace")
        .send_json(&request_body)
        .map_err(|e| SchoolProposeError::Api(e.to_string()))?
        .body_mut()
        .read_json()
        .map_err(|e| SchoolProposeError::Api(format!("parse response: {e}")))?;

    Ok(response.html_url)
}

fn timestamp() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_owner_repo_simple() {
        let (owner, repo) = parse_owner_repo("prod9/school").expect("should parse");
        assert_eq!(owner, "prod9");
        assert_eq!(repo, "school");
    }

    #[test]
    fn parse_owner_repo_with_path() {
        let (owner, repo) = parse_owner_repo("prod9/mono:school").expect("should parse");
        assert_eq!(owner, "prod9");
        assert_eq!(repo, "mono");
    }

    #[test]
    fn parse_owner_repo_invalid() {
        assert!(parse_owner_repo("noslash").is_err());
    }
}
