//! Network-dependent integration tests.
//!
//! These tests are `#[ignore]` by default. Run them with:
//!   cargo test -- --ignored
//! or via the CI flag:
//!   ACE_TEST_NETWORK=1 cargo test -- --ignored
//!
//! They require network access (git clone from GitHub).

mod common;

use common::TestEnv;

fn require_network() {
    if std::env::var("ACE_TEST_NETWORK").is_err() {
        eprintln!("skipping: ACE_TEST_NETWORK not set");
    }
}

#[test]
#[ignore]
fn setup_remote_school() {
    require_network();

    let env = TestEnv::new();
    env.git_init();

    // Use a known public repo. This tests the full clone + install + link flow.
    // prod9/school is the org's actual school repo.
    env.ace()
        .args(["setup", "prod9/school"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    env.assert_exists("ace.toml");
    env.assert_contains("ace.toml", "prod9/school");

    // Skills should be symlinked from cache.
    env.assert_exists(".claude/skills");

    // CLAUDE.md should be generated.
    env.assert_exists("CLAUDE.md");
}

#[test]
#[ignore]
fn setup_remote_then_rerun() {
    require_network();

    let env = TestEnv::new();
    env.git_init();

    // First setup — clones school.
    env.ace()
        .args(["setup", "prod9/school"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();

    // Second run (no subcommand) — should update + exec.
    // Will fail at exec (no claude binary) but Prepare should succeed.
    // Check that it gets past the prepare phase.
    let output = env.ace()
        .timeout(std::time::Duration::from_secs(30))
        .output()
        .expect("ace run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should not contain "clone failed" — the update path should work.
    assert!(
        !stderr.contains("clone failed"),
        "re-run should not re-clone: {stderr}"
    );
}
