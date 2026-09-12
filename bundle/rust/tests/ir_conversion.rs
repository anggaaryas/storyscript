use std::path::{Path, PathBuf};

use prost::Message;
use storyscript_bundle::config::Project;
use storyscript_bundle::ir::v1;
use storyscript_bundle::project;
use storyscript_bundle::proto::storybundle::v1 as pb;
use storyscript_bundle::template;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo_project")
}

#[test]
fn conversion_strips_source_only_data_and_preserves_interpolation_order() {
    let compiled = project::compile(&fixture()).expect("fixture compiles");
    let bytes = compiled.story.encode_to_vec();
    let debug_bytes = String::from_utf8_lossy(&bytes);
    assert!(!debug_bytes.contains("Authoring-only comment"));
    assert!(!debug_bytes.contains(".StoryScript"));

    let actor = &compiled.story.initialization.as_ref().unwrap().actors[0];
    let segments = &actor.display_name.as_ref().unwrap().segments;
    assert!(matches!(
        segments[0].value,
        Some(pb::string_segment::Value::Literal(ref value)) if value == "Hero "
    ));
    assert!(matches!(
        segments[1].value,
        Some(pb::string_segment::Value::Variable(ref value)) if value == "time_of_day"
    ));
}

#[test]
fn escaped_dollar_becomes_literal_data_not_placeholder_syntax() {
    let input = format!(
        "cost={}{{amount}} and ${{actual}}",
        storyscript_parser::interpolation::ESCAPED_DOLLAR_MARKER
    );
    let compiled = template::compile(&input).expect("template");
    assert!(matches!(
        compiled.segments[0].value,
        Some(pb::string_segment::Value::Literal(ref value)) if value == "cost=${amount} and "
    ));
    assert!(matches!(
        compiled.segments[1].value,
        Some(pb::string_segment::Value::Variable(ref value)) if value == "actual"
    ));
}

#[test]
fn current_advanced_examples_convert_without_source_metadata() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let examples = [
        "example/feature_logic_functions.StoryScript",
        "example/feature_loop_for_repeat_snapshot.StoryScript",
        "example/feature_array_collection_ops.StoryScript",
        "example/interpolation_variable_read.StoryScript",
    ];
    let metadata = Project {
        id: "example.coverage".to_string(),
        name: "Coverage".to_string(),
        version: "1.0.0".to_string(),
        entry: "unused.StoryScript".to_string(),
        compiler_version: storyscript_bundle::COMPILER_VERSION.to_string(),
    };

    for relative in examples {
        let output = storyscript_parser::compiler::compile_file(&repository.join(relative))
            .unwrap_or_else(|error| panic!("{relative}: {error}"));
        assert!(
            output
                .diagnostics
                .iter()
                .all(|diagnostic| !diagnostic.is_error()),
            "{relative}: {:?}",
            output
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        let converted = v1::convert(&output.script.unwrap(), &metadata)
            .unwrap_or_else(|error| panic!("{relative}: {error}"));
        assert!(!converted.scenes.is_empty(), "{relative}");
    }
}
