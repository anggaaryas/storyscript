#![cfg(feature = "storybundle-runtime")]

use std::path::Path;

use storyscript_bundle::limits::ResourceLimits;
use storyscript_bundle::loader::{VerificationPolicy, load};
use storyscript_bundle::trust::{TrustStore, key_id_for_bytes};
use storyscript_player::SemanticPlayer;
use storyscript_player::contract::{
    HARD_LIMITS, MediaEffect, Origin, SemanticEvent, SessionStatus,
};

const BUNDLE: &[u8] = include_bytes!("../../storyscript_bundle/example/assets/demo.storybundle");
const KEY_HEX: &str = include_str!("../../storyscript_bundle/example/assets/demo_public_key.txt");
const GAME_BUNDLE: &[u8] =
    include_bytes!("../../storyscript_bundle/example/assets/station_nine.storybundle");
const GAME_KEY_HEX: &str =
    include_str!("../../storyscript_bundle/example/assets/station_nine_public_key.txt");

mod localization_support;
#[test]
fn signed_localized_bundle_and_source_project_are_equivalent_and_restore_cross_locale() {
    use localization_support::{prefs, project, signed};
    let root = project();
    let (bytes, trust) = signed(root.path());
    let bundle = load(
        &bytes,
        &trust,
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .unwrap();
    for locale in ["en", "id", "fr"] {
        let prefs = prefs(&[locale]);
        let mut source =
            SemanticPlayer::from_project_seeded(root.path(), &prefs, [17; 32], HARD_LIMITS)
                .unwrap();
        let mut bundled =
            SemanticPlayer::from_loaded_bundle_with_locales(&bundle, &prefs, HARD_LIMITS).unwrap();
        assert_eq!(source.resolved_locale(), bundled.resolved_locale());
        assert_eq!(source.current().current, bundled.current().current);
        for _ in 0..3 {
            assert_eq!(
                source.advance().unwrap().current,
                bundled.advance().unwrap().current
            );
        }
        let save = bundled.export_save().unwrap();
        let mut restored = SemanticPlayer::restore_loaded_bundle_with_locales(
            &bundle,
            &save,
            &localization_support::prefs(&["id"]),
            HARD_LIMITS,
        )
        .unwrap();
        assert_eq!(restored.resolved_locale(), Some("id"));
        assert!(
            matches!(&restored.current().current, SemanticEvent::Choices(v) if v[0].text == "Lanjutkan")
        );
        restored.choose(0).unwrap();
        restored.advance().unwrap();
        assert!(
            matches!(&restored.current().current, SemanticEvent::Narration(v) if v == "Plain 3")
        );
        source.choose(0).unwrap();
        bundled.choose(0).unwrap();
        assert_eq!(
            source.advance().unwrap().current,
            bundled.advance().unwrap().current
        );
    }
}

fn loaded() -> storyscript_bundle::loader::LoadedBundle {
    load_signed(BUNDLE, KEY_HEX)
}

fn load_signed(bytes: &[u8], text: &str) -> storyscript_bundle::loader::LoadedBundle {
    let text = text.trim();
    let mut key = [0u8; 32];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap();
    }
    let mut trust = TrustStore::new();
    trust.insert(key_id_for_bytes(&key), key).unwrap();
    load(
        bytes,
        &trust,
        VerificationPolicy::Strict,
        ResourceLimits::HARD,
    )
    .unwrap()
}

#[test]
fn station_nine_signed_game_has_distinct_playable_branches() {
    let bundle = load_signed(GAME_BUNDLE, GAME_KEY_HEX);
    for (first, second, expected) in [
        (0, 0, "Station Nine survives"),
        (0, 1, "station goes dark"),
        (1, 0, "Station Nine is lost"),
        (1, 1, "Station Nine is saved"),
        (1, 2, "station goes dark"),
    ] {
        let mut player = SemanticPlayer::from_loaded_bundle(&bundle, HARD_LIMITS).unwrap();
        let mut first_choice = false;
        let mut second_choice = false;
        let mut lines = Vec::new();
        let mut word_count = 0;
        let mut backgrounds = Vec::new();
        let mut portraits = Vec::new();
        let mut heard_neri = false;
        for _ in 0..160 {
            let delta = player.current();
            for effect in &delta.effects {
                if let MediaEffect::Background(path) = effect {
                    backgrounds.push(path.clone());
                }
            }
            let current = delta.current.clone();
            match current {
                SemanticEvent::Choices(ref choices) if !first_choice => {
                    assert_eq!(choices.len(), 2);
                    first_choice = true;
                    player.choose(first).unwrap();
                }
                SemanticEvent::Choices(ref choices) => {
                    assert_eq!(choices.len(), if first == 0 { 2 } else { 3 });
                    second_choice = true;
                    player.choose(second).unwrap();
                }
                SemanticEvent::Narration(text) => {
                    word_count += text.split_whitespace().count();
                    lines.push(text);
                    player.advance().unwrap();
                }
                SemanticEvent::Dialogue {
                    actor_id,
                    portrait_path,
                    text,
                    ..
                } => {
                    word_count += text.split_whitespace().count();
                    if actor_id == "NER" {
                        heard_neri = true;
                        assert!(portrait_path.is_none());
                    }
                    if let Some(path) = portrait_path {
                        portraits.push(path);
                    }
                    player.advance().unwrap();
                }
                SemanticEvent::End => break,
                _ => {
                    player.advance().unwrap();
                }
            }
        }
        assert!(first_choice && second_choice);
        assert!(matches!(player.current().current, SemanticEvent::End));
        assert!(word_count >= 1500, "branch was only {word_count} words");
        assert!(heard_neri);
        assert!(portraits.iter().any(|path| path == "portraits/dot.svg"));
        assert!(
            backgrounds
                .iter()
                .any(|path| path == "backgrounds/reactor.svg")
        );
        assert!(
            backgrounds
                .iter()
                .any(|path| path == "backgrounds/station.svg")
        );
        if first == 1 {
            assert!(
                backgrounds
                    .iter()
                    .any(|path| path == "backgrounds/archive.svg")
            );
        }
        if first == 0 {
            assert!(
                portraits
                    .iter()
                    .any(|path| path == "portraits/dot_calm.svg")
            );
        }
        if first == 1 && second == 1 {
            assert!(portraits.iter().any(|path| path == "portraits/dot_dim.svg"));
            assert!(lines.iter().any(|line| line.contains("memories are not")));
        }
        if (first == 0 && second == 0) || (first == 1 && second == 1) {
            assert!(
                backgrounds
                    .iter()
                    .any(|path| path == "backgrounds/dawn.svg")
            );
        }
        assert!(
            lines.iter().any(|line| line.contains(expected)),
            "{lines:?}"
        );
    }
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
