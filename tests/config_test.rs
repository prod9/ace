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
