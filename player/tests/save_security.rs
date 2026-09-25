use prost::Message;
use storyscript_player::SemanticPlayer;
use storyscript_player::contract::proto::storyplayer::v1 as wire;
use storyscript_player::contract::{HARD_LIMITS, PlayerLimits};

const SOURCE: &str = r#"
* INIT { $count as integer = 1 @start first }
* first { #STORY "hello" @end }
"#;

fn valid_save() -> Vec<u8> {
    SemanticPlayer::from_source(SOURCE, HARD_LIMITS)
        .unwrap()
        .export_save()
        .unwrap()
}

#[test]
fn malformed_oversized_and_wrong_identity_saves_fail_closed() {
    assert_eq!(
        SemanticPlayer::restore_source(SOURCE, &[0xff, 0xff], HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let lowered = PlayerLimits {
        save_bytes: 8,
        ..HARD_LIMITS
    };
    assert_eq!(
        SemanticPlayer::restore_source(SOURCE, &valid_save(), lowered)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let changed = SOURCE.replace("hello", "different");
    assert_eq!(
        SemanticPlayer::restore_source(&changed, &valid_save(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_INCOMPATIBLE"
    );
}

#[test]
fn unknown_variables_targets_and_rng_are_rejected_before_player_exposure() {
    let bytes = valid_save();
    let mut save = wire::PlayerSave::decode(bytes.as_slice()).unwrap();
    save.globals[0].name = "forged".into();
    assert_eq!(
        SemanticPlayer::restore_source(SOURCE, &save.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );

    let mut save = wire::PlayerSave::decode(valid_save().as_slice()).unwrap();
    save.current = Some(wire::SemanticEvent {
        kind: Some(wire::semantic_event::Kind::Choices(wire::Choices {
            items: vec![wire::Choice {
                text: "bad".into(),
                target_scene: "missing".into(),
            }],
        })),
    });
    assert_eq!(
        SemanticPlayer::restore_source(SOURCE, &save.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );

    let mut save = wire::PlayerSave::decode(valid_save().as_slice()).unwrap();
    save.rng.as_mut().unwrap().algorithm_version = 99;
    assert_eq!(
        SemanticPlayer::restore_source(SOURCE, &save.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
}

#[test]
fn failed_restore_cannot_modify_an_existing_session() {
    let mut existing = SemanticPlayer::from_source(SOURCE, HARD_LIMITS).unwrap();
    let before = existing.current().clone();
    let mut corrupt = valid_save();
    corrupt.truncate(corrupt.len() / 2);
    assert!(SemanticPlayer::restore_source(SOURCE, &corrupt, HARD_LIMITS).is_err());
    assert_eq!(existing.current(), &before);
    existing.advance().unwrap();
}
