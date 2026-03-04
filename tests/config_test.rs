mod common;

use common::TestEnv;

#[test]
fn config_shows_effective() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

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
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    let output = env.ace()
        .args(["config"])
        .output()
        .expect("ace config");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# school.toml"), "should include school.toml section header");
    assert!(stdout.contains("test-school"), "should include school name from school.toml");
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
