mod common;

use common::TestEnv;

#[test]
fn setup_embedded_school() {
    let env = TestEnv::new();
    env.git_init();

    // Embedded school: school.toml + skills/ in project root.
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test Skill\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    // ace.toml written with school specifier.
    env.assert_exists("ace.toml");
    env.assert_contains("ace.toml", "school");

    // Skills symlinked into .claude/skills -> project root skills/.
    env.assert_symlink(".claude/skills", "skills");

    // CLAUDE.md generated with school name.
    env.assert_exists("CLAUDE.md");
    env.assert_contains("CLAUDE.md", "test-school");
}

#[test]
fn setup_not_in_git_repo() {
    let env = TestEnv::new();
    // No git init — should fail.
    env.write_file("school.toml", "name = \"test-school\"\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .failure()
        .stderr(predicates::str::contains("git"));
}

#[test]
fn setup_already_set_up() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test Skill\n");

    // First setup succeeds.
    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    // Second setup fails — already set up.
    env.ace()
        .args(["setup", "."])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already set up"));
}

#[test]
fn setup_links_all_four_folders() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");

    for folder in &["skills", "rules", "commands", "agents"] {
        env.mkdir(&format!("{folder}/example"));
        env.write_file(&format!("{folder}/example/SKILL.md"), "# Example\n");
    }

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    for folder in &["skills", "rules", "commands", "agents"] {
        env.assert_symlink(&format!(".claude/{folder}"), folder);
    }
}

#[test]
fn setup_links_partial_folders() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");

    // Only skills/ and commands/ exist in school.
    env.mkdir("skills/my-skill");
    env.write_file("skills/my-skill/SKILL.md", "# Skill\n");
    env.mkdir("commands/my-cmd");
    env.write_file("commands/my-cmd/SKILL.md", "# Cmd\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    env.assert_symlink(".claude/skills", "skills");
    env.assert_symlink(".claude/commands", "commands");
    env.assert_not_exists(".claude/rules");
    env.assert_not_exists(".claude/agents");
}

#[test]
fn setup_adopts_existing_backend_dir() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");

    // School has skills/.
    env.mkdir("skills/school-skill");
    env.write_file("skills/school-skill/SKILL.md", "# School\n");

    // Project already has a real .claude/skills/ dir with content.
    env.mkdir(".claude/skills/my-local-skill");
    env.write_file(".claude/skills/my-local-skill/SKILL.md", "# Local\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    // Original dir renamed to previous-skills/.
    env.assert_exists(".claude/previous-skills/my-local-skill/SKILL.md");
    env.assert_contains(".claude/previous-skills/my-local-skill/SKILL.md", "Local");

    // Symlink now points to school skills.
    env.assert_symlink(".claude/skills", "skills");
}

#[test]
fn setup_idempotent_relink() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test\n");

    // First setup.
    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    env.assert_symlink(".claude/skills", "skills");

    // Delete ace.toml to allow re-setup.
    std::fs::remove_file(env.path("ace.toml")).expect("remove ace.toml");

    // Re-setup — symlink should still be correct.
    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    env.assert_symlink(".claude/skills", "skills");
}

#[test]
fn setup_generates_claude_md() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("school.toml", "name = \"My Cool School\"\n");
    env.mkdir("skills/example");
    env.write_file("skills/example/SKILL.md", "# Example\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    env.assert_exists("CLAUDE.md");
    env.assert_contains("CLAUDE.md", "My Cool School");
    // Should reference the skills directory.
    env.assert_contains("CLAUDE.md", "skills");
}

#[test]
fn setup_embedded_with_subpath() {
    let env = TestEnv::new();
    env.git_init();

    // school.toml lives in school/ subdir.
    env.write_file("school/school.toml", "name = \"sub-school\"\n");
    env.mkdir("school/skills/sub-skill");
    env.write_file("school/skills/sub-skill/SKILL.md", "# Sub\n");

    env.ace()
        .args(["setup", ".:school"])
        .assert()
        .success();

    env.assert_exists("ace.toml");
    env.assert_contains("ace.toml", ".:school");

    // Symlinks should point into school/ subdir.
    env.assert_symlink(".claude/skills", "school/skills");
}

use predicates;
