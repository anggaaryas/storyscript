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
mod localization_support;

#[test]
fn strict_english_indonesian_catalogs_bind_visible_scalar_types() {
    let root = localization_support::project();
    let compiled = storyscript_bundle::project::compile(root.path()).unwrap();
    assert_eq!(compiled.catalogs.len(), 2);
    assert_eq!(
        compiled.story.localization.as_ref().unwrap().default_locale,
        "en"
    );
    assert_eq!(
        storyscript_bundle::localization::message_contracts(&compiled.story).unwrap()["greeting"]
            .iter()
            .map(|arg| arg.name.as_str())
            .collect::<Vec<_>>(),
        ["name"]
    );
    assert!(!compiled.catalogs[0].canonical.contains("Author comment"));
}

#[test]
fn incomplete_unused_duplicate_drift_out_of_scope_array_and_unsafe_catalogs_fail() {
    use localization_support::{project, replace};
    for (from, to) in [
        ("greeting =", "unused ="),
        ("{ $name }", "{ $count }"),
        ("{ $name }", "{ $unknown }"),
        ("{ $name }", "{ DATETIME($name) }"),
        ("{ $name }", "{ CUSTOM($name) }"),
        ("{ -brand }", "{ greeting }"),
    ] {
        let root = project();
        replace(root.path(), "localization/id.ftl", from, to);
        assert!(
            storyscript_bundle::project::compile(root.path()).is_err(),
            "{to}"
        );
    }
    for suffix in [
        "\nunused = Obsolete\n",
        "\ngreeting = Duplicate\n",
        "\n    .attribute = Unsupported\n",
    ] {
        let root = project();
        let path = root.path().join("localization/en.ftl");
        std::fs::write(
            &path,
            format!("{}{suffix}", std::fs::read_to_string(&path).unwrap()),
        )
        .unwrap();
        assert!(storyscript_bundle::project::compile(root.path()).is_err());
    }
    let root = project();
    replace(
        root.path(),
        "story/main.StoryScript",
        "$name as string = \"Ada\"",
        "$name as array<string> = [\"Ada\"]",
    );
    assert!(storyscript_bundle::project::compile(root.path()).is_err());
    let root = project();
    replace(
        root.path(),
        "StoryScript.toml",
        "\"en\", \"id\"",
        "\"en\", \"EN\"",
    );
    assert!(storyscript_bundle::project::compile(root.path()).is_err());
    let root = project();
    replace(
        root.path(),
        "StoryScript.toml",
        "root = \"localization\"",
        "root = \"../localization\"",
    );
    assert!(storyscript_bundle::project::compile(root.path()).is_err());
    let root = project();
    for locale in ["en", "id"] {
        replace(
            root.path(),
            &format!("localization/{locale}.ftl"),
            "{ $name }",
            "{ NUMBER($name) }",
        );
    }
    assert!(
        storyscript_bundle::project::compile(root.path()).is_err(),
        "NUMBER must reject strings during validation"
    );
    let root = project();
    for locale in ["en", "id"] {
        replace(
            root.path(),
            &format!("localization/{locale}.ftl"),
            "{ $name }",
            "{ $local }",
        );
    }
    replace(
        root.path(),
        "story/main.StoryScript",
        "@\"greeting\";",
        "\"First\";",
    );
    replace(
        root.path(),
        "story/main.StoryScript",
        "\"Plain ${count}\";",
        "@\"greeting\";",
    );
    assert!(
        storyscript_bundle::project::compile(root.path()).is_err(),
        "scene-local argument must not escape its scene"
    );
}

#[test]
fn transitive_reference_cycles_and_expansion_bombs_are_bounded() {
    use storyscript_bundle::localization::parse_catalog;
    let term_bomb = format!(
        "-term = {}\na = {{ -term(value: \"{}\") }}\n",
        "{ $value } ".repeat(90),
        "x".repeat(20_000)
    );
    assert!(
        parse_catalog("en", &term_bomb, "en.ftl").is_err(),
        "literal term bindings must be multiplied by their use count before formatting"
    );
    assert!(parse_catalog("en", "🙂 = Invalid\n", "en.ftl").is_err());
    assert!(parse_catalog("en", "a = { b }\nb = { a }\n", "en.ftl").is_err());
    assert!(parse_catalog("en", "a = { -t }\n-t = { a }\n", "en.ftl").is_err());
    let mut source = String::from("m0 = seed\n");
    for n in 1..22 {
        source.push_str(&format!("m{n} = {{ m{} }} {{ m{} }}\n", n - 1, n - 1));
    }
    assert!(parse_catalog("en", &source, "en.ftl").is_err());
    let mut chain = String::from("m000 = seed\n");
    for n in 1..150 {
        chain.push_str(&format!("m{n:03} = {{ m{:03} }}\n", n - 1));
    }
    assert!(
        parse_catalog("en", &chain, "en.ftl").is_err(),
        "memoized dependencies must still enforce expanded depth"
    );
    assert!(
        parse_catalog(
            "en",
            "a = { NUMBER($n, maximumFractionDigits: 999) }\n",
            "en.ftl"
        )
        .is_err()
    );
    assert!(parse_catalog("en", "a = { NUMBER(9007199254740993) }\n", "en.ftl").is_err());
    assert!(
        parse_catalog(
            "en",
            "-term = { $style }\na = { -term(style: \"short\") }\n",
            "en.ftl"
        )
        .is_ok()
    );
}

#[cfg(unix)]
#[test]
fn catalog_symlink_escape_and_case_alias_are_rejected() {
    use std::os::unix::fs::symlink;
    let root = localization_support::project();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("id.ftl"), "greeting = Secret").unwrap();
    std::fs::remove_file(root.path().join("localization/id.ftl")).unwrap();
    symlink(
        outside.path().join("id.ftl"),
        root.path().join("localization/id.ftl"),
    )
    .unwrap();
    assert!(storyscript_bundle::project::compile(root.path()).is_err());
    let root = localization_support::project();
    std::fs::rename(
        root.path().join("localization/en.ftl"),
        root.path().join("localization/EN.ftl"),
    )
    .unwrap();
    assert!(storyscript_bundle::project::compile(root.path()).is_err());
}
