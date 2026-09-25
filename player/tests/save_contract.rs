use prost::Message;
use prost_types::FileDescriptorSet;
use storyscript_player::contract::{
    BundleSigner, HARD_LIMITS, Origin, PlayerLimits, SAVE_DESCRIPTOR, SAVE_RUNTIME_VERSION,
    SAVE_SCHEMA_SHA256, SAVE_SCHEMA_VERSION, SemanticEvent, decode_save_contract,
    descriptor_sha256, proto::storyplayer::v1 as wire,
};

#[test]
fn descriptor_is_pinned_and_versions_are_explicit() {
    assert!(!SAVE_DESCRIPTOR.is_empty());
    assert_eq!(descriptor_sha256(), SAVE_SCHEMA_SHA256.trim());
    assert_eq!(SAVE_SCHEMA_VERSION, 1);
    assert_eq!(SAVE_RUNTIME_VERSION, 1);
}

#[test]
fn descriptor_requires_all_progress_fields_without_static_story() {
    let set = FileDescriptorSet::decode(SAVE_DESCRIPTOR).unwrap();
    let file = &set.file[0];
    assert_eq!(file.package.as_deref(), Some("storyplayer.v1"));
    let messages = &file.message_type;
    let fields = |name: &str| {
        messages
            .iter()
            .find(|m| m.name.as_deref() == Some(name))
            .unwrap()
            .field
            .iter()
            .map(|f| f.name.as_deref().unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        fields("PlayerSave"),
        [
            "schema_version",
            "runtime_version",
            "origin",
            "globals",
            "locals",
            "current_scene",
            "background",
            "bgm",
            "current",
            "pending",
            "rng",
            "history",
            "next_sequence",
            "first_retained_sequence",
            "omitted_history_count",
            "status",
            "current_effects",
        ]
    );
    assert_eq!(
        fields("BundleOrigin"),
        [
            "format_version",
            "compiler_version",
            "compiled_schema_sha256",
            "project_id",
            "project_version",
            "compiled_entry_sha256",
            "signer_key_id",
            "unsigned_development",
            "runtime_fingerprint_sha256",
            "runtime_identity",
        ]
    );
    assert_eq!(
        fields("SemanticEvent"),
        [
            "scene",
            "narration",
            "dialogue",
            "choices",
            "media",
            "end",
            "error",
            "jump",
        ]
    );
    assert_eq!(
        fields("MediaEffect"),
        ["background_path", "bgm_path", "bgm_stop", "sfx_path"]
    );
    assert_eq!(
        fields("SessionRng"),
        [
            "algorithm_version",
            "seed",
            "stream",
            "word_position_low",
            "word_position_high"
        ]
    );
    assert!(!messages.iter().any(|m| matches!(
        m.name.as_deref(),
        Some("Script" | "CompiledStory" | "StoryBlock")
    )));
}

#[test]
fn ordered_records_and_exact_scalars_roundtrip() {
    let mut save = wire::PlayerSave {
        schema_version: SAVE_SCHEMA_VERSION,
        runtime_version: SAVE_RUNTIME_VERSION,
        origin: Some(wire::Origin {
            kind: Some(wire::origin::Kind::Source(wire::SourceOrigin {
                parser_version: "0.1".into(),
                compiler_version: "0.1".into(),
                runtime_fingerprint_sha256: "a".repeat(64),
                runtime_identity: "storyscript-player/1".into(),
            })),
        }),
        current_scene: "intro".into(),
        rng: Some(wire::SessionRng {
            algorithm_version: 1,
            seed: vec![7; 32],
            stream: 3,
            word_position_low: 27,
            word_position_high: 0,
        }),
        status: wire::SessionStatus::Active as i32,
        next_sequence: 3,
        first_retained_sequence: 1,
        ..Default::default()
    };
    save.globals = vec![
        wire::Variable {
            name: "decimal".into(),
            declared_type: wire::ValueType::Decimal as i32,
            value: Some(wire::Value {
                kind: Some(wire::value::Kind::Decimal(wire::DecimalValue {
                    mantissa: "-1234567890123456789012345678".into(),
                    scale: 18,
                })),
            }),
        },
        wire::Variable {
            name: "integer".into(),
            declared_type: wire::ValueType::Integer as i32,
            value: Some(wire::Value {
                kind: Some(wire::value::Kind::Integer(i64::MAX)),
            }),
        },
        wire::Variable {
            name: "array".into(),
            declared_type: wire::ValueType::ArrayBoolean as i32,
            value: Some(wire::Value {
                kind: Some(wire::value::Kind::Array(wire::ScalarArray {
                    element_type: wire::ValueType::Boolean as i32,
                    items: vec![wire::Value {
                        kind: Some(wire::value::Kind::Boolean(false)),
                    }],
                })),
            }),
        },
    ];
    save.locals.push(wire::Variable {
        name: "text".into(),
        declared_type: wire::ValueType::String as i32,
        value: Some(wire::Value {
            kind: Some(wire::value::Kind::Text("hello".into())),
        }),
    });
    let decoded = wire::PlayerSave::decode(save.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded, save);
    assert_eq!(
        decoded
            .globals
            .iter()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>(),
        ["decimal", "integer", "array"]
    );
    assert_eq!(decoded.globals[0].value, save.globals[0].value);
    assert!(matches!(
        decoded.globals[1].value.as_ref().unwrap().kind,
        Some(wire::value::Kind::Integer(i64::MAX))
    ));
}

#[test]
fn every_event_effect_and_origin_has_a_distinct_variant() {
    let events = [
        wire::semantic_event::Kind::Scene(wire::SceneTransition { scene: "a".into() }),
        wire::semantic_event::Kind::Narration("text".into()),
        wire::semantic_event::Kind::Dialogue(wire::Dialogue {
            actor_id: "a".into(),
            portrait_path: Some("portrait.png".into()),
            ..Default::default()
        }),
        wire::semantic_event::Kind::Choices(wire::Choices {
            items: vec![wire::Choice {
                text: "yes".into(),
                target_scene: "b".into(),
            }],
        }),
        wire::semantic_event::Kind::Media(wire::MediaEffect {
            kind: Some(wire::media_effect::Kind::SfxPath("tap.wav".into())),
        }),
        wire::semantic_event::Kind::End(true),
        wire::semantic_event::Kind::Error(wire::RuntimeError {
            code: "R_LIMIT".into(),
            scene: "a".into(),
            ..Default::default()
        }),
        wire::semantic_event::Kind::Jump("b".into()),
    ];
    for event in events {
        let item = wire::SemanticEvent { kind: Some(event) };
        assert_eq!(
            wire::SemanticEvent::decode(item.encode_to_vec().as_slice()).unwrap(),
            item
        );
    }
    for effect in [
        wire::media_effect::Kind::BackgroundPath("bg".into()),
        wire::media_effect::Kind::BgmPath("music".into()),
        wire::media_effect::Kind::BgmStop(true),
        wire::media_effect::Kind::SfxPath("sound".into()),
    ] {
        let item = wire::MediaEffect { kind: Some(effect) };
        assert_eq!(
            wire::MediaEffect::decode(item.encode_to_vec().as_slice()).unwrap(),
            item
        );
    }
    let source = Origin::Source {
        parser_version: "p".into(),
        compiler_version: "c".into(),
        runtime_identity: "r".into(),
        semantic_sha256: "s".into(),
    };
    let bundle = Origin::Bundle {
        format_version: 1,
        compiler_version: "c".into(),
        schema_sha256: "h".into(),
        project_id: "p".into(),
        project_version: "1".into(),
        compiled_entry_sha256: "d".into(),
        signer: BundleSigner::UnsignedDevelopment,
        runtime_identity: "r".into(),
        semantic_sha256: "s".into(),
    };
    assert_ne!(source, bundle);
    assert_ne!(SemanticEvent::End, SemanticEvent::Narration(String::new()));
    let signed = wire::BundleOrigin {
        signer: Some(wire::bundle_origin::Signer::SignerKeyId("key".into())),
        ..Default::default()
    };
    let unsigned = wire::BundleOrigin {
        signer: Some(wire::bundle_origin::Signer::UnsignedDevelopment(true)),
        ..Default::default()
    };
    assert_ne!(signed.encode_to_vec(), unsigned.encode_to_vec());
}

#[test]
fn all_scalar_and_array_value_types_survive_wire_roundtrip() {
    use wire::{ValueType as T, value::Kind as K};
    let scalar = [
        (T::Integer, K::Integer(i64::MIN)),
        (
            T::Decimal,
            K::Decimal(wire::DecimalValue {
                mantissa: "100".into(),
                scale: 2,
            }),
        ),
        (T::Boolean, K::Boolean(false)),
        (T::String, K::Text("\u{1f642}".into())),
    ];
    for (ty, kind) in scalar {
        let mut save = valid_wire_save();
        save.globals.push(wire::Variable {
            name: "value".into(),
            declared_type: ty as i32,
            value: Some(wire::Value { kind: Some(kind) }),
        });
        let bytes = save.encode_to_vec();
        assert_eq!(decode_save_contract(&bytes, HARD_LIMITS).unwrap(), save);
    }
    for (array_ty, element_ty, element) in [
        (T::ArrayInteger, T::Integer, K::Integer(i64::MAX)),
        (
            T::ArrayDecimal,
            T::Decimal,
            K::Decimal(wire::DecimalValue {
                mantissa: "-100".into(),
                scale: 2,
            }),
        ),
        (T::ArrayBoolean, T::Boolean, K::Boolean(true)),
        (T::ArrayString, T::String, K::Text("text".into())),
    ] {
        let mut save = valid_wire_save();
        save.globals.push(wire::Variable {
            name: "array".into(),
            declared_type: array_ty as i32,
            value: Some(wire::Value {
                kind: Some(K::Array(wire::ScalarArray {
                    element_type: element_ty as i32,
                    items: vec![wire::Value {
                        kind: Some(element),
                    }],
                })),
            }),
        });
        let bytes = save.encode_to_vec();
        assert_eq!(decode_save_contract(&bytes, HARD_LIMITS).unwrap(), save);
    }
}

#[test]
fn limits_are_lower_only_and_no_source_story_is_embedded() {
    assert_eq!(HARD_LIMITS.operations_per_interaction, 100_000);
    assert_eq!(HARD_LIMITS.logic_depth, 128);
    assert_eq!(HARD_LIMITS.pending_events_per_scene, 16_384);
    assert_eq!(HARD_LIMITS.array_elements, 16_384);
    assert_eq!(HARD_LIMITS.rendered_bytes, 1 << 20);
    assert_eq!(HARD_LIMITS.history_entries, 10_000);
    assert_eq!(HARD_LIMITS.history_bytes, 8 << 20);
    assert_eq!(HARD_LIMITS.save_bytes, 16 << 20);
    assert!(HARD_LIMITS.lowered().is_ok());
    let limits = PlayerLimits {
        history_entries: 4,
        ..HARD_LIMITS
    };
    assert_eq!(limits.lowered().unwrap().history_entries, 4);
    assert_eq!(
        PlayerLimits {
            save_bytes: HARD_LIMITS.save_bytes + 1,
            ..HARD_LIMITS
        }
        .lowered()
        .unwrap_err()
        .resource,
        "save_bytes"
    );
    assert_eq!(
        PlayerLimits {
            rendered_bytes: 0,
            ..HARD_LIMITS
        }
        .lowered()
        .unwrap_err()
        .resource,
        "rendered_bytes"
    );
    let schema = include_str!("../proto/storyplayer/v1/player_save.proto");
    assert!(!schema.contains("CompiledStory"));
    assert!(!schema.contains("source_bytes"));
    assert!(!schema.contains("archive_bytes"));
    assert!(!schema.contains("map<"));
}

fn valid_wire_save() -> wire::PlayerSave {
    wire::PlayerSave {
        schema_version: 1,
        runtime_version: 1,
        origin: Some(wire::Origin {
            kind: Some(wire::origin::Kind::Source(wire::SourceOrigin {
                parser_version: "0.1.0".into(),
                compiler_version: "0.1.0".into(),
                runtime_fingerprint_sha256: "a".repeat(64),
                runtime_identity: "storyscript-player/1".into(),
            })),
        }),
        current_scene: "intro".into(),
        current: Some(wire::SemanticEvent {
            kind: Some(wire::semantic_event::Kind::Narration("hello".into())),
        }),
        rng: Some(wire::SessionRng {
            algorithm_version: 1,
            seed: vec![0; 32],
            ..Default::default()
        }),
        status: wire::SessionStatus::Active as i32,
        next_sequence: 1,
        ..Default::default()
    }
}

#[test]
fn contract_decoder_rejects_unknown_required_and_duplicate_records() {
    let good = valid_wire_save();
    let bytes = good.encode_to_vec();
    assert_eq!(decode_save_contract(&bytes, HARD_LIMITS).unwrap(), good);
    let mut oversized = HARD_LIMITS;
    oversized.save_bytes = bytes.len() - 1;
    assert_eq!(
        decode_save_contract(&bytes, oversized).unwrap_err().code,
        "R_SAVE_STATE_CORRUPT"
    );
    assert_eq!(
        decode_save_contract(&bytes[..bytes.len() - 1], HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let mut version = good.clone();
    version.schema_version = 2;
    assert_eq!(
        decode_save_contract(&version.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_INCOMPATIBLE"
    );
    let mut unknown_field = bytes.clone();
    unknown_field.extend_from_slice(&[0x98, 0x06, 0x01]); // field 99
    assert_eq!(
        decode_save_contract(&unknown_field, HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let mut unknown_enum = good.clone();
    unknown_enum.status = 99;
    assert_eq!(
        decode_save_contract(&unknown_enum.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let mut unknown_oneof = good.clone();
    unknown_oneof.current = Some(wire::SemanticEvent::default());
    assert_eq!(
        decode_save_contract(&unknown_oneof.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let mut duplicate = good.clone();
    let var = wire::Variable {
        name: "counter".into(),
        declared_type: wire::ValueType::Integer as i32,
        value: Some(wire::Value {
            kind: Some(wire::value::Kind::Integer(1)),
        }),
    };
    duplicate.globals = vec![var.clone(), var];
    assert_eq!(
        decode_save_contract(&duplicate.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let mut history = good.clone();
    let entry = wire::HistoryEntry {
        sequence: 0,
        event: good.current.clone(),
        scene: "intro".into(),
        ..Default::default()
    };
    history.history = vec![entry.clone(), entry];
    assert_eq!(
        decode_save_contract(&history.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let mut bad_value = good.clone();
    bad_value.globals.push(wire::Variable {
        name: "decimal".into(),
        declared_type: wire::ValueType::Decimal as i32,
        value: Some(wire::Value {
            kind: Some(wire::value::Kind::Decimal(wire::DecimalValue {
                mantissa: "01".into(),
                scale: 0,
            })),
        }),
    });
    assert_eq!(
        decode_save_contract(&bad_value.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
    let mut invalid_rng = good.clone();
    invalid_rng.rng.as_mut().unwrap().word_position_high = 16;
    assert_eq!(
        decode_save_contract(&invalid_rng.encode_to_vec(), HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );
}
