mod common;

use common::TestEnv;

// -- ace mcp reset --

const SCHOOL_TOML_TWO_SERVERS: &str = r#"
name = "test-school"

[[mcp]]
name = "linear"
url = "https://mcp.linear.app/mcp"

[[mcp]]
name = "github"
url = "https://api.githubcopilot.com/mcp/"
"#;

#[test]
fn mcp_reset_removes_all_registered() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_TWO_SERVERS);
    env.write_flaude_mcp_list(&["linear", "github"]);

    env.ace().args(["mcp", "reset"]).assert().success();

    // After reset, both should be removed from the list
    let list_content = env.read_file(".flaude-mcp-list");
    assert!(!list_content.contains("linear"), "linear should be removed");
    assert!(!list_content.contains("github"), "github should be removed");
}

#[test]
fn mcp_reset_removes_specific_server() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_TWO_SERVERS);
    env.write_flaude_mcp_list(&["linear", "github"]);

    env.ace().args(["mcp", "reset", "linear"]).assert().success();

    // Only linear should be removed
    let list_content = env.read_file(".flaude-mcp-list");
    assert!(!list_content.contains("linear"), "linear should be removed");
    assert!(list_content.contains("github"), "github should remain");
}

#[test]
fn mcp_reset_noop_when_nothing_registered() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_TWO_SERVERS);
    // No flaude-mcp-list → nothing registered

    env.ace().args(["mcp", "reset"]).assert().success();
}

#[test]
fn mcp_clear_is_alias_for_reset() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_TWO_SERVERS);
    env.write_flaude_mcp_list(&["linear"]);

    env.ace().args(["mcp", "clear"]).assert().success();

    let list_content = env.read_file(".flaude-mcp-list");
    assert!(!list_content.contains("linear"), "linear should be removed");
}

// -- ace mcp check --

#[test]
fn mcp_check_reports_registered_servers() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_TWO_SERVERS);
    env.write_flaude_mcp_list(&["linear", "github"]);

    let output = env.ace().args(["mcp", "check"]).output().expect("should run");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains("linear"), "should report linear status");
    assert!(combined.contains("github"), "should report github status");
}

#[test]
fn mcp_check_reports_missing_servers() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_TWO_SERVERS);
    // Only linear is registered
    env.write_flaude_mcp_list(&["linear"]);

    let output = env.ace().args(["mcp", "check"]).output().expect("should run");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(combined.contains("github"), "should mention github as missing");
}

// -- ace mcp (default) --

#[test]
fn mcp_default_registers_missing_and_checks() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_TWO_SERVERS);
    // Nothing registered yet

    env.ace().args(["mcp"]).assert().success();

    // Should have registered both servers
    let records = env.read_flaude_mcp_records();
    assert_eq!(records.len(), 2, "should register both missing servers");
}

const SCHOOL_TOML_OAUTH: &str = r#"
name = "test-school"

[[mcp]]
name = "linear"
url = "https://mcp.linear.app/mcp"
"#;

const SCHOOL_TOML_WITH_HEADERS: &str = r#"
name = "test-school"

[[mcp]]
name = "sentry"
url = "https://mcp.sentry.dev/sse"

[mcp.headers]
Authorization = "Bearer test-token-123"
"#;

#[test]
fn mcp_register_oauth_server() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_OAUTH);

    env.ace().assert().success();

    let records = env.read_flaude_mcp_records();
    assert_eq!(records.len(), 1, "should register one server");
    assert_eq!(records[0].name, "linear");
    assert_eq!(records[0].url, "https://mcp.linear.app/mcp");
    assert!(records[0].headers.is_empty(), "OAuth server has no headers");
}

#[test]
fn mcp_register_with_headers() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_WITH_HEADERS);

    env.ace().assert().success();

    let records = env.read_flaude_mcp_records();
    assert_eq!(records.len(), 1, "should register one server");
    assert_eq!(records[0].name, "sentry");
    assert!(
        records[0]
            .headers
            .iter()
            .any(|h| h == "Authorization: Bearer test-token-123"),
        "should include auth header, got: {:?}",
        records[0].headers
    );
}

#[test]
fn mcp_backend_flag_uses_overridden_backend() {
    let env = TestEnv::new();
    env.setup_flaude_school(SCHOOL_TOML_OAUTH);
    env.mkdir("bin");
    env.write_executable(
        "bin/codex",
        r#"#!/bin/sh
if [ "$1" = "mcp" ] && [ "$2" = "list" ] && [ "$3" = "--json" ]; then
  printf '[]'
  exit 0
fi

if [ "$1" = "mcp" ] && [ "$2" = "add" ]; then
  printf '%s\n' "$@" > "$HOME/codex-mcp-add.txt"
  exit 0
fi

echo "unexpected invocation: $*" >&2
exit 1
"#,
    );

    env.ace_with_path_prefix(&env.path("bin"))
        .args(["--backend", "codex", "mcp"])
        .assert()
        .success();

    env.assert_exists("codex-mcp-add.txt");
    env.assert_not_exists(".flaude-mcp-records.jsonl");
}
