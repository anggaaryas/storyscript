mod common;

use storyscript_bundle::limits::ResourceLimits;
use storyscript_bundle::loader::{VerificationPolicy, load};

use common::TestBundle;

#[test]
fn authenticated_format_compiler_and_schema_mismatches_are_distinct() {
    let bundle = TestBundle::new();
    let cases = [
        (
            bundle.rewrite_manifest(|manifest| manifest.format_version = 99, true),
            "B_UNSUPPORTED_FORMAT",
        ),
        (
            bundle.rewrite_manifest(
                |manifest| manifest.compiler_version = "99.0.0".to_string(),
                true,
            ),
            "B_COMPILER_MISMATCH",
        ),
        (
            bundle.rewrite_manifest(|manifest| manifest.schema_sha256 = "0".repeat(64), true),
            "B_SCHEMA_MISMATCH",
        ),
    ];

    for (bytes, expected_code) in cases {
        let error = load(
            &bytes,
            &bundle.trust_store(),
            VerificationPolicy::Strict,
            ResourceLimits::HARD,
        )
        .expect_err("incompatible identity must fail");
        assert_eq!(error.code().to_string(), expected_code);
    }
}

#[test]
fn compatibility_fields_are_not_reported_before_signature_authentication() {
    let bundle = TestBundle::new();
    let untrusted = bundle.rewrite_manifest(|manifest| manifest.format_version = 99, false);
    let error = load(
        &untrusted,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("bad signature must win");
    assert_eq!(error.code().to_string(), "B_BAD_SIGNATURE");
}
