mod common;

use common::TestEnv;

#[test]
fn import_no_school_context() {
    let env = TestEnv::new();
    env.git_init();

    // No school.toml, no ace.toml — should fail with "no config" error.
    env.ace()
        .args(["import", "owner/repo", "--skill", "my-skill"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no config found"));
}

#[test]
fn import_clone_failure_invalid_source() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.mkdir("skills");

    // Source that cannot be cloned — nonexistent GitHub repo.
    env.ace()
        .args(["import", "nonexistent-owner-xxxxx/nonexistent-repo-xxxxx", "--skill", "my-skill"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("clone failed"));
}

#[test]
fn import_requires_source_argument() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");

    // Missing required <source> argument.
    env.ace()
        .args(["import"])
        .assert()
        .failure();
}

#[test]
fn import_from_local_school_context() {
    let env = TestEnv::new();
    env.git_init();

    // School repo context (school.toml present) but invalid remote source.
    env.write_file("school.toml", "name = \"my-school\"\n");
    env.mkdir("skills");

    // The source is invalid, so clone fails — but this verifies that import
    // correctly detects the school context via school.toml.
    env.ace()
        .args(["import", "nonexistent-owner-xxxxx/nonexistent-repo-xxxxx"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("clone failed"));
}

#[test]
fn import_without_skill_flag_clone_failure() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.mkdir("skills");

    // No --skill flag — auto-select or prompt would happen after clone.
    // Clone fails first, so we verify the error path without --skill.
    env.ace()
        .args(["import", "nonexistent-owner-xxxxx/nonexistent-repo-xxxxx"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("clone failed"));
}

#[test]
fn import_no_git_repo_with_school_toml() {
    let env = TestEnv::new();
    // No git init — but school.toml exists.
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.mkdir("skills");

    // Import should still work to find school context (school.toml check
    // doesn't require git), but clone will fail on the remote source.
    env.ace()
        .args(["import", "nonexistent-owner-xxxxx/nonexistent-repo-xxxxx", "--skill", "x"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("clone failed"));
}

#[test]
fn import_skill_flag_requires_value() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");

    // --skill without a value should be a clap argument error.
    env.ace()
        .args(["import", "owner/repo", "--skill"])
        .assert()
        .failure();
}

#[test]
fn import_with_existing_imports_clone_failure() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file(
        "school.toml",
        r#"name = "test-school"

[[imports]]
skill = "existing-skill"
source = "some-owner/some-repo"
"#,
    );
    env.mkdir("skills/existing-skill");
    env.write_file("skills/existing-skill/SKILL.md", "# Existing\n");

    // Importing a new skill from an invalid source fails at clone.
    // Verifies that having existing imports doesn't break the import flow.
    env.ace()
        .args(["import", "nonexistent-owner-xxxxx/nonexistent-repo-xxxxx", "--skill", "new-skill"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("clone failed"));
}

use predicates;
