mod common;

use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::EncodePrivateKey;
use pkcs8::LineEnding;
use rand::rngs::OsRng;
use storyscript_bundle::limits::ResourceLimits;
use storyscript_bundle::loader::{VerificationPolicy, load};
use storyscript_bundle::manifest::{MANIFEST_PATH, SIGNATURE_PATH};
use storyscript_bundle::signing::{BundleSigner, Ed25519Signer};
use storyscript_bundle::trust::TrustStore;

use common::TestBundle;

#[test]
fn strict_policy_rejects_unknown_and_wrong_signatures() {
    let bundle = TestBundle::new();
    let error = load(
        &bundle.bytes,
        &TrustStore::new(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("unknown signer must fail");
    assert_eq!(error.code().to_string(), "B_UNKNOWN_SIGNER");

    let wrong_key = SigningKey::generate(&mut OsRng);
    let pem = wrong_key.to_pkcs8_pem(LineEnding::LF).expect("PKCS#8");
    let signer = Ed25519Signer::from_pkcs8_pem(pem.as_str()).expect("signer");
    let entries = common::read_entries(&bundle.bytes);
    let manifest = entries
        .iter()
        .find(|(path, _)| path == MANIFEST_PATH)
        .unwrap()
        .1
        .clone();
    let wrong_signature = signer.sign_manifest(&manifest).unwrap().to_vec();
    let tampered = bundle.rewrite(|path, bytes| {
        if path == SIGNATURE_PATH {
            *bytes = wrong_signature.clone();
        }
        true
    });
    let error = load(
        &tampered,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("wrong signature must fail");
    assert_eq!(error.code().to_string(), "B_BAD_SIGNATURE");
}

#[test]
fn manifest_and_asset_tampering_are_detected_at_their_boundaries() {
    let bundle = TestBundle::new();
    let manifest_tamper = bundle.rewrite_manifest(
        |manifest| manifest.project.name.push_str(" tampered"),
        false,
    );
    let error = load(
        &manifest_tamper,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("manifest tamper must fail");
    assert_eq!(error.code().to_string(), "B_BAD_SIGNATURE");

    let asset_tamper = bundle.rewrite(|path, bytes| {
        if path == "assets/portraits/hero.svg" {
            bytes[0] ^= 1;
        }
        true
    });
    let error = load(
        &asset_tamper,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("asset tamper must fail");
    assert_eq!(error.code().to_string(), "B_DIGEST_MISMATCH");
}

#[test]
fn unsigned_development_requires_an_explicit_policy_and_is_visible() {
    let bundle = TestBundle::new();
    let unsigned = bundle.rewrite(|path, _| path != SIGNATURE_PATH);
    let strict_error = load(
        &unsigned,
        &bundle.trust_store(),
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .expect_err("strict unsigned load must fail");
    assert_eq!(strict_error.code().to_string(), "B_UNSIGNED_DISALLOWED");

    let loaded = load(
        &unsigned,
        &TrustStore::new(),
        VerificationPolicy::UnsignedDevelopment,
        ResourceLimits::HARD,
    )
    .expect("explicit development load");
    assert!(loaded.verification_status().is_unsigned_development);
    assert!(loaded.verification_status().signer_key_id.is_none());
}
