mod common;

use common::TestEnv;

#[test]
fn config_shows_effective() {
    let env = TestEnv::new();
    env.setup_embedded("top-gun");

    let output = env.ace()
        .args(["config"])
        .output()
        .expect("ace config");

    assert!(output.status.success(), "ace config should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain the school specifier and backend field.
    assert!(stdout.contains("school"), "output should contain school field");
    assert!(stdout.contains("backend"), "output should contain backend field");
}

#[test]
fn config_includes_school_toml() {
    let env = TestEnv::new();
    env.setup_embedded("top-gun");

    let output = env.ace()
        .args(["config"])
        .output()
        .expect("ace config");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# school.toml"), "should include school.toml section header");
    assert!(stdout.contains("top-gun"), "should include school name from school.toml");
}

#[test]
fn config_shows_trust_from_local() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.write_file("ace.local.toml", "trust = \"auto\"\n");

    let output = env.ace()
        .args(["config"])
        .output()
        .expect("ace config");

    assert!(output.status.success(), "ace config should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("trust = \"auto\""), "trust should appear in effective config");
}

#[test]
fn config_backcompat_yolo_becomes_trust() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.write_file("ace.local.toml", "yolo = true\n");

    let output = env.ace()
        .args(["config"])
        .output()
        .expect("ace config");

    assert!(output.status.success(), "ace config should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("trust = \"yolo\""), "yolo=true should resolve to trust=yolo");
}

#[test]
fn config_no_ace_toml() {
    let env = TestEnv::new();
    // No ace.toml — require_state should fail.

    env.ace()
        .args(["config"])
        .assert()
        .failure();
}

#[test]
fn config_backend_flag_overrides_effective_backend() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"phoenix\"\n");

    let output = env.ace()
        .args(["--backend", "codex", "config"])
        .output()
        .expect("ace config");

    assert!(output.status.success(), "ace config should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend = \"codex\""), "backend override should appear in effective config");
}

#[test]
fn config_backend_short_flag_overrides_effective_backend() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"phoenix\"\n");

    let output = env.ace()
        .args(["-b", "codex", "config"])
        .output()
        .expect("ace config");

    assert!(output.status.success(), "ace config should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend = \"codex\""), "short backend override should appear in effective config");
}

#[test]
fn config_backend_alias_flag_overrides_effective_backend() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"phoenix\"\n");

    let output = env.ace()
        .args(["--codex", "config"])
        .output()
        .expect("ace config");

    assert!(output.status.success(), "ace config should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend = \"codex\""), "backend alias should appear in effective config");
}

#[test]
fn config_backend_alias_conflicts_with_backend_flag() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"phoenix\"\n");

    env.ace()
        .args(["--backend", "codex", "--claude", "config"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot combine multiple backend override flags"));
}

// -- config get --

#[test]
fn config_get_backend() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"phoenix\"\n");

    let output = env.ace()
        .args(["config", "get", "backend"])
        .output()
        .expect("ace config get backend");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "flaude");
}

#[test]
fn config_get_school() {
    let env = TestEnv::new();
    env.setup_embedded("top-gun");

    let output = env.ace()
        .args(["config", "get", "school"])
        .output()
        .expect("ace config get school");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), ".");
}

#[test]
fn config_get_trust_default() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    let output = env.ace()
        .args(["config", "get", "trust"])
        .output()
        .expect("ace config get trust");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "default");
}

#[test]
fn config_get_env_key() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");
    env.write_file("ace.toml", "school = \".\"\n\n[env]\nFOO = \"bar\"\n");

    let output = env.ace()
        .args(["config", "get", "env.FOO"])
        .output()
        .expect("ace config get env.FOO");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "bar");
}

#[test]
fn config_get_unknown_key() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["config", "get", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown config key"));
}

// -- config set --

#[test]
fn config_set_backend_to_project() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["config", "set", "backend", "codex"])
        .assert()
        .success();

    env.assert_contains("ace.toml", "backend = \"codex\"");
}

#[test]
fn config_set_trust_defaults_to_local() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["config", "set", "trust", "auto"])
        .assert()
        .success();

    // Trust is personal-only → defaults to local scope
    env.assert_contains("ace.local.toml", "trust = \"auto\"");
}

#[test]
fn config_set_with_explicit_user_scope() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["--user", "config", "set", "backend", "codex"])
        .assert()
        .success();

    // Should be in user config, not project
    env.assert_contains("config/ace/ace.toml", "backend = \"codex\"");
    env.assert_not_contains("ace.toml", "codex");
}

#[test]
fn config_set_with_global_alias() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["--global", "config", "set", "trust", "yolo"])
        .assert()
        .success();

    env.assert_contains("config/ace/ace.toml", "trust = \"yolo\"");
}

#[test]
fn config_set_env_dot_path() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["config", "set", "env.MY_KEY", "my_value"])
        .assert()
        .success();

    env.assert_contains("ace.toml", "MY_KEY = \"my_value\"");
}

#[test]
fn config_set_resume_to_local() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["config", "set", "resume", "false"])
        .assert()
        .success();

    env.assert_contains("ace.local.toml", "resume = false");
}

#[test]
fn config_set_invalid_backend() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["config", "set", "backend", "invalid"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown backend"));
}

#[test]
fn config_set_invalid_trust() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["config", "set", "trust", "invalid"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown trust mode"));
}

// -- scope flag conflicts --

#[test]
fn scope_flags_conflict() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["--user", "--local", "config", "set", "trust", "auto"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot combine multiple scope flags"));
}

// -- user layer resolution --

#[test]
fn user_layer_provides_defaults() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    // Set backend at user level
    env.write_file("config/ace/ace.toml", "backend = \"codex\"\n");

    let output = env.ace()
        .args(["config", "get", "backend"])
        .output()
        .expect("ace config get backend");

    assert!(output.status.success());
    // Embedded setup writes ace.toml with no backend, so user layer should win.
    // But setup_embedded uses `ace setup .` which doesn't set backend.
    // Actually let me check — setup_embedded does git_init + setup_embedded_school + ace setup .
    // ace setup writes school=. but no backend. So user layer codex should be effective.
    // However, default backend fallback is claude. User layer codex should override that.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "codex");
}

#[test]
fn local_layer_overrides_user_layer_trust() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.write_file("config/ace/ace.toml", "trust = \"auto\"\n");
    env.write_file("ace.local.toml", "trust = \"yolo\"\n");

    let output = env.ace()
        .args(["config", "get", "trust"])
        .output()
        .expect("ace config get trust");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "yolo");
}

#[test]
fn user_layer_trust_used_when_no_local() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.write_file("config/ace/ace.toml", "trust = \"auto\"\n");

    let output = env.ace()
        .args(["config", "get", "trust"])
        .output()
        .expect("ace config get trust");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "auto");
}

// -- read-only inspection survives a stale backend selector --

#[test]
fn config_show_survives_unknown_backend() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");
    env.write_file("ace.local.toml", "backend = \"no-such-backend\"\n");

    let output = env.ace()
        .args(["config"])
        .output()
        .expect("ace config");

    assert!(output.status.success(), "ace config show should succeed even with unknown backend");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("backend = \"no-such-backend\""),
        "should print the configured backend name verbatim, got: {stdout}"
    );
}

#[test]
fn config_get_backend_survives_unknown_backend() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");
    env.write_file("ace.local.toml", "backend = \"no-such-backend\"\n");

    let output = env.ace()
        .args(["config", "get", "backend"])
        .output()
        .expect("ace config get backend");

    assert!(output.status.success(), "ace config get backend should succeed even with unknown backend");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "no-such-backend");
}

// -- yolo with scope --

#[test]
fn yolo_with_user_scope() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["--user", "yolo"])
        .assert()
        .success();

    env.assert_contains("config/ace/ace.toml", "trust = \"yolo\"");
}

#[test]
fn auto_with_user_scope() {
    let env = TestEnv::new();
    env.setup_embedded("phoenix");

    env.ace()
        .args(["--user", "auto"])
        .assert()
        .success();

    env.assert_contains("config/ace/ace.toml", "trust = \"auto\"");
}
