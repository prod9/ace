mod common;

use common::TestEnv;

#[test]
fn yolo_writes_local_toml() {
    let env = TestEnv::new();
    env.setup_embedded("test-school");

    env.ace()
        .arg("yolo")
        .assert()
        .success();

    env.assert_contains("ace.local.toml", "yolo = true");
}

#[test]
fn yolo_preserves_existing_local_fields() {
    let env = TestEnv::new();
    env.setup_embedded("test-school");
    env.write_file("ace.local.toml", "backend = \"opencode\"\n");

    env.ace()
        .arg("yolo")
        .assert()
        .success();

    env.assert_contains("ace.local.toml", "yolo = true");
    env.assert_contains("ace.local.toml", "backend = \"opencode\"");
}
