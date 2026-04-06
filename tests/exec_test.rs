mod common;

use common::TestEnv;

#[test]
fn exec_records_session() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"test-school\"\n");

    env.ace().assert().success();

    let records = env.read_flaude_exec_records();
    assert_eq!(records.len(), 1, "should record one exec call");
    assert_eq!(records[0].trust, "default", "default trust level");
    assert!(!records[0].session_prompt.is_empty(), "session prompt should be non-empty");
}

#[test]
fn exec_yolo_records_trust() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"test-school\"\n");
    env.write_file("ace.local.toml", "trust = \"yolo\"\n");

    let output = env.ace().output().expect("ace run");

    assert!(output.status.success(), "ace should succeed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("yolo mode"), "should warn about yolo mode");

    let records = env.read_flaude_exec_records();
    assert_eq!(records.len(), 1, "should record one exec call");
    assert_eq!(records[0].trust, "yolo", "trust should be yolo");
}

#[test]
fn exec_auto_records_trust() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"test-school\"\n");
    env.write_file("ace.local.toml", "trust = \"auto\"\n");

    let output = env.ace().output().expect("ace run");

    assert!(output.status.success(), "ace should succeed");

    let records = env.read_flaude_exec_records();
    assert_eq!(records.len(), 1, "should record one exec call");
    assert_eq!(records[0].trust, "auto", "trust should be auto");
}

#[test]
fn exec_backcompat_yolo_true() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"test-school\"\n");
    env.write_file("ace.local.toml", "yolo = true\n");

    let output = env.ace().output().expect("ace run");

    assert!(output.status.success(), "ace should succeed");

    let records = env.read_flaude_exec_records();
    assert_eq!(records.len(), 1, "should record one exec call");
    assert_eq!(records[0].trust, "yolo", "yolo=true backcompat should record trust=yolo");
}
