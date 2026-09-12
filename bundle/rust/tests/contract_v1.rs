use std::collections::BTreeSet;

use prost::Message;
use prost_types::{DescriptorProto, FileDescriptorSet};
use storyscript_bundle::config::ProjectConfig;
use storyscript_bundle::contract::validate_story;
use storyscript_bundle::proto::storybundle::v1 as pb;
use storyscript_bundle::{COMPILER_VERSION, FORMAT_VERSION, schema};

fn literal(value: &str) -> pb::InterpolatedString {
    pb::InterpolatedString {
        segments: vec![pb::StringSegment {
            value: Some(pb::string_segment::Value::Literal(value.to_string())),
        }],
    }
}

fn valid_story() -> pb::CompiledStory {
    pb::CompiledStory {
        format_version: FORMAT_VERSION,
        project: Some(pb::ProjectMetadata {
            id: "demo.story".to_string(),
            name: "Demo".to_string(),
            version: "1.0.0".to_string(),
        }),
        initialization: Some(pb::Initialization {
            variables: vec![pb::VariableDefinition {
                name: "ratio".to_string(),
                r#type: pb::VariableType::Decimal.into(),
                value: Some(pb::Expression {
                    value: Some(pb::expression::Value::Decimal(pb::DecimalValue {
                        canonical: "1.2300".to_string(),
                    })),
                }),
            }],
            actors: vec![pb::Actor {
                id: "NARRATOR".to_string(),
                display_name: Some(literal("Narrator")),
                portraits: Vec::new(),
            }],
            start_scene: "opening".to_string(),
        }),
        logic_blocks: Vec::new(),
        scenes: vec![pb::Scene {
            label: "opening".to_string(),
            prep: None,
            story: Some(pb::StoryBlock {
                statements: vec![pb::StoryStatement {
                    value: Some(pb::story_statement::Value::End(())),
                }],
            }),
        }],
    }
}

#[test]
fn descriptor_fingerprint_matches_checked_contract() {
    assert_eq!(
        schema::check().expect("schema check"),
        schema::descriptor_sha256()
    );
    assert_eq!(schema::descriptor_sha256(), schema::EXPECTED_SHA256.trim());
}

#[test]
fn project_config_requires_the_exact_compiler_pin() {
    let valid = format!(
        r#"
[project]
id = "demo.story"
name = "Demo"
version = "1.2.3"
entry = "story/main.StoryScript"
compiler-version = "{COMPILER_VERSION}"

[assets]
root = "assets"
literal-policy = "required"
dynamic-files = ["portraits/hero.png"]
dynamic-globs = ["backgrounds/day-*.png"]
"#
    );
    let config = ProjectConfig::parse(&valid).expect("valid exact pin");
    assert_eq!(config.project.compiler_version, COMPILER_VERSION);

    let mismatch = valid.replace(
        &format!("compiler-version = \"{COMPILER_VERSION}\""),
        "compiler-version = \"99.0.0\"",
    );
    let error = ProjectConfig::parse(&mismatch).expect_err("mismatch must fail");
    assert_eq!(error.code().to_string(), "B_COMPILER_MISMATCH");
}

#[test]
fn project_config_never_accepts_signing_key_fields() {
    let source = format!(
        r#"
[project]
id = "demo.story"
name = "Demo"
version = "1.0.0"
entry = "main.StoryScript"
compiler-version = "{COMPILER_VERSION}"
signing-key = "private.pem"
"#
    );
    assert!(ProjectConfig::parse(&source).is_err());
}

#[test]
fn protobuf_decode_does_not_bypass_required_invariants() {
    let bytes = pb::CompiledStory::default().encode_to_vec();
    let decoded = pb::CompiledStory::decode(bytes.as_slice()).expect("proto3 decode");
    assert!(validate_story(&decoded).is_err());
}

#[test]
fn decimal_scale_round_trips_losslessly() {
    let story = valid_story();
    validate_story(&story).expect("valid semantic story");
    let bytes = story.encode_to_vec();
    let decoded = pb::CompiledStory::decode(bytes.as_slice()).expect("decode");
    let expression = decoded
        .initialization
        .unwrap()
        .variables
        .remove(0)
        .value
        .unwrap();
    match expression.value.unwrap() {
        pb::expression::Value::Decimal(value) => assert_eq!(value.canonical, "1.2300"),
        other => panic!("expected decimal, found {other:?}"),
    }
}

#[test]
fn every_ast_family_has_an_explicit_schema_message() {
    let descriptor = FileDescriptorSet::decode(schema::DESCRIPTOR_SET).expect("descriptor set");
    let mut names = BTreeSet::new();
    for file in descriptor.file {
        if file.package.as_deref() == Some("storybundle.v1") {
            collect_message_names("", &file.message_type, &mut names);
        }
    }

    for required in [
        "Initialization",
        "VariableDefinition",
        "Actor",
        "Portrait",
        "LogicBlock",
        "PrepStatement",
        "BackgroundDirective",
        "BgmDirective",
        "SfxDirective",
        "VariableAssignment",
        "PrepIfElse",
        "PrepForSnapshot",
        "PrepRepeat",
        "ReturnStatement",
        "StoryStatement",
        "Narration",
        "VariableOutput",
        "Dialogue",
        "StoryIfElse",
        "ChoiceBlock",
        "ChoiceOption",
        "ChoiceIf",
        "ChoiceRepeat",
        "ChoiceForSnapshot",
        "Jump",
        "StoryForSnapshot",
        "StoryRepeat",
        "RepeatCount",
        "Expression",
        "DecimalValue",
        "VariableReference",
        "BinaryExpression",
        "CallExpression",
        "ListExpression",
        "InterpolatedString",
    ] {
        assert!(
            names.contains(required),
            "missing schema message {required}"
        );
    }
}

#[test]
fn schema_excludes_source_only_fields() {
    let descriptor = FileDescriptorSet::decode(schema::DESCRIPTOR_SET).expect("descriptor set");
    let forbidden = [
        "line",
        "column",
        "filename",
        "file_name",
        "source",
        "include_path",
        "require",
        "comment",
    ];
    for file in descriptor.file {
        if file.package.as_deref() != Some("storybundle.v1") {
            continue;
        }
        visit_fields(&file.message_type, &mut |field| {
            let name = field.name.as_deref().unwrap_or_default();
            assert!(
                !forbidden.contains(&name),
                "source-only field {name} leaked"
            );
        });
    }
}

fn collect_message_names(prefix: &str, messages: &[DescriptorProto], names: &mut BTreeSet<String>) {
    for message in messages {
        let name = message.name.as_deref().unwrap_or_default();
        let qualified = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}.{name}")
        };
        names.insert(qualified.clone());
        collect_message_names(&qualified, &message.nested_type, names);
    }
}

fn visit_fields(
    messages: &[DescriptorProto],
    visitor: &mut impl FnMut(&prost_types::FieldDescriptorProto),
) {
    for message in messages {
        for field in &message.field {
            visitor(field);
        }
        visit_fields(&message.nested_type, visitor);
    }
}
