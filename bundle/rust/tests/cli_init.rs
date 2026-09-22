use std::fs;
use std::process::Command;

use storyscript_bundle::project;

fn init_command(target: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"));
    command.args([
        "init",
        target.to_str().expect("UTF-8 test path"),
        "--id",
        "com.example.starter",
        "--name",
        "Example Starter",
    ]);
    command
}

#[test]
fn init_creates_a_minimal_compilable_project() {
    let temp = tempfile::tempdir().expect("temporary parent");
    let target = temp.path().join("starter");
    let result = init_command(&target).output().expect("run init CLI");

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        format!("Initialized {}\n", target.display())
    );
    assert!(target.join("StoryScript.toml").is_file());
    assert!(target.join("story/main.StoryScript").is_file());
    assert!(target.join("assets/.gitkeep").is_file());

    let compiled = project::compile(&target).expect("generated project compiles");
    assert_eq!(compiled.config.project.id, "com.example.starter");
    assert_eq!(compiled.config.project.name, "Example Starter");
    assert_eq!(compiled.story.scenes.len(), 1);
    assert!(compiled.assets.is_empty());
}

#[test]
fn init_returns_json_success() {
    let temp = tempfile::tempdir().expect("temporary parent");
    let target = temp.path().join("starter");
    let result = init_command(&target)
        .arg("--json")
        .output()
        .expect("run init CLI");

    assert!(result.status.success());
    let output: serde_json::Value =
        serde_json::from_slice(&result.stdout).expect("JSON success output");
    assert_eq!(output["status"], "ok");
    assert_eq!(output["path"], target.to_str().expect("UTF-8 test path"));
}

#[test]
fn init_rejects_every_existing_target_without_modifying_it() {
    let temp = tempfile::tempdir().expect("temporary parent");
    let target = temp.path().join("starter");
    fs::create_dir(&target).expect("existing directory");
    fs::write(target.join("keep.txt"), "keep").expect("existing content");

    let result = init_command(&target)
        .arg("--json")
        .output()
        .expect("run init CLI");

    assert!(!result.status.success());
    let output: serde_json::Value =
        serde_json::from_slice(&result.stderr).expect("JSON error output");
    assert_eq!(output["status"], "error");
    assert_eq!(output["code"], "B_OUTPUT_INVALID");
    assert_eq!(
        fs::read_to_string(target.join("keep.txt")).expect("preserved content"),
        "keep"
    );
    assert!(!target.join("StoryScript.toml").exists());
}

#[test]
fn init_rejects_an_existing_empty_directory() {
    let temp = tempfile::tempdir().expect("temporary parent");
    let target = temp.path().join("starter");
    fs::create_dir(&target).expect("existing directory");

    let result = init_command(&target).output().expect("run init CLI");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("B_OUTPUT_INVALID"));
    assert_eq!(
        fs::read_dir(&target).expect("existing directory").count(),
        0
    );
}

#[test]
fn init_rejects_an_existing_file() {
    let temp = tempfile::tempdir().expect("temporary parent");
    let target = temp.path().join("starter");
    fs::write(&target, "keep").expect("existing file");

    let result = init_command(&target).output().expect("run init CLI");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("B_OUTPUT_INVALID"));
    assert_eq!(fs::read_to_string(&target).expect("preserved file"), "keep");
}

#[test]
fn init_rejects_empty_required_metadata_before_creating_the_target() {
    let temp = tempfile::tempdir().expect("temporary parent");
    let target = temp.path().join("starter");
    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args([
            "init",
            target.to_str().expect("UTF-8 test path"),
            "--id",
            " ",
            "--name",
            "Example Starter",
        ])
        .output()
        .expect("run init CLI");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("B_CONFIG_INVALID"));
    assert!(!target.exists());
}

#[test]
fn init_requires_explicit_id_and_name_flags() {
    let temp = tempfile::tempdir().expect("temporary parent");
    let target = temp.path().join("starter");
    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args(["init", target.to_str().expect("UTF-8 test path")])
        .output()
        .expect("run init CLI");

    assert_eq!(result.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("--id <ID>"));
    assert!(stderr.contains("--name <NAME>"));
    assert!(!target.exists());
}

#[test]
fn init_escapes_explicit_metadata_in_toml() {
    let temp = tempfile::tempdir().expect("temporary parent");
    let target = temp.path().join("starter");
    let result = Command::new(env!("CARGO_BIN_EXE_storyscript-bundle"))
        .args([
            "init",
            target.to_str().expect("UTF-8 test path"),
            "--id",
            "com.example.\"quoted\"",
            "--name",
            "A \"Quoted\" Story",
        ])
        .output()
        .expect("run init CLI");

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let compiled = project::compile(&target).expect("generated project compiles");
    assert_eq!(compiled.config.project.id, "com.example.\"quoted\"");
    assert_eq!(compiled.config.project.name, "A \"Quoted\" Story");
}

#[cfg(unix)]
#[test]
fn init_rejects_an_existing_symlink() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary parent");
    let destination = temp.path().join("destination");
    let target = temp.path().join("starter");
    fs::create_dir(&destination).expect("destination");
    symlink(&destination, &target).expect("symlink");

    let result = init_command(&target).output().expect("run init CLI");

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("B_OUTPUT_INVALID"));
    assert!(!destination.join("StoryScript.toml").exists());
}
