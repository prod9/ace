mod common;

use common::TestEnv;

#[test]
fn exec_records_backend_args() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"test-school\"\n");

    env.ace().assert().success();

    let records = env.read_flaude_exec_records();
    assert_eq!(records.len(), 1, "should record one exec call");
}

#[test]
fn exec_yolo_passes_flag() {
    let env = TestEnv::new();
    env.setup_flaude_school("name = \"test-school\"\n");
    env.write_file("ace.local.toml", "yolo = true\n");

    let output = env.ace().output().expect("ace run");

    assert!(output.status.success(), "ace should succeed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("yolo"), "should warn about yolo mode");

    let records = env.read_flaude_exec_records();
    assert_eq!(records.len(), 1, "should record one exec call");

    assert!(
        records[0].backend_args.contains(&"--yolo".to_string()),
        "backend_args should contain --yolo, got: {:?}",
        records[0].backend_args,
    );
}
