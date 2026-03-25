mod common;

use common::{read_flaude_records, TestEnv};

const ACE_TOML_FLAUDE: &str = r#"
school = "."
backend = "flaude"
"#;

fn setup_flaude(env: &TestEnv) {
    env.git_init();
    env.write_file("school.toml", "name = \"test-school\"\n");
    env.write_file("ace.toml", ACE_TOML_FLAUDE);
    env.mkdir("skills/test-skill");
    env.write_file("skills/test-skill/SKILL.md", "# Test\n");
    env.write_file("CLAUDE.md", "# Test\n");
    env.mkdir(".claude");
    env.symlink("skills", ".claude/skills");
}

#[test]
fn exec_records_backend_args() {
    let env = TestEnv::new();
    setup_flaude(&env);

    env.ace_flaude("")
        .assert()
        .success();

    let records = read_flaude_records(&env.path("flaude-record.jsonl"));
    let exec_records: Vec<_> = records.iter().filter(|r| r.action == "exec").collect();
    assert_eq!(exec_records.len(), 1, "should record one exec call");
}

#[test]
fn exec_yolo_passes_flag() {
    let env = TestEnv::new();
    setup_flaude(&env);
    env.write_file("ace.local.toml", "yolo = true\n");

    let output = env.ace_flaude("")
        .output()
        .expect("ace run");

    assert!(output.status.success(), "ace should succeed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("yolo"), "should warn about yolo mode");

    let records = read_flaude_records(&env.path("flaude-record.jsonl"));
    let exec_records: Vec<_> = records.iter().filter(|r| r.action == "exec").collect();
    assert_eq!(exec_records.len(), 1, "should record one exec call");

    assert!(
        exec_records[0].backend_args.contains(&"--yolo".to_string()),
        "backend_args should contain --yolo, got: {:?}",
        exec_records[0].backend_args,
    );
}
