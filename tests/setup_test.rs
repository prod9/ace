mod common;

use common::TestEnv;

#[test]
fn setup_embedded_school() {
    let env = TestEnv::new();
    env.setup_embedded("danger-zone");

    // ace.toml written with school specifier.
    env.assert_exists("ace.toml");
    env.assert_contains("ace.toml", "school");

    // Skills symlinked into .claude/skills -> project root skills/.
    env.assert_symlink(".claude/skills", "skills");

    // CLAUDE.md generated with school name.
    env.assert_exists("CLAUDE.md");
    env.assert_contains("CLAUDE.md", "danger-zone");
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
    env.setup_embedded("iceman");

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
    env.setup_embedded("goose");

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
    env.setup_embedded("viper");

    env.assert_exists("CLAUDE.md");
    env.assert_contains("CLAUDE.md", "viper");
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

#[test]
fn setup_gitignore_ignores_symlinks() {
    let env = TestEnv::new();
    env.git_init();
    env.setup_embedded_school("rooster");

    // Commit school files first so they're tracked.
    env.git_commit("school files");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    // After setup, .claude/skills is a new symlink. The .gitignore should
    // prevent it from appearing as untracked in git status.
    // Git rolls untracked entries up to the directory level, so we check
    // that .claude/ itself doesn't appear (all its contents are ignored).
    let status = env.git_status();
    assert!(
        !status.contains(".claude/"),
        ".claude/ entries should be ignored by git, but git status shows:\n{status}"
    );
}

#[test]
fn setup_codex_backend() {
    let env = TestEnv::new();
    env.git_init();

    env.write_file(
        "school.toml",
        "name = \"slider\"\nbackend = \"codex\"\n",
    );
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    env.assert_symlink(".agents/skills", "skills");
    env.assert_exists("AGENTS.md");
    env.assert_contains("AGENTS.md", "slider");
    env.assert_not_exists("CLAUDE.md");
}

#[test]
fn setup_opencode_backend() {
    let env = TestEnv::new();
    env.git_init();

    env.write_file(
        "school.toml",
        "name = \"jester\"\nbackend = \"opencode\"\n",
    );
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test\n");

    env.ace()
        .args(["setup", "."])
        .assert()
        .success();

    env.assert_symlink(".opencode/skills", "skills");
    env.assert_exists("AGENTS.md");
    env.assert_contains("AGENTS.md", "jester");
    env.assert_not_exists("CLAUDE.md");
}

use predicates;
