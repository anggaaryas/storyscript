use std::fs;
use std::path::Path;

use storyscript_parser::compiler::{compile_file, compile_project, compile_source};
use storyscript_parser::diagnostic::DiagnosticCode;
use tempfile::TempDir;

const ROOT_PREFIX: &str = r#"
* INIT {
    @include ["modules/first.StoryScript", "modules/second.StoryScript"];
    @start root_scene;
}

* root_scene {
    #STORY
    "root"
    @end
}
"#;

const FIRST_CHILD: &str = r#"
* REQUIRE { }
* first_scene {
    #STORY
    "first"
    @end
}
"#;

const SECOND_CHILD: &str = r#"
* REQUIRE { }
* second_scene {
    #STORY
    "second"
    @end
}
"#;

#[test]
fn keyed_sites_share_immutable_scope_frames_instead_of_copying_globals() {
    let output = compile_source(
        r#"* INIT { $count as integer = 1; @start s } * s { #STORY @"first-id"; @"second-id"; @end }"#,
    );
    assert!(output.diagnostics.iter().all(|d| !d.is_error()));
    let sites = output.message_sites();
    assert!(std::sync::Arc::ptr_eq(
        &sites[0].visible_variables,
        &sites[1].visible_variables
    ));
    assert_eq!(
        sites[0].visible_variables["count"],
        storyscript_parser::ast::VarType::Integer
    );
}

fn write_project() -> TempDir {
    let project = tempfile::tempdir().expect("temporary project");
    fs::create_dir(project.path().join("modules")).expect("modules directory");
    fs::write(project.path().join("root.StoryScript"), ROOT_PREFIX).expect("root source");
    fs::write(
        project.path().join("modules/first.StoryScript"),
        FIRST_CHILD,
    )
    .expect("first child");
    fs::write(
        project.path().join("modules/second.StoryScript"),
        SECOND_CHILD,
    )
    .expect("second child");
    project
}

#[test]
fn keyed_inventory_preserves_include_paths_spans_and_scope_and_rejects_duplicates() {
    let project = write_project();
    fs::write(
        project.path().join("root.StoryScript"),
        ROOT_PREFIX.replace("\"root\"", "@\"root-line\""),
    )
    .unwrap();
    fs::write(
        project.path().join("modules/first.StoryScript"),
        FIRST_CHILD.replace("\"first\"", "@\"child-line\""),
    )
    .unwrap();
    let output = compile_project(project.path(), Path::new("root.StoryScript")).unwrap();
    assert!(output.diagnostics.iter().all(|d| !d.is_error()));
    let sites = output.message_sites();
    assert_eq!(sites[0].text.source, "root.StoryScript");
    assert_eq!(sites[1].text.source, "modules/first.StoryScript");
    assert!(sites.iter().all(|s| s.text.line > 0 && s.text.column > 0));
    fs::write(
        project.path().join("modules/first.StoryScript"),
        FIRST_CHILD.replace("\"first\"", "@\"root-line\""),
    )
    .unwrap();
    assert_has_code(
        &compile_project(project.path(), Path::new("root.StoryScript")).unwrap(),
        DiagnosticCode::ELocalizationIdDuplicate,
    );

    let source = r#"* INIT { $names as array<string> = ["Ada"]; $count as integer = 2; @start s }
* s { #PREP $local as boolean = true; #STORY for ($name in snapshot $names) { @"loop-line"; } @end }"#;
    let output = compile_source(source);
    assert!(
        output.diagnostics.iter().all(|d| !d.is_error()),
        "{:?}",
        output.diagnostics
    );
    let sites = output.message_sites();
    assert_eq!(
        sites[0].visible_variables["count"],
        storyscript_parser::ast::VarType::Integer
    );
    assert_eq!(
        sites[0].visible_variables["local"],
        storyscript_parser::ast::VarType::Boolean
    );
    assert_eq!(
        sites[0].visible_variables["name"],
        storyscript_parser::ast::VarType::String
    );
}

fn assert_has_code(output: &storyscript_parser::compiler::CompileOutput, code: DiagnosticCode) {
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code),
        "expected {code}, got {:?}",
        output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn project_compile_merges_children_in_manifest_order() {
    let project = write_project();
    let output = compile_project(project.path(), Path::new("root.StoryScript"))
        .expect("project should be readable");

    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error())
    );
    let script = output.script.expect("valid script");
    let labels = script
        .scenes
        .iter()
        .map(|scene| scene.label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(labels, ["root_scene", "first_scene", "second_scene"]);
}

#[test]
fn compile_file_keeps_valid_include_callers_compatible() {
    let project = write_project();
    let output = compile_file(&project.path().join("root.StoryScript"))
        .expect("compatibility API should compile");

    assert!(output.script.is_some());
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.is_error())
    );
}

#[test]
fn project_compile_rejects_parent_traversal() {
    let project = tempfile::tempdir().expect("temporary project");
    fs::create_dir(project.path().join("scripts")).expect("scripts directory");
    fs::write(
        project.path().join("scripts/root.StoryScript"),
        ROOT_PREFIX.replace("modules/first.StoryScript", "../outside.StoryScript"),
    )
    .expect("root source");

    let output = compile_project(project.path(), Path::new("scripts/root.StoryScript"))
        .expect("root should be readable");
    assert_has_code(&output, DiagnosticCode::EIncludeFileNotFound);
}

#[test]
fn project_compile_rejects_absolute_include_paths() {
    let project = tempfile::tempdir().expect("temporary project");
    let source = ROOT_PREFIX
        .replace("modules/first.StoryScript", "/tmp/first.StoryScript")
        .replace(", \"modules/second.StoryScript\"", "");
    fs::write(project.path().join("root.StoryScript"), source).expect("root source");

    let output = compile_project(project.path(), Path::new("root.StoryScript"))
        .expect("root should be readable");
    assert_has_code(&output, DiagnosticCode::EIncludeFileNotFound);
}

#[cfg(unix)]
#[test]
fn project_compile_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let project = tempfile::tempdir().expect("temporary project");
    let outside = tempfile::tempdir().expect("outside directory");
    fs::write(outside.path().join("child.StoryScript"), FIRST_CHILD).expect("outside child");
    symlink(
        outside.path().join("child.StoryScript"),
        project.path().join("escape.StoryScript"),
    )
    .expect("symlink");
    let source = ROOT_PREFIX
        .replace("modules/first.StoryScript", "escape.StoryScript")
        .replace(", \"modules/second.StoryScript\"", "");
    fs::write(project.path().join("root.StoryScript"), source).expect("root source");

    let output = compile_project(project.path(), Path::new("root.StoryScript"))
        .expect("root should be readable");
    assert_has_code(&output, DiagnosticCode::EIncludeFileNotFound);
}

#[test]
fn project_compile_rejects_case_folded_duplicate_paths() {
    let project = tempfile::tempdir().expect("temporary project");
    fs::write(
        project.path().join("root.StoryScript"),
        ROOT_PREFIX
            .replace("modules/first.StoryScript", "Child.StoryScript")
            .replace("modules/second.StoryScript", "child.StoryScript"),
    )
    .expect("root source");
    fs::write(project.path().join("Child.StoryScript"), FIRST_CHILD).expect("first child");
    fs::write(project.path().join("child.StoryScript"), SECOND_CHILD).expect("second child");

    let output = compile_project(project.path(), Path::new("root.StoryScript"))
        .expect("root should be readable");
    assert_has_code(&output, DiagnosticCode::EIncludeDuplicatePath);
}

#[test]
fn project_compile_rejects_non_utf8_child_source() {
    let project = tempfile::tempdir().expect("temporary project");
    let source = ROOT_PREFIX.replace(", \"modules/second.StoryScript\"", "");
    fs::create_dir(project.path().join("modules")).expect("modules directory");
    fs::write(project.path().join("root.StoryScript"), source).expect("root source");
    fs::write(
        project.path().join("modules/first.StoryScript"),
        [0xff, 0xfe, 0xfd],
    )
    .expect("invalid source bytes");

    let output = compile_project(project.path(), Path::new("root.StoryScript"))
        .expect("root should be readable");
    assert_has_code(&output, DiagnosticCode::EIncludeFileNotFound);
}

#[test]
fn missing_init_is_a_hard_failure_with_init_count() {
    let output = compile_source(r#"* scene { #STORY "missing init" @end }"#);

    assert!(output.script.is_none());
    assert_has_code(&output, DiagnosticCode::EInitCount);
}

#[test]
fn repeated_init_is_a_hard_failure_with_init_count() {
    let output = compile_source(
        r#"
        * INIT { @start scene; }
        * INIT { @start scene; }
        * scene { #STORY "duplicate" @end }
        "#,
    );

    assert!(output.script.is_none());
    assert_has_code(&output, DiagnosticCode::EInitCount);
}

#[test]
fn out_of_order_init_is_a_hard_failure_with_init_order() {
    let output = compile_source(
        r#"
        logic noop() { return; }
        * INIT { @start scene; }
        * scene { #STORY "late init" @end }
        "#,
    );

    assert!(output.script.is_none());
    assert_has_code(&output, DiagnosticCode::EInitOrder);
}
