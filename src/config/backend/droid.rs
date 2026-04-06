use std::collections::HashSet;
use std::process::Command;

use super::{McpDecl, McpStatus};

/// Check if DROID is ready: ~/.factory/settings.json exists.
pub(super) fn is_ready() -> bool {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };
    std::path::Path::new(&home)
        .join(".factory/settings.json")
        .exists()
}

pub(super) fn mcp_list() -> HashSet<String> {
    // TODO: Parse ~/.factory/mcp.json when format is confirmed.
    HashSet::new()
}

pub(super) fn mcp_add(entry: &McpDecl) -> Result<(), String> {
    let args = build_mcp_add_args(entry);

    let output = Command::new("droid")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
}

pub(super) fn mcp_remove(name: &str) -> Result<(), String> {
    let args = build_mcp_remove_args(name);

    let output = Command::new("droid")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
}

fn build_mcp_remove_args(name: &str) -> Vec<String> {
    vec![
        "mcp".to_string(),
        "remove".to_string(),
        name.to_string(),
    ]
}

pub(super) fn mcp_check(names: &[String]) -> Vec<McpStatus> {
    let prompt = format!(
        "You have MCP servers registered. For each of the following, call any tool to verify \
         it responds. Reply with only a JSON array: [{{\"name\":\"...\",\"ok\":true/false}}]. \
         Servers: {}",
        names.join(", ")
    );

    let output = Command::new("droid")
        .args(["exec", &prompt, "-o", "json"])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_check_output(&stdout)
}

/// Parse Droid's `{"type":"result","result":"..."}` envelope.
fn parse_check_output(output: &str) -> Vec<McpStatus> {
    let parsed: serde_json::Value = match serde_json::from_str(output) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    if parsed.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false) {
        return Vec::new();
    }

    match parsed.get("result") {
        Some(serde_json::Value::String(s)) => super::parse_status_array(s),
        Some(serde_json::Value::Array(_)) => {
            let json = parsed["result"].to_string();
            super::parse_status_array(&json)
        }
        _ => Vec::new(),
    }
}

/// Build CLI args for `droid mcp add <name> <url> --type http [--header "K: V"]`.
fn build_mcp_add_args(entry: &McpDecl) -> Vec<String> {
    let mut args = vec![
        "mcp".to_string(),
        "add".to_string(),
        entry.name.clone(),
        entry.url.clone(),
        "--type".to_string(),
        "http".to_string(),
    ];

    let mut headers: Vec<(&String, &String)> = entry.headers.iter().collect();
    headers.sort_by_key(|(k, _)| k.as_str());

    for (key, value) in headers {
        args.push("--header".to_string());
        args.push(format!("{key}: {value}"));
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_args_basic() {
        let entry = McpDecl {
            name: "linear".to_string(),
            url: "https://mcp.linear.app/mcp".to_string(),
            headers: std::collections::HashMap::new(),
            instructions: String::new(),
        };

        let args = build_mcp_add_args(&entry);
        assert_eq!(
            args,
            vec!["mcp", "add", "linear", "https://mcp.linear.app/mcp", "--type", "http"]
        );
    }

    #[test]
    fn build_args_with_header() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer tok".to_string());

        let entry = McpDecl {
            name: "github".to_string(),
            url: "https://api.githubcopilot.com/mcp/".to_string(),
            headers,
            instructions: String::new(),
        };

        let args = build_mcp_add_args(&entry);
        assert_eq!(
            args,
            vec![
                "mcp", "add", "github", "https://api.githubcopilot.com/mcp/",
                "--type", "http",
                "--header", "Authorization: Bearer tok",
            ]
        );
    }

    // -- parse_check_output --

    #[test]
    fn parse_check_valid() {
        let output = r#"{"type":"result","result":"[{\"name\":\"linear\",\"ok\":true}]"}"#;
        let result = parse_check_output(output);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "linear");
        assert!(result[0].ok);
    }

    #[test]
    fn parse_check_malformed_returns_empty() {
        assert!(parse_check_output("not json").is_empty());
        assert!(parse_check_output(r#"{"type":"result","result":"garbage"}"#).is_empty());
    }

    #[test]
    fn parse_check_error_returns_empty() {
        let output = r#"{"type":"result","subtype":"failure","is_error":true,"result":"Exec failed"}"#;
        assert!(parse_check_output(output).is_empty());
    }

    // -- build_mcp_remove_args --

    #[test]
    fn remove_args_basic() {
        let args = build_mcp_remove_args("linear");
        assert_eq!(args, vec!["mcp", "remove", "linear"]);
    }

    #[test]
    fn build_args_headers_sorted() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Custom".to_string(), "val".to_string());
        headers.insert("Authorization".to_string(), "Bearer tok".to_string());

        let entry = McpDecl {
            name: "test".to_string(),
            url: "https://example.com/mcp".to_string(),
            headers,
            instructions: String::new(),
        };

        let args = build_mcp_add_args(&entry);
        let header_positions: Vec<usize> = args
            .iter()
            .enumerate()
            .filter(|(_, a)| *a == "--header")
            .map(|(i, _)| i)
            .collect();

        assert_eq!(header_positions.len(), 2);
        assert_eq!(args[header_positions[0] + 1], "Authorization: Bearer tok");
        assert_eq!(args[header_positions[1] + 1], "X-Custom: val");
    }
}
