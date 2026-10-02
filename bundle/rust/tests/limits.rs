mod common;

use prost::Message;
use storyscript_bundle::limits::ResourceLimits;
use storyscript_bundle::loader::{VerificationPolicy, load};
use storyscript_bundle::manifest::COMPILED_PATH;
use storyscript_bundle::proto::storybundle::v1 as pb;

use common::TestBundle;

#[test]
fn catalog_locale_id_and_independent_entry_limits_are_enforced() {
    let bundle = TestBundle::localized();
    let over = bundle.rewrite_payload("localization/en.ftl", vec![b'a'; (16 << 20) + 1]);
    assert_eq!(
        load(
            &over,
            &bundle.trust_store(),
            VerificationPolicy::Strict,
            ResourceLimits::HARD
        )
        .unwrap_err()
        .code()
        .to_string(),
        "B_RESOURCE_LIMIT"
    );
    let mut metadata = pb::LocalizationMetadata {
        default_locale: "en".into(),
        supported_locales: vec!["en".into(); 65],
    };
    assert!(storyscript_bundle::contract::validate_localization(&metadata).is_err());
    metadata.supported_locales.truncate(1);
    assert!(storyscript_bundle::contract::validate_localization(&metadata).is_ok());
    assert!(!storyscript_bundle::contract::valid_message_id(
        &"a".repeat(257)
    ));
    assert!(
        storyscript_bundle::localization::parse_catalog(
            "en",
            &"a".repeat((16 << 20) + 1),
            "en.ftl"
        )
        .is_err()
    );
    let mut source = String::new();
    for n in 0..=100_000 {
        source.push_str(&format!("m{n} = Text\n"));
    }
    assert!(storyscript_bundle::localization::parse_catalog("en", &source, "en.ftl").is_err());
}

#[test]
fn hosts_can_lower_but_never_raise_hard_limits() {
    let hard = ResourceLimits::HARD;
    let requested = ResourceLimits {
        max_archive_bytes: hard.max_archive_bytes * 2,
        max_total_uncompressed_bytes: 1234,
        max_entry_bytes: hard.max_entry_bytes * 2,
        max_compiled_ir_bytes: hard.max_compiled_ir_bytes * 2,
        max_manifest_bytes: hard.max_manifest_bytes * 2,
        max_entries: hard.max_entries * 2,
        max_path_bytes: hard.max_path_bytes * 2,
        max_semantic_depth: hard.max_semantic_depth * 2,
    };
    let effective = hard.lowered(requested);
    assert_eq!(effective.max_archive_bytes, hard.max_archive_bytes);
    assert_eq!(effective.max_total_uncompressed_bytes, 1234);
    assert_eq!(effective.max_entry_bytes, hard.max_entry_bytes);
    assert_eq!(effective.max_semantic_depth, hard.max_semantic_depth);
}

#[test]
fn preflight_rejects_archive_and_entry_sizes_before_content_exposure() {
    let bundle = TestBundle::new();
    let mut limits = ResourceLimits::HARD;
    limits.max_archive_bytes = bundle.bytes.len() as u64 - 1;
    let error = load(
        &bundle.bytes,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        limits,
    )
    .expect_err("archive boundary must fail");
    assert_eq!(error.code().to_string(), "B_RESOURCE_LIMIT");

    let mut limits = ResourceLimits::HARD;
    limits.max_entry_bytes = 1;
    let error = load(
        &bundle.bytes,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        limits,
    )
    .expect_err("entry boundary must fail");
    assert_eq!(error.code().to_string(), "B_RESOURCE_LIMIT");
}

#[test]
fn manifest_cannot_declare_a_profile_above_hard_limits() {
    let bundle = TestBundle::new();
    let oversized = bundle.rewrite_manifest(
        |manifest| {
            manifest.resource_limits.max_archive_bytes = ResourceLimits::HARD.max_archive_bytes + 1;
        },
        true,
    );
    let error = load(
        &oversized,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("declared profile must fail");
    assert_eq!(error.code().to_string(), "B_RESOURCE_LIMIT");
}

#[test]
fn signed_profile_can_lower_loader_limits() {
    let bundle = TestBundle::new();
    let lowered = bundle.rewrite_manifest(
        |manifest| manifest.resource_limits.max_entry_bytes = 1,
        true,
    );
    let error = load(
        &lowered,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("signed lower limit must be authoritative");
    assert_eq!(error.code().to_string(), "B_RESOURCE_LIMIT");
}

#[test]
fn lowered_semantic_depth_is_enforced_after_bounded_decode() {
    let bundle = TestBundle::new();
    let entries = common::read_entries(&bundle.bytes);
    let compiled = entries
        .iter()
        .find(|(path, _)| path == COMPILED_PATH)
        .unwrap();
    let mut story = pb::CompiledStory::decode(compiled.1.as_slice()).expect("compiled story");
    story.initialization.as_mut().unwrap().variables[0].value = Some(pb::Expression {
        value: Some(pb::expression::Value::Binary(Box::new(
            pb::BinaryExpression {
                left: Some(Box::new(pb::Expression {
                    value: Some(pb::expression::Value::Integer(1)),
                })),
                operator: pb::BinaryOperator::Add.into(),
                right: Some(Box::new(pb::Expression {
                    value: Some(pb::expression::Value::Integer(2)),
                })),
            },
        ))),
    });
    let nested = bundle.rewrite_payload(COMPILED_PATH, story.encode_to_vec());
    let mut limits = ResourceLimits::HARD;
    limits.max_semantic_depth = 1;
    let error = load(
        &nested,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        limits,
    )
    .expect_err("nested expression must exceed lowered limit");
    assert_eq!(error.code().to_string(), "B_RESOURCE_LIMIT");
}
