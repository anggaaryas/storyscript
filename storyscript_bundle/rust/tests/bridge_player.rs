use std::path::PathBuf;

use ed25519_dalek::pkcs8::EncodePrivateKey;
use ed25519_dalek::SigningKey;
use pkcs8::LineEnding;
use rand::rngs::OsRng;
use storyscript_bundle::api::bundle::{
    bridge_hard_limits, bundle_dispose, bundle_open_bytes, BridgeTrustKey, BridgeVerificationPolicy,
};
use storyscript_bundle::api::player::*;
use storyscript_bundle_core::exporter;
use storyscript_bundle_core::signing::{BundleSigner, Ed25519Signer};

#[test]
fn existing_verified_resource_creates_isolated_players_and_parent_first_disposal_is_safe() {
    let (bytes, trust_key) = signed_fixture();
    let opened_bundle = bundle_open_bytes(
        bytes,
        vec![trust_key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
    )
    .opened
    .expect("bundle");
    let first =
        bundle_player_open_from_bundle(&opened_bundle.resource, bundle_player_hard_limits())
            .opened
            .expect("player");
    let second =
        bundle_player_open_from_bundle(&opened_bundle.resource, bundle_player_hard_limits())
            .opened
            .expect("player");
    assert!(bundle_dispose(&opened_bundle.resource).released);

    assert_eq!(
        bundle_player_advance(&first.resource)
            .delta
            .unwrap()
            .event
            .kind,
        "dialogue"
    );
    assert_eq!(
        bundle_player_current(&second.resource)
            .delta
            .unwrap()
            .event
            .kind,
        "scene"
    );
    let asset = bundle_player_read_asset(&first.resource, "portraits/hero.svg".into(), 64 * 1024);
    assert!(asset.bytes.unwrap().starts_with(b"<svg"));
    assert!(bundle_player_dispose(&first.resource).released);
    assert_eq!(
        bundle_player_current(&first.resource).error.unwrap().code,
        "R_PLAYER_DISPOSED"
    );
    assert!(bundle_player_dispose(&second.resource).released);
}

#[test]
fn fused_open_save_restore_and_bounds_do_not_expose_compiled_model() {
    let (bytes, trust_key) = signed_fixture();
    let opened = bundle_player_open_bytes(
        bytes.clone(),
        vec![trust_key.clone()],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        bundle_player_hard_limits(),
    )
    .opened
    .expect("fused player");
    bundle_player_advance(&opened.resource);
    let save = bundle_player_export_save(&opened.resource).bytes.unwrap();
    let restored = bundle_player_restore_bytes(
        bytes,
        save,
        vec![trust_key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        bundle_player_hard_limits(),
    )
    .opened
    .expect("restored");
    assert_eq!(
        bundle_player_current(&restored.resource)
            .delta
            .unwrap()
            .event
            .kind,
        "dialogue"
    );
    assert_eq!(
        bundle_player_read_asset(&restored.resource, "portraits/hero.svg".into(), 1)
            .error
            .unwrap()
            .code,
        "B_RESOURCE_LIMIT"
    );
}

#[test]
fn failed_verification_and_invalid_runtime_limits_expose_no_player() {
    let (mut bytes, trust_key) = signed_fixture();
    bytes[0] ^= 1;
    let failed = bundle_player_open_bytes(
        bytes,
        vec![trust_key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        bundle_player_hard_limits(),
    );
    assert!(failed.opened.is_none());
    assert!(failed.error.is_some());

    let (bytes, trust_key) = signed_fixture();
    let mut limits = bundle_player_hard_limits();
    limits.history_entries = 0;
    let failed = bundle_player_open_bytes(
        bytes,
        vec![trust_key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        limits,
    );
    assert!(failed.opened.is_none());
    assert_eq!(failed.error.unwrap().code, "R_LIMIT_CONFIGURATION");
}

fn signed_fixture() -> (Vec<u8>, BridgeTrustKey) {
    let private_key = SigningKey::generate(&mut OsRng);
    let pem = private_key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("PKCS#8 PEM");
    let signer = Ed25519Signer::from_pkcs8_pem(pem.as_str()).expect("signer");
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bundle/rust/tests/fixtures/demo_project");
    let bytes = exporter::export(&project, &signer).expect("fixture export");
    let trust_key = BridgeTrustKey {
        public_key: signer.public_key_bytes().to_vec(),
        expected_key_id: Some(signer.key_id()),
    };
    (bytes, trust_key)
}
