use std::fs;

use storyscript_bundle::assets::discover;
use storyscript_bundle::config::{Assets, LiteralAssetPolicy};
use storyscript_bundle::proto::storybundle::v1 as pb;

fn interpolation(parts: &[(&str, bool)]) -> pb::InterpolatedString {
    pb::InterpolatedString {
        segments: parts
            .iter()
            .map(|(value, variable)| pb::StringSegment {
                value: Some(if *variable {
                    pb::string_segment::Value::Variable((*value).to_string())
                } else {
                    pb::string_segment::Value::Literal((*value).to_string())
                }),
            })
            .collect(),
    }
}

fn story_with_portraits(paths: Vec<pb::InterpolatedString>) -> pb::CompiledStory {
    pb::CompiledStory {
        initialization: Some(pb::Initialization {
            actors: vec![pb::Actor {
                id: "HERO".to_string(),
                display_name: Some(interpolation(&[("Hero", false)])),
                portraits: paths
                    .into_iter()
                    .enumerate()
                    .map(|(index, path)| pb::Portrait {
                        emotion: format!("emotion{index}"),
                        asset_path: Some(path),
                    })
                    .collect(),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn default_assets() -> Assets {
    Assets {
        root: "assets".to_string(),
        literal_policy: LiteralAssetPolicy::Required,
        dynamic_files: Vec::new(),
        dynamic_globs: Vec::new(),
    }
}

#[test]
fn literal_and_dynamic_assets_form_a_sorted_closed_graph() {
    let root = tempfile::tempdir().expect("project");
    fs::create_dir_all(root.path().join("assets/backgrounds")).expect("asset directories");
    fs::write(root.path().join("assets/portrait.svg"), "portrait").expect("portrait");
    fs::write(root.path().join("assets/backgrounds/day.svg"), "day").expect("day");
    fs::write(root.path().join("assets/backgrounds/night.svg"), "night").expect("night");
    let mut config = default_assets();
    config.dynamic_globs = vec!["backgrounds/*.svg".to_string()];
    let story = story_with_portraits(vec![
        interpolation(&[("portrait.svg", false)]),
        interpolation(&[("backgrounds/", false), ("time", true), (".svg", false)]),
    ]);

    let assets = discover(root.path(), &config, &story).expect("closed graph");
    assert_eq!(
        assets
            .iter()
            .map(|asset| asset.logical_path.as_str())
            .collect::<Vec<_>>(),
        [
            "backgrounds/day.svg",
            "backgrounds/night.svg",
            "portrait.svg"
        ]
    );
}

#[test]
fn missing_literal_and_unmatched_dynamic_templates_fail_closed() {
    let root = tempfile::tempdir().expect("project");
    fs::create_dir(root.path().join("assets")).expect("assets");
    let missing = story_with_portraits(vec![interpolation(&[("missing.svg", false)])]);
    assert!(discover(root.path(), &default_assets(), &missing).is_err());

    fs::write(root.path().join("assets/configured.svg"), "configured").expect("asset");
    let mut config = default_assets();
    config.dynamic_files = vec!["configured.svg".to_string()];
    let dynamic = story_with_portraits(vec![interpolation(&[
        ("backgrounds/", false),
        ("time", true),
        (".svg", false),
    ])]);
    assert!(discover(root.path(), &config, &dynamic).is_err());
}

#[test]
fn traversal_and_case_fold_collisions_are_rejected() {
    let root = tempfile::tempdir().expect("project");
    fs::create_dir(root.path().join("assets")).expect("assets");
    fs::write(root.path().join("assets/Hero.svg"), "upper").expect("upper");
    fs::write(root.path().join("assets/hero.svg"), "lower").expect("lower");

    let traversal = story_with_portraits(vec![interpolation(&[("../outside.svg", false)])]);
    assert!(discover(root.path(), &default_assets(), &traversal).is_err());

    let collision = story_with_portraits(vec![
        interpolation(&[("Hero.svg", false)]),
        interpolation(&[("hero.svg", false)]),
    ]);
    assert!(discover(root.path(), &default_assets(), &collision).is_err());

    let source = story_with_portraits(vec![interpolation(&[("raw.StoryScript", false)])]);
    assert!(discover(root.path(), &default_assets(), &source).is_err());
}

#[cfg(unix)]
#[test]
fn symlink_escape_is_rejected() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("project");
    let outside = tempfile::tempdir().expect("outside");
    fs::create_dir(root.path().join("assets")).expect("assets");
    fs::write(outside.path().join("secret.svg"), "secret").expect("outside asset");
    symlink(
        outside.path().join("secret.svg"),
        root.path().join("assets/escape.svg"),
    )
    .expect("symlink");
    let story = story_with_portraits(vec![interpolation(&[("escape.svg", false)])]);

    assert!(discover(root.path(), &default_assets(), &story).is_err());
}
