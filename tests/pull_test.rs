mod common;

use common::{RemoteSchool, TestEnv};

// -- Helpers --

/// Push a new commit to origin via a temp working copy.
fn push_to_origin(env: &TestEnv, school: &RemoteSchool, file: &str, content: &str) {
    let work = env.path("_push_work");
    let origin_str = school.origin.to_str().expect("origin path");
    let work_str = work.to_str().expect("work path");

    env.git_in(env.root(), &["clone", "--quiet", origin_str, work_str]);
    std::fs::write(work.join(file), content).expect("write file");
    env.git_in(&work, &["add", "-A"]);
    env.git_in(
        &work,
        &[
            "-c",
            "user.email=test@test.com",
            "-c",
            "user.name=Test",
            "commit",
            "-m",
            "update from origin",
        ],
    );
    env.git_in(&work, &["push"]);
    std::fs::remove_dir_all(&work).expect("cleanup work dir");
}

// -- Tests --

#[test]
fn pull_fetches_without_cooldown() {
    let env = TestEnv::new();
    let school = env.setup_remote_school("test/school");

    // First run — creates FETCH_HEAD via normal ace flow.
    env.ace().assert().success();

    // Push new content to origin while FETCH_HEAD is still fresh.
    push_to_origin(&env, &school, "skills/maverick/NEW.md", "# New\n");

    // `ace pull` should fetch even though FETCH_HEAD is fresh.
    env.ace().args(["pull"]).assert().success();

    // Verify new content was fetched and merged.
    assert!(
        school.cache.join("skills/maverick/NEW.md").exists(),
        "pull should fetch new file despite fresh FETCH_HEAD",
    );
}

#[test]
fn pull_no_school_fails() {
    let env = TestEnv::new();
    env.git_init();
    env.write_file("ace.toml", "backend = \"flaude\"\n");

    env.ace().args(["pull"]).assert().failure();
}
