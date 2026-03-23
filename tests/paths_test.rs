mod common;

use common::TestEnv;

#[test]
fn paths_lists_config_paths() {
    let env = TestEnv::new();
    env.setup_embedded("top-gun");

    let output = env.ace()
        .args(["paths"])
        .output()
        .expect("ace paths");

    assert!(output.status.success(), "ace paths should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have tab-separated key\tvalue lines.
    assert!(stdout.contains("config.user\t"), "should list config.user");
    assert!(stdout.contains("config.local\t"), "should list config.local");
    assert!(stdout.contains("config.project\t"), "should list config.project");
    assert!(stdout.contains("school.source\t"), "should list school.source");
    assert!(stdout.contains("school.root\t"), "should list school.root");
}

#[test]
fn paths_single_key() {
    let env = TestEnv::new();
    env.setup_embedded("top-gun");

    let output = env.ace()
        .args(["paths", "config.project"])
        .output()
        .expect("ace paths config.project");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output only the project config path — contains ace.toml path.
    assert!(stdout.contains("ace.toml"), "should output project config path");
    // Should NOT contain other keys.
    assert!(!stdout.contains("config.user"), "should not list other keys");
}

#[test]
fn paths_alias() {
    let env = TestEnv::new();
    env.setup_embedded("top-gun");

    let output = env.ace()
        .args(["paths", "project"])
        .output()
        .expect("ace paths project");

    assert!(output.status.success(), "alias 'project' should resolve");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ace.toml"), "alias should resolve to config.project path");
}

#[test]
fn paths_unknown_key() {
    let env = TestEnv::new();
    env.setup_embedded("top-gun");

    env.ace()
        .args(["paths", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown key"));
}

use predicates;
