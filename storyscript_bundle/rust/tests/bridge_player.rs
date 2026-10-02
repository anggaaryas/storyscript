use std::path::PathBuf;

use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use pkcs8::LineEnding;
use storyscript_bundle::api::bundle::{
    BridgeTrustKey, BridgeVerificationPolicy, bridge_hard_limits, bundle_dispose, bundle_open_bytes,
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
    let first = bundle_player_open_from_bundle(
        &opened_bundle.resource,
        bundle_player_hard_limits(),
        vec![],
    )
    .opened
    .expect("player");
    let second = bundle_player_open_from_bundle(
        &opened_bundle.resource,
        bundle_player_hard_limits(),
        vec![],
    )
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
        vec![],
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
        vec![],
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
        vec![],
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
        vec![],
    );
    assert!(failed.opened.is_none());
    assert_eq!(failed.error.unwrap().code, "R_LIMIT_CONFIGURATION");
}

fn signed_fixture() -> (Vec<u8>, BridgeTrustKey) {
    signed_project("demo_project")
}

fn signed_project(name: &str) -> (Vec<u8>, BridgeTrustKey) {
    // Deterministic in-memory test key only; not a shipped trust identity.
    let private_key = SigningKey::from_bytes(&[29; 32]);
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
fn localized_fused_bytes_and_path_restore_rerender_without_model_or_catalog_transfer() {
    let (bytes, key) = signed_project("localized_project");
    let opened = bundle_player_open_bytes(
        bytes.clone(),
        vec![key.clone()],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        bundle_player_hard_limits(),
        vec!["id-ID".into(), "en".into()],
    )
    .opened
    .unwrap();
    assert_eq!(opened.resolved_locale.as_deref(), Some("id"));
    assert!(!opened.has_unresolved_localization);
    assert!(
        bundle_player_advance(&opened.resource)
            .delta
            .unwrap()
            .event
            .text
            .unwrap()
            .starts_with("Halo")
    );
    let current = bundle_player_advance(&opened.resource).delta.unwrap();
    assert!(current.event.text.unwrap().contains("barang"));
    let save = bundle_player_export_save(&opened.resource).bytes.unwrap();
    let restored = bundle_player_restore_bytes(
        bytes.clone(),
        save.clone(),
        vec![key.clone()],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        bundle_player_hard_limits(),
        vec!["en".into()],
    )
    .opened
    .unwrap();
    assert_eq!(restored.resolved_locale.as_deref(), Some("en"));
    assert_eq!(restored.current.sequence, current.sequence);
    assert!(restored.current.event.text.unwrap().contains("items"));
    assert!(
        bundle_player_history(&restored.resource, 0, 256)
            .page
            .unwrap()
            .entries
            .iter()
            .any(|e| e
                .event
                .text
                .as_deref()
                .is_some_and(|s| s.starts_with("Hello")))
    );
    assert_eq!(
        bundle_player_advance(&restored.resource)
            .delta
            .unwrap()
            .event
            .choices[0]
            .text,
        "Continue"
    );
    assert_eq!(
        bundle_player_export_save(&opened.resource).bytes.unwrap(),
        save
    );

    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("localized.storybundle");
    std::fs::write(&path, &bytes).unwrap();
    let path = path.to_string_lossy().to_string();
    let fallback = bundle_player_open_path(
        path.clone(),
        vec![key.clone()],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        bundle_player_hard_limits(),
        vec!["fr-FR".into()],
    )
    .opened
    .unwrap();
    assert_eq!(fallback.resolved_locale.as_deref(), Some("en"));
    let path_restore = bundle_player_restore_path(
        path,
        save,
        vec![key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        bundle_player_hard_limits(),
        vec!["en".into(), "id".into()],
    )
    .opened
    .unwrap();
    assert_eq!(path_restore.resolved_locale.as_deref(), Some("en"));
    assert!(path_restore.current.event.text.unwrap().contains("items"));
}

#[test]
fn localized_existing_bundle_handoff_keeps_verified_catalog_lease_in_both_disposal_orders() {
    let (bytes, key) = signed_project("localized_project");
    let parent = bundle_open_bytes(
        bytes,
        vec![key],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
    )
    .opened
    .unwrap();
    let child = bundle_player_open_from_bundle(
        &parent.resource,
        bundle_player_hard_limits(),
        vec!["id".into()],
    )
    .opened
    .unwrap();
    bundle_player_advance(&child.resource);
    let save = bundle_player_export_save(&child.resource).bytes.unwrap();
    let english = bundle_player_restore_from_bundle(
        &parent.resource,
        save,
        bundle_player_hard_limits(),
        vec!["en".into()],
    )
    .opened
    .unwrap();
    assert_eq!(english.resolved_locale.as_deref(), Some("en"));
    assert!(english.current.event.text.unwrap().starts_with("Hello"));
    assert!(bundle_player_dispose(&english.resource).released);
    let third =
        bundle_player_open_from_bundle(&parent.resource, bundle_player_hard_limits(), vec![])
            .opened
            .unwrap();
    assert_eq!(third.resolved_locale.as_deref(), Some("en"));
    assert!(bundle_dispose(&parent.resource).released);
    assert!(
        bundle_player_advance(&child.resource)
            .delta
            .unwrap()
            .event
            .text
            .unwrap()
            .contains("barang")
    );
    assert_eq!(
        bundle_player_advance(&child.resource)
            .delta
            .unwrap()
            .event
            .choices[0]
            .text,
        "Lanjutkan"
    );
    assert!(
        bundle_player_open_from_bundle(&parent.resource, bundle_player_hard_limits(), vec![])
            .opened
            .is_none()
    );
    assert!(bundle_player_dispose(&child.resource).released);
    assert!(!bundle_player_dispose(&child.resource).released);
    assert!(bundle_player_dispose(&third.resource).released);
}

#[test]
fn catalog_tamper_and_invalid_preferences_never_publish_a_player() {
    let (bytes, key) = signed_project("localized_project");
    let invalid = bundle_player_open_bytes(
        bytes.clone(),
        vec![key.clone()],
        BridgeVerificationPolicy::Strict,
        bridge_hard_limits(),
        bundle_player_hard_limits(),
        vec!["id_ID".into()],
    );
    assert!(invalid.opened.is_none());
    assert_eq!(invalid.error.unwrap().code, "R_LOCALIZATION_LOCALE");
    // Rewrite a real catalog with valid ZIP CRCs but without updating the signed digest.
    use std::io::{Cursor, Read, Write};
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let mut body = Vec::new();
        entry.read_to_end(&mut body).unwrap();
        if entry.name() == "localization/id.ftl" {
            body[0] ^= 1;
        }
        output
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        output.write_all(&body).unwrap();
    }
    let tampered = output.finish().unwrap().into_inner();
    for policy in [
        BridgeVerificationPolicy::Strict,
        BridgeVerificationPolicy::UnsignedDevelopment,
    ] {
        let failure = bundle_player_open_bytes(
            tampered.clone(),
            vec![key.clone()],
            policy,
            bridge_hard_limits(),
            bundle_player_hard_limits(),
            vec!["id".into()],
        );
        assert!(failure.opened.is_none());
        assert!(failure.error.unwrap().code.starts_with("B_"));
    }
}
