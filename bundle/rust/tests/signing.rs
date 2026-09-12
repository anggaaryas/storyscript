use ed25519_dalek::pkcs8::EncodePrivateKey;
use ed25519_dalek::{Signature, SigningKey, Verifier};
use pkcs8::LineEnding;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use storyscript_bundle::signing::{BundleSigner, Ed25519Signer, signing_digest};

#[test]
fn pkcs8_signer_has_stable_key_id_and_verifiable_domain_signature() {
    let key = SigningKey::generate(&mut OsRng);
    let pem = key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("PKCS#8")
        .to_string();
    let signer = Ed25519Signer::from_pkcs8_pem(&pem).expect("signer");
    let manifest = br#"{"format_version":1}
"#;
    let signature = signer.sign_manifest(manifest).expect("signature");

    assert_eq!(
        signer.key_id(),
        hex::encode(Sha256::digest(key.verifying_key().to_bytes()))
    );
    key.verifying_key()
        .verify(
            &signing_digest(manifest),
            &Signature::from_bytes(&signature),
        )
        .expect("valid signature");
}

#[test]
fn invalid_pkcs8_is_rejected_without_panic() {
    let error = match Ed25519Signer::from_pkcs8_pem("not a private key") {
        Ok(_) => panic!("invalid key accepted"),
        Err(error) => error,
    };
    assert_eq!(error.code().to_string(), "B_SIGNING_KEY_INVALID");
}
