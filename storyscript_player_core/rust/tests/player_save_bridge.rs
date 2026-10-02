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

fn project_root() -> String {
    format!(
        "{}/../../bundle/rust/tests/fixtures/localized_project",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn project_locales_render_and_restore_current_pending_and_history() {
    let opened =
        source_player_open_project(project_root(), vec!["id-ID".into(), "en".into()], limits())
            .opened
            .unwrap();
    assert_eq!(opened.resolved_locale.as_deref(), Some("id"));
    assert!(!opened.has_unresolved_localization);
    let greeting = source_player_advance(&opened.resource).delta.unwrap();
    assert!(greeting.event.text.unwrap().starts_with("Halo"));
    let count = source_player_advance(&opened.resource).delta.unwrap();
    assert!(count.event.text.unwrap().contains("barang"));
    let save = source_player_export_save(&opened.resource).bytes.unwrap();
    let restored =
        source_player_restore_project(project_root(), save.clone(), vec!["en".into()], limits())
            .opened
            .unwrap();
    assert_eq!(restored.resolved_locale.as_deref(), Some("en"));
    assert_eq!(restored.current.sequence, count.sequence);
    assert!(restored.current.event.text.unwrap().contains("items"));
    let history = source_player_history(&restored.resource, 0, 256)
        .page
        .unwrap();
    assert!(history.entries.iter().any(|e| e
        .event
        .text
        .as_deref()
        .is_some_and(|s| s.starts_with("Hello"))));
    assert_eq!(
        source_player_advance(&restored.resource)
            .delta
            .unwrap()
            .event
            .choices[0]
            .text,
        "Continue"
    );
    assert_eq!(
        source_player_export_save(&opened.resource).bytes.unwrap(),
        save
    );
    let fallback = source_player_open_project(project_root(), vec!["fr-FR".into()], limits())
        .opened
        .unwrap();
    assert_eq!(fallback.resolved_locale.as_deref(), Some("en"));
    let ordered =
        source_player_open_project(project_root(), vec!["en".into(), "id".into()], limits())
            .opened
            .unwrap();
    assert_eq!(ordered.resolved_locale.as_deref(), Some("en"));
    let malformed = source_player_open_project(project_root(), vec!["id_ID".into()], limits());
    assert!(malformed.opened.is_none());
    assert_eq!(malformed.error.unwrap().code, "R_LOCALIZATION_LOCALE");
}

#[test]
fn raw_keyed_sessions_report_unresolved_ids_and_project_errors_are_structured() {
    let source = "* INIT { @start first; } * first { #STORY @\"greeting\"; @end; }";
    let raw = source_player_open_raw(source.into(), limits())
        .opened
        .unwrap();
    assert_eq!(raw.resolved_locale, None);
    assert!(raw.has_unresolved_localization);
    assert_eq!(
        source_player_advance(&raw.resource)
            .delta
            .unwrap()
            .event
            .text
            .as_deref(),
        Some("greeting")
    );

    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("story")).unwrap();
    std::fs::create_dir(root.path().join("assets")).unwrap();
    std::fs::create_dir(root.path().join("localization")).unwrap();
    std::fs::copy(
        format!("{}/StoryScript.toml", project_root()),
        root.path().join("StoryScript.toml"),
    )
    .unwrap();
    std::fs::write(root.path().join("story/main.StoryScript"),
        "* INIT { $count as integer = 9007199254740993; @start first; } * first { #STORY @\"unsafe\"; @end; }").unwrap();
    for locale in ["en", "id"] {
        std::fs::write(
            root.path().join(format!("localization/{locale}.ftl")),
            "unsafe = { NUMBER($count) }\n",
        )
        .unwrap();
    }
    let result = source_player_open_project(root.path().to_string_lossy().into(), vec![], limits());
    assert!(result.opened.is_none());
    assert_eq!(result.error.unwrap().code, "R_LOCALIZATION_NUMBER");
    // An error in a later scene must not mutate an already-published checkpoint.
    std::fs::write(
        root.path().join("story/main.StoryScript"),
        "* INIT { $count as integer = 9007199254740993; @start first; }
         * first { #STORY \"safe\"; @choice { \"Next\" -> second; } }
         * second { #STORY @\"unsafe\"; @end; }",
    )
    .unwrap();
    let stable = source_player_open_project(root.path().to_string_lossy().into(), vec![], limits())
        .opened
        .unwrap();
    source_player_advance(&stable.resource);
    source_player_advance(&stable.resource);
    let before = source_player_export_save(&stable.resource).bytes.unwrap();
    let failed = source_player_choose(&stable.resource, 0);
    assert!(failed.delta.is_none());
    assert_eq!(failed.error.unwrap().code, "R_LOCALIZATION_NUMBER");
    assert_eq!(
        source_player_export_save(&stable.resource).bytes.unwrap(),
        before
    );

    let raw_path = root.path().join("raw.StoryScript");
    std::fs::write(&raw_path, source).unwrap();
    let path = raw_path.to_string_lossy().to_string();
    let path_player = source_player_open_path(path.clone(), limits())
        .opened
        .unwrap();
    assert!(path_player.has_unresolved_localization);
    assert_eq!(path_player.resolved_locale, None);
    let delta = source_player_advance(&path_player.resource).delta.unwrap();
    assert_eq!(delta.event.text.as_deref(), Some("greeting"));
    let raw_save = source_player_export_save(&path_player.resource)
        .bytes
        .unwrap();
    let restored_path = source_player_restore_path(path, raw_save.clone(), limits())
        .opened
        .unwrap();
    assert_eq!(restored_path.current, delta);
    assert!(restored_path.has_unresolved_localization);
    let restored_raw = source_player_restore_raw(source.into(), raw_save, limits())
        .opened
        .unwrap();
    assert_eq!(restored_raw.resolved_locale, None);
    assert!(restored_raw.has_unresolved_localization);
    let mut invalid = limits();
    invalid.rendered_bytes += 1;
    assert_eq!(
        source_player_open_project(project_root(), vec![], invalid)
            .error
            .unwrap()
            .code,
        "R_LIMIT_CONFIGURATION"
    );
}
