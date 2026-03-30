mod common;

use common::TestEnv;

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
