use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("install-agents").unwrap()
}

fn write_module_yaml(dir: &std::path::Path, name: &str) {
    fs::write(dir.join("module.yaml"), format!("name: {name}\n")).unwrap();
}

fn agent_md(name: &str) -> String {
    format!(
        "---\ntitle: {name}\nclaude.name: {name}\nclaude.model: sonnet\n\
         claude.description: Test agent\nclaude.tools: Read, Grep\n---\n\n\
         # {name}\n\nAgent body content.\n"
    )
}

#[test]
fn no_args_exits_1() {
    cmd()
        .assert()
        .code(1)
        .stderr(predicate::str::contains("source directory required"));
}

#[test]
fn version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("install-agents"));
}

#[test]
fn deploy_basic() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("agents");
    let dst = dir.path().join(".claude/agents");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("TestAgent.md"), agent_md("TestAgent")).unwrap();
    write_module_yaml(dir.path(), "test-module");

    cmd()
        .current_dir(dir.path())
        .arg(src.to_str().unwrap())
        .args(["--dst", dst.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed: TestAgent.md"));

    assert!(dst.join("TestAgent.md").exists());
    let content = fs::read_to_string(dst.join("TestAgent.md")).unwrap();
    assert!(content.contains("source: test-module/"));
    assert!(content.contains("TestAgent.md"));
}

#[test]
fn dry_run_no_write() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("agents");
    let dst = dir.path().join(".claude/agents");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("TestAgent.md"), agent_md("TestAgent")).unwrap();
    write_module_yaml(dir.path(), "test-module");

    cmd()
        .arg(src.to_str().unwrap())
        .args(["--dst", dst.to_str().unwrap(), "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[dry-run] Would install: TestAgent.md",
        ));

    assert!(!dst.join("TestAgent.md").exists());
}

#[test]
fn clean_removes_synced() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("agents");
    let dst = dir.path().join("output");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&dst).unwrap();
    fs::write(src.join("TestAgent.md"), agent_md("TestAgent")).unwrap();
    write_module_yaml(dir.path(), "test-module");

    // Deploy first so we have a properly formatted synced file
    Command::cargo_bin("install-agents")
        .unwrap()
        .arg(src.to_str().unwrap())
        .args(["--dst", dst.to_str().unwrap()])
        .assert()
        .success();

    cmd()
        .arg(src.to_str().unwrap())
        .args(["--dst", dst.to_str().unwrap(), "--clean"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed: TestAgent.md"))
        .stdout(predicate::str::contains("Installed: TestAgent.md"));
}

#[test]
fn skips_template() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("agents");
    let dst = dir.path().join("output");
    fs::create_dir_all(&src).unwrap();
    write_module_yaml(dir.path(), "test-module");
    fs::write(
        src.join("_Template.md"),
        "---\ntitle: Template\nclaude.name: Template\n---\n\nTemplate content.\n",
    )
    .unwrap();

    cmd()
        .arg(src.to_str().unwrap())
        .args(["--dst", dst.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed").not());
}

#[test]
fn skips_user_owned() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("agents");
    let dst = dir.path().join("output");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&dst).unwrap();
    fs::write(src.join("MyAgent.md"), agent_md("MyAgent")).unwrap();
    write_module_yaml(dir.path(), "test-module");

    // Pre-create a user-owned agent (no synced-from header)
    fs::write(
        dst.join("MyAgent.md"),
        "---\nname: MyAgent\n---\n\nUser-created content.\n",
    )
    .unwrap();

    cmd()
        .arg(src.to_str().unwrap())
        .args(["--dst", dst.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("user-created agent"));

    // Original content preserved
    let content = fs::read_to_string(dst.join("MyAgent.md")).unwrap();
    assert!(content.contains("User-created content"));
}

#[test]
fn invalid_dir_exits_1() {
    cmd()
        .arg("/tmp/nonexistent-dir-99999")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn dst_override() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("agents");
    let custom_dst = dir.path().join("custom-output");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("TestAgent.md"), agent_md("TestAgent")).unwrap();
    write_module_yaml(dir.path(), "test-module");

    cmd()
        .arg(src.to_str().unwrap())
        .args(["--dst", custom_dst.to_str().unwrap()])
        .assert()
        .success();

    assert!(custom_dst.join("TestAgent.md").exists());
}

#[test]
fn provider_detection_gemini() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("agents");
    let dst = dir.path().join(".gemini/agents");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("TestAgent.md"), agent_md("TestAgent")).unwrap();
    write_module_yaml(dir.path(), "test-module");

    cmd()
        .arg(src.to_str().unwrap())
        .args(["--dst", dst.to_str().unwrap()])
        .assert()
        .success();

    let content = fs::read_to_string(dst.join("TestAgent.md")).unwrap();
    // Gemini format: kebab-case name, kind: local
    assert!(content.contains("name: test-agent"));
    assert!(content.contains("kind: local"));
}

#[test]
fn help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}
