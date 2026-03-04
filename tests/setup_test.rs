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

    // Skills symlinked into .claude/skills → project root skills/.
    env.assert_symlink(".claude/skills", "skills");

    // CLAUDE.md generated with school name.
    env.assert_exists("CLAUDE.md");
    env.assert_contains("CLAUDE.md", "test-school");
}
