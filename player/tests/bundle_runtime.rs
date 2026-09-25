#![cfg(feature = "storybundle-runtime")]

use std::path::Path;

use storyscript_bundle::limits::ResourceLimits;
use storyscript_bundle::loader::{VerificationPolicy, load};
use storyscript_bundle::trust::{TrustStore, key_id_for_bytes};
use storyscript_player::SemanticPlayer;
use storyscript_player::contract::{HARD_LIMITS, Origin, SemanticEvent, SessionStatus};

const BUNDLE: &[u8] = include_bytes!("../../storyscript_bundle/example/assets/demo.storybundle");
const KEY_HEX: &str = include_str!("../../storyscript_bundle/example/assets/demo_public_key.txt");

fn loaded() -> storyscript_bundle::loader::LoadedBundle {
    let text = KEY_HEX.trim();
    let mut key = [0u8; 32];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap();
    }
    let mut trust = TrustStore::new();
    trust.insert(key_id_for_bytes(&key), key).unwrap();
    load(
        BUNDLE,
        &trust,
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .unwrap()
}

#[test]
fn verified_bundle_and_source_use_lockstep_semantics_but_distinct_origins() {
    let bundle = loaded();
    let mut bundled = SemanticPlayer::from_loaded_bundle(&bundle, HARD_LIMITS).unwrap();
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../bundle/rust/tests/fixtures/demo_project/story/main.StoryScript");
    let mut source = SemanticPlayer::from_file(&source_path, HARD_LIMITS).unwrap();
    assert!(matches!(bundled.origin(), Origin::Bundle { .. }));
    assert!(matches!(source.origin(), Origin::Source { .. }));
    assert_eq!(bundled.current().current, source.current().current);
    assert_eq!(bundled.current().effects, source.current().effects);
    while bundled.current().status == SessionStatus::Active {
        assert_eq!(
            bundled.advance().unwrap().current,
            source.advance().unwrap().current
        );
    }
    assert!(matches!(bundled.current().current, SemanticEvent::End));

    let source_save = source.export_save().unwrap();
    assert_eq!(
        SemanticPlayer::restore_loaded_bundle(&bundle, &source_save, HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_INCOMPATIBLE"
    );
}

#[test]
fn bundle_save_roundtrip_requires_exact_verified_identity() {
    let bundle = loaded();
    let mut player = SemanticPlayer::from_loaded_bundle(&bundle, HARD_LIMITS).unwrap();
    player.advance().unwrap();
    let save = player.export_save().unwrap();
    let mut restored = SemanticPlayer::restore_loaded_bundle(&bundle, &save, HARD_LIMITS).unwrap();
    assert_eq!(restored.current(), player.current());
    assert_eq!(restored.advance().unwrap(), player.advance().unwrap());
}
