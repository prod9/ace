mod common;

use common::TestEnv;

const SCHOOL_TOML_BASIC: &str = r#"
name = "test-school"
backend = "codex"
"#;

const SCHOOL_TOML_OAUTH: &str = r#"
name = "test-school"
backend = "codex"

[[mcp]]
name = "linear"
url = "https://mcp.linear.app/mcp"
"#;

const SCHOOL_TOML_WITH_HEADERS: &str = r#"
name = "test-school"
backend = "codex"

[[mcp]]
name = "github"
url = "https://api.githubcopilot.com/mcp/"

[mcp.headers]
Authorization = "Bearer test-token-123"
"#;

#[test]
fn codex_exec_does_not_send_session_prompt_and_uses_auto_flags() {
    let env = TestEnv::new();
    env.setup_codex_school(SCHOOL_TOML_BASIC);
    env.write_file("ace.local.toml", "trust = \"auto\"\n");
    env.mkdir("bin");
    env.write_executable(
        "bin/codex",
        r#"#!/bin/sh
# Record all args for assertion
printf '%s\n' "$@" > "$HOME/codex-exec-args.txt"
printf 'argc=%s\n' "$#" > "$HOME/codex-exec-meta.txt"
idx=1
for arg in "$@"; do
  printf '__ARG_%s_START__\n%s\n__ARG_%s_END__\n' "$idx" "$arg" "$idx" >> "$HOME/codex-exec-meta.txt"
  idx=$((idx + 1))
done
exit 0
"#,
    );

    // Default: resume mode — should get `resume --last -a on-request --sandbox danger-full-access`
    env.ace_with_path_prefix(&env.path("bin"))
        .assert()
        .success();

    let args = env.read_file("codex-exec-args.txt");
    assert!(args.contains("resume"), "expected resume subcommand, got:\n{args}");
    assert!(args.contains("--last"), "expected --last flag, got:\n{args}");
    assert!(args.contains("--ask-for-approval"), "expected --ask-for-approval flag, got:\n{args}");
    assert!(args.contains("on-request"), "expected on-request approval policy, got:\n{args}");
    assert!(args.contains("--sandbox"), "expected --sandbox flag, got:\n{args}");
    assert!(args.contains("danger-full-access"), "expected danger-full-access sandbox, got:\n{args}");
    assert!(!args.contains("developer_instructions="), "resume should skip developer_instructions:\n{args}");

    // New session: should get `-a on-request --sandbox danger-full-access -c developer_instructions=...`
    env.ace_with_path_prefix(&env.path("bin"))
        .args(["new"])
        .assert()
        .success();

    let args = env.read_file("codex-exec-args.txt");
    assert!(!args.contains("resume"), "new should not resume, got:\n{args}");
    assert!(args.contains("danger-full-access"), "expected danger-full-access sandbox, got:\n{args}");
    assert!(args.contains("developer_instructions="), "new should pass developer_instructions:\n{args}");
}

#[test]
fn codex_mcp_register_uses_cli_for_oauth_server() {
    let env = TestEnv::new();
    env.setup_codex_school(SCHOOL_TOML_OAUTH);
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
        .args(["mcp"])
        .assert()
        .success();

    let args = env.read_file("codex-mcp-add.txt");
    assert!(args.contains("mcp"));
    assert!(args.contains("add"));
    assert!(args.contains("linear"));
    assert!(args.contains("--url"));
    assert!(args.contains("https://mcp.linear.app/mcp"));
}

#[test]
fn codex_mcp_register_with_headers_falls_back_to_config() {
    let env = TestEnv::new();
    env.setup_codex_school(SCHOOL_TOML_WITH_HEADERS);
    env.mkdir("bin");
    env.write_executable(
        "bin/codex",
        r#"#!/bin/sh
if [ "$1" = "mcp" ] && [ "$2" = "list" ] && [ "$3" = "--json" ]; then
  printf '[]'
  exit 0
fi

if [ "$1" = "mcp" ] && [ "$2" = "add" ]; then
  echo "unexpected cli add" > "$HOME/codex-unexpected-add.txt"
  exit 1
fi

echo "unexpected invocation: $*" >&2
exit 1
"#,
    );

    env.ace_with_path_prefix(&env.path("bin"))
        .args(["mcp"])
        .assert()
        .success();

    env.assert_not_exists("codex-unexpected-add.txt");
    env.assert_contains(".codex/config.toml", "https://api.githubcopilot.com/mcp/");
    env.assert_contains(".codex/config.toml", "Authorization");
    env.assert_contains(".codex/config.toml", "Bearer test-token-123");
}

#[test]
fn codex_mcp_check_uses_output_file_and_schema() {
    let env = TestEnv::new();
    env.setup_codex_school(SCHOOL_TOML_OAUTH);
    env.mkdir("bin");
    env.write_executable(
        "bin/codex",
        r#"#!/bin/sh
if [ "$1" = "mcp" ] && [ "$2" = "list" ] && [ "$3" = "--json" ]; then
  printf '[{"name":"linear","enabled":true}]'
  exit 0
fi

if [ "$1" = "exec" ]; then
  shift
  output_file=""
  schema_file=""
  while [ $# -gt 0 ]; do
    case "$1" in
      -o)
        output_file="$2"
        shift 2
        ;;
      --output-schema)
        schema_file="$2"
        shift 2
        ;;
      *)
        shift
        ;;
    esac
  done

  [ -n "$output_file" ] || exit 10
  [ -n "$schema_file" ] || exit 11

  printf '[{"name":"linear","ok":true}]' > "$output_file"
  printf '%s\n' "$schema_file" > "$HOME/codex-check-schema.txt"
  exit 0
fi

echo "unexpected invocation: $*" >&2
exit 1
"#,
    );

    let output = env.ace_with_path_prefix(&env.path("bin"))
        .args(["mcp", "check"])
        .output()
        .expect("run ace mcp check");

    assert!(output.status.success(), "ace mcp check should succeed");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("linear"), "expected linear status in output:\n{combined}");
    env.assert_exists("codex-check-schema.txt");
}
