mod common;

use common::{read_flaude_mcp_records, TestEnv};

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

const SCHOOL_TOML_MULTI: &str = r#"
name = "test-school"

[[mcp]]
name = "linear"
url = "https://mcp.linear.app/mcp"

[[mcp]]
name = "sentry"
url = "https://mcp.sentry.dev/sse"
"#;

const ACE_TOML_FLAUDE: &str = r#"
school = "."
backend = "flaude"
"#;

fn setup_env(env: &TestEnv, school_toml: &str) {
    env.git_init();
    env.write_file("school.toml", school_toml);
    env.write_file("ace.toml", ACE_TOML_FLAUDE);
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test\n");
    env.write_file("CLAUDE.md", "# Test\n");
    env.mkdir(".claude");
    env.symlink("skills", ".claude/skills");
}

#[test]
fn mcp_register_oauth_server() {
    let env = TestEnv::new();
    setup_env(&env, SCHOOL_TOML_OAUTH);

    env.ace_flaude("")
        .assert()
        .success();

    let records = read_flaude_mcp_records(&env.path("flaude-record.jsonl"));
    assert_eq!(records.len(), 1, "should register one server");
    assert_eq!(records[0].name, "linear");
    assert_eq!(records[0].url, "https://mcp.linear.app/mcp");
    assert!(records[0].headers.is_empty(), "OAuth server has no headers");
}

#[test]
fn mcp_register_with_headers() {
    let env = TestEnv::new();
    setup_env(&env, SCHOOL_TOML_WITH_HEADERS);

    env.ace_flaude("")
        .assert()
        .success();

    let records = read_flaude_mcp_records(&env.path("flaude-record.jsonl"));
    assert_eq!(records.len(), 1, "should register one server");
    assert_eq!(records[0].name, "sentry");
    assert!(
        records[0].headers.iter().any(|h| h == "Authorization: Bearer test-token-123"),
        "should include auth header, got: {:?}",
        records[0].headers
    );
}

#[test]
fn mcp_skip_already_registered() {
    let env = TestEnv::new();
    setup_env(&env, SCHOOL_TOML_OAUTH);

    // Pre-register "linear" in flaude's mcp list.
    env.ace_flaude("linear")
        .assert()
        .success();

    let records = read_flaude_mcp_records(&env.path("flaude-record.jsonl"));
    assert!(records.is_empty(), "should not register already-registered server");
}

#[test]
fn mcp_register_multiple_skip_existing() {
    let env = TestEnv::new();
    setup_env(&env, SCHOOL_TOML_MULTI);

    // Pre-register "linear" only.
    env.ace_flaude("linear")
        .assert()
        .success();

    let records = read_flaude_mcp_records(&env.path("flaude-record.jsonl"));
    assert_eq!(records.len(), 1, "should register only the missing server");
    assert_eq!(records[0].name, "sentry");
}

#[test]
fn mcp_no_entries() {
    let env = TestEnv::new();
    setup_env(&env, "name = \"test-school\"\n");

    env.ace_flaude("")
        .assert()
        .success();

    let records = read_flaude_mcp_records(&env.path("flaude-record.jsonl"));
    assert!(records.is_empty(), "no MCP entries means no registrations");
}
