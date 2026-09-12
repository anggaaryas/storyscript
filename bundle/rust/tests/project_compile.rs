use std::fs;
use std::path::{Path, PathBuf};

use storyscript_bundle::COMPILER_VERSION;
use storyscript_bundle::project;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo_project")
}

#[test]
fn compiles_project_descriptor_includes_ir_and_closed_assets() {
    let compiled = project::compile(&fixture()).expect("fixture compiles");

    let labels = compiled
        .story
        .scenes
        .iter()
        .map(|scene| scene.label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(labels, ["opening", "child_scene"]);
    assert_eq!(compiled.config.project.compiler_version, COMPILER_VERSION);
    assert_eq!(
        compiled
            .assets
            .iter()
            .map(|asset| asset.logical_path.as_str())
            .collect::<Vec<_>>(),
        [
            "audio/click.m3u8",
            "audio/theme.m3u8",
            "backgrounds/day.svg",
            "backgrounds/night.svg",
            "portraits/hero.svg",
        ]
    );
}

#[test]
fn rejects_project_with_non_exact_compiler_version() {
    let project_root = tempfile::tempdir().expect("temporary project");
    fs::write(
        project_root.path().join("StoryScript.toml"),
        r#"
[project]
id = "bad.pin"
name = "Bad Pin"
version = "1.0.0"
entry = "main.StoryScript"
compiler-version = "99.0.0"
"#,
    )
    .expect("configuration");

    let error = project::compile(project_root.path()).expect_err("pin mismatch must fail");
    assert_eq!(error.code().to_string(), "B_COMPILER_MISMATCH");
}

#[test]
fn parser_diagnostics_remain_typed_project_failures() {
    let project_root = tempfile::tempdir().expect("temporary project");
    fs::write(
        project_root.path().join("StoryScript.toml"),
        format!(
            r#"
[project]
id = "bad.source"
name = "Bad Source"
version = "1.0.0"
entry = "main.StoryScript"
compiler-version = "{COMPILER_VERSION}"
"#
        ),
    )
    .expect("configuration");
    fs::write(
        project_root.path().join("main.StoryScript"),
        "* scene { #STORY @end }",
    )
    .expect("source");
    fs::create_dir(project_root.path().join("assets")).expect("assets");

    let error = project::compile(project_root.path()).expect_err("source must fail");
    assert_eq!(error.code().to_string(), "B_COMPILE_FAILED");
    assert!(error.to_string().contains("E_INIT_COUNT"));
}
