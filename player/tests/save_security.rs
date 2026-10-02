use prost::Message;
use storyscript_player::SemanticPlayer;
use storyscript_player::contract::proto::storyplayer::v1 as wire;
use storyscript_player::contract::{HARD_LIMITS, PlayerLimits};

mod localization_support;
#[test]
fn hostile_message_ids_names_types_values_and_old_runtime_identity_are_rejected() {
    use localization_support::{prefs, project};
    let root = project();
    let mut player =
        SemanticPlayer::from_project(root.path(), &prefs(&["en"]), HARD_LIMITS).unwrap();
    player.advance().unwrap();
    let save = wire::PlayerSave::decode(player.export_save().unwrap().as_slice()).unwrap();
    for case in ["id", "name", "type", "array", "duplicate", "old"] {
        let mut hostile = save.clone();
        if case == "old" {
            if let Some(wire::origin::Kind::Source(origin)) =
                hostile.origin.as_mut().unwrap().kind.as_mut()
            {
                origin.runtime_identity = "storyscript-player/0.1.0:model-v1".into();
            }
        } else {
            let Some(wire::semantic_event::Kind::Narration(text)) =
                hostile.current.as_mut().unwrap().kind.as_mut()
            else {
                panic!("narration")
            };
            let Some(wire::story_text::Value::Message(message)) = text.value.as_mut() else {
                panic!("message")
            };
            match case {
                "id" => message.id = "unknown-id".into(),
                "name" => message.arguments[0].name = "unknown".into(),
                "type" => {
                    message.arguments[0].r#type = wire::ValueType::Boolean as i32;
                    message.arguments[0].value = Some(wire::Value {
                        kind: Some(wire::value::Kind::Boolean(true)),
                    });
                }
                "array" => message.arguments[0].r#type = wire::ValueType::ArrayString as i32,
                "duplicate" => message.arguments.push(message.arguments[0].clone()),
                _ => unreachable!(),
            }
        }
        let error = SemanticPlayer::restore_project(
            root.path(),
            &hostile.encode_to_vec(),
            &prefs(&["id"]),
            HARD_LIMITS,
        )
        .unwrap_err();
        assert_eq!(
            error.code,
            if case == "old" {
                "R_SAVE_INCOMPATIBLE"
            } else {
                "R_SAVE_STATE_CORRUPT"
            },
            "{case}"
        );
    }
}

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
                text: Some("bad".into()),
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
