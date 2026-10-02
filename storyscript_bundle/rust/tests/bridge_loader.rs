use std::path::PathBuf;

use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use pkcs8::LineEnding;
use storyscript_bundle::api::bundle::{
    BridgeLimits, BridgeTrustKey, BridgeVerificationPolicy, bridge_hard_limits, bundle_dispose,
    bundle_open_bytes, bundle_read_asset,
};
use storyscript_bundle_core::exporter;
use storyscript_bundle_core::signing::{BundleSigner, Ed25519Signer};

#[test]
fn strict_open_owns_verified_bytes_and_disposal_is_idempotent() {
    let (bytes, trust_key) = signed_fixture();
    let result = bundle_open_bytes(
        bytes,
        vec![trust_key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
    );
    assert!(result.error.is_none());
    let opened = result.opened.expect("verified bundle");
    assert_eq!(opened.manifest.project.id, "storybundle.demo");
    assert!(!opened.verification.is_unsigned_development);
    assert!(!opened.compiled_story.is_empty());

    let asset = bundle_read_asset(
        &opened.resource,
        "portraits/hero.svg".to_string(),
        64 * 1024,
    );
    assert!(asset.error.is_none());
    assert!(asset.bytes.expect("asset bytes").starts_with(b"<svg"));

    assert!(bundle_dispose(&opened.resource).released);
    assert!(!bundle_dispose(&opened.resource).released);
    let disposed = bundle_read_asset(
        &opened.resource,
        "portraits/hero.svg".to_string(),
        64 * 1024,
    );
    assert_eq!(
        disposed.error.expect("disposed error").code,
        "B_RESOURCE_DISPOSED"
    );
}

#[test]
fn bridge_rejects_invalid_trust_material_and_preflights_limits() {
    let invalid_key = bundle_open_bytes(
        vec![],
        vec![BridgeTrustKey {
            public_key: vec![7; 31],
            expected_key_id: None,
        }],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
    );
    assert_eq!(
        invalid_key.error.expect("key error").code,
        "B_SIGNING_KEY_INVALID"
    );

    let limits = BridgeLimits {
        max_archive_bytes: 3,
        ..bridge_hard_limits()
    };
    let oversized = bundle_open_bytes(vec![0; 4], vec![], BridgeVerificationPolicy::Strict, limits);
    assert_eq!(
        oversized.error.expect("limit error").code,
        "B_RESOURCE_LIMIT"
    );
}

#[test]
fn bounded_asset_reads_fail_before_allocating() {
    let (bytes, trust_key) = signed_fixture();
    let opened = bundle_open_bytes(
        bytes,
        vec![trust_key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
    )
    .opened
    .expect("verified bundle");
    let result = bundle_read_asset(&opened.resource, "portraits/hero.svg".to_string(), 1);
    assert_eq!(
        result.error.expect("bounded read error").code,
        "B_RESOURCE_LIMIT"
    );
}

fn signed_fixture() -> (Vec<u8>, BridgeTrustKey) {
    signed_project("demo_project")
}

fn signed_project(name: &str) -> (Vec<u8>, BridgeTrustKey) {
    let private_key = SigningKey::from_bytes(&[31; 32]);
    let pem = private_key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("PKCS#8 PEM");
    let signer = Ed25519Signer::from_pkcs8_pem(pem.as_str()).expect("signer");
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bundle/rust/tests/fixtures")
        .join(name);
    let bytes = exporter::export(&project, &signer).expect("fixture export");
    let trust_key = BridgeTrustKey {
        public_key: signer.public_key_bytes().to_vec(),
        expected_key_id: Some(signer.key_id()),
    };
    (bytes, trust_key)
}

#[test]
fn localized_inspector_metadata_does_not_copy_catalog_bodies() {
    use prost::Message;
    use storyscript_bundle_core::proto::storybundle::v1::CompiledStory;
    let (bytes, key) = signed_project("localized_project");
    let opened = bundle_open_bytes(
        bytes,
        vec![key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
    )
    .opened
    .unwrap();
    let story = CompiledStory::decode(opened.compiled_story.as_slice()).unwrap();
    let locales = story.localization.unwrap();
    assert_eq!(locales.default_locale, "en");
    assert_eq!(locales.supported_locales, ["en", "id"]);
    for prose in [
        b"Hello".as_slice(),
        b"Halo".as_slice(),
        b"Author comment".as_slice(),
    ] {
        assert!(
            !opened
                .compiled_story
                .windows(prose.len())
                .any(|s| s == prose)
        );
    }
    let catalog = bundle_read_asset(&opened.resource, "localization/id.ftl".into(), 1 << 20);
    assert!(catalog.bytes.is_none()); // Catalogs are not public assets.
    assert!(catalog.error.is_some());
}
