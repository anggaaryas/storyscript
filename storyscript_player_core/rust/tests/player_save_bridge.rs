use storyscript_player_core::api::player_v2::*;

const SOURCE: &str = r#"
* INIT { @start first }
* first { #STORY "hello" @choice { "Next" -> second } }
* second { #STORY "done" @end }
"#;

fn limits() -> BridgePlayerLimits {
    source_player_hard_limits()
}

#[test]
fn source_resource_progresses_pages_saves_and_restores_compactly() {
    let opened = source_player_open_raw(SOURCE.into(), limits());
    assert!(opened.error.is_none());
    let opened = opened.opened.unwrap();
    assert_eq!(opened.current.event.kind, "scene");
    let resource = opened.resource;
    let narration = source_player_advance(&resource).delta.unwrap();
    assert_eq!(narration.event.text.as_deref(), Some("hello"));
    let choices = source_player_advance(&resource).delta.unwrap();
    assert_eq!(choices.event.choices.len(), 1);
    assert!(source_player_choose(&resource, 9).error.is_some());
    let save = source_player_export_save(&resource).bytes.unwrap();
    let page = source_player_history(&resource, 0, 1).page.unwrap();
    assert_eq!(page.entries.len(), 1);

    let restored = source_player_restore_raw(SOURCE.into(), save, limits())
        .opened
        .unwrap();
    assert_eq!(restored.current, choices);
    assert_eq!(
        source_player_choose(&restored.resource, 0)
            .delta
            .unwrap()
            .event
            .kind,
        "scene"
    );
}

#[test]
fn path_open_identity_limits_corruption_and_disposal_are_structured() {
    let path = format!(
        "{}/../../example/feature_else_if_chain.StoryScript",
        env!("CARGO_MANIFEST_DIR")
    );
    assert!(source_player_open_path(path, limits()).opened.is_some());

    let mut invalid_limits = limits();
    invalid_limits.history_entries = 0;
    assert_eq!(
        source_player_open_raw(SOURCE.into(), invalid_limits)
            .error
            .unwrap()
            .code,
        "R_LIMIT_CONFIGURATION"
    );
    assert_eq!(
        source_player_restore_raw(SOURCE.into(), vec![0xff], limits())
            .error
            .unwrap()
            .code,
        "R_SAVE_STATE_CORRUPT"
    );

    let resource = source_player_open_raw(SOURCE.into(), limits())
        .opened
        .unwrap()
        .resource;
    assert!(source_player_dispose(&resource).released);
    assert!(!source_player_dispose(&resource).released);
    assert_eq!(
        source_player_current(&resource).error.unwrap().code,
        "R_PLAYER_DISPOSED"
    );
}

#[test]
fn legacy_numeric_session_api_remains_available() {
    let id = storyscript_player_core::api::player::player_open_raw(SOURCE.into()).unwrap();
    let state = storyscript_player_core::api::player::player_get_state(id).unwrap();
    assert_eq!(state.session_id, id);
    assert!(storyscript_player_core::api::player::player_close(id));
}
