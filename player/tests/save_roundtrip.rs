use storyscript_player::SemanticPlayer;
use storyscript_player::contract::{HARD_LIMITS, SemanticEvent};

mod localization_support;
#[test]
fn cross_locale_restore_rerenders_current_pending_and_history_from_snapshots_without_prep() {
    use localization_support::{prefs, project, replace};
    use prost::Message;
    use storyscript_player::contract::proto::storyplayer::v1 as w;
    let root = project();
    replace(
        root.path(),
        "story/main.StoryScript",
        "$local as string = \"local\";",
        "$local as string = \"local\"; $count += 1;",
    );
    let mut player =
        SemanticPlayer::from_project_seeded(root.path(), &prefs(&["en"]), [17; 32], HARD_LIMITS)
            .unwrap();
    player.advance().unwrap();
    let bytes = player.export_save().unwrap();
    assert!(!bytes.windows(5).any(|v| v == b"Hello"));
    let mut wire = w::PlayerSave::decode(bytes.as_slice()).unwrap();
    // Change live variables, but not the already-flattened argument snapshots.
    wire.globals
        .iter_mut()
        .find(|v| v.name == "name")
        .unwrap()
        .value = Some(w::Value {
        kind: Some(w::value::Kind::Text("Changed".into())),
    });
    let bytes = wire.encode_to_vec();
    let mut restored =
        SemanticPlayer::restore_project(root.path(), &bytes, &prefs(&["id-ID"]), HARD_LIMITS)
            .unwrap();
    assert_eq!(restored.resolved_locale(), Some("id"));
    assert!(
        matches!(&restored.current().current, SemanticEvent::Narration(v) if v.starts_with("Halo") && v.contains("Ada") && !v.contains("Changed"))
    );
    restored.advance().unwrap();
    assert!(
        matches!(&restored.current().current, SemanticEvent::Dialogue { text, .. } if text.contains("barang") && text.contains('3'))
    );
    assert!(
        matches!(&restored.history_page(0, 10).entries[1].event, SemanticEvent::Narration(v) if v.starts_with("Halo"))
    );
    restored.advance().unwrap();
    assert!(
        matches!(&restored.current().current, SemanticEvent::Choices(v) if v[0].text == "Lanjutkan")
    );
    let save = restored.export_save().unwrap();
    let mut english =
        SemanticPlayer::restore_project(root.path(), &save, &prefs(&["en"]), HARD_LIMITS).unwrap();
    assert!(
        matches!(&english.current().current, SemanticEvent::Choices(v) if v[0].text == "Continue")
    );
    assert!(
        english
            .history_page(0, 10)
            .entries
            .iter()
            .any(|e| matches!(&e.event, SemanticEvent::Narration(v) if v.starts_with("Hello")))
    );
    english.choose(0).unwrap();
    english.advance().unwrap();
    assert!(matches!(&english.current().current, SemanticEvent::Narration(v) if v == "Plain 4"));
}

#[test]
fn raw_keyed_saves_preserve_references_and_rerender_ids_without_catalogs() {
    let source = r#"* INIT { @start s } * s { #STORY @"raw-id"; @end }"#;
    let mut player = SemanticPlayer::from_source(source, HARD_LIMITS).unwrap();
    player.advance().unwrap();
    let save = player.export_save().unwrap();
    let restored = SemanticPlayer::restore_source(source, &save, HARD_LIMITS).unwrap();
    assert_eq!(restored.current(), player.current());
    assert!(restored.has_unresolved_localization());
    assert_eq!(restored.resolved_locale(), None);
}

#[test]
fn repeated_site_snapshots_preserve_distinct_loop_values_after_iterator_disappears() {
    use localization_support::{prefs, project};
    let root = project();
    std::fs::write(
        root.path().join("story/main.StoryScript"),
        r#"* INIT { $names as array<string> = ["Ada", "Lin"]; @start s }
* s { #STORY for ($item in snapshot $names) { @"name-line"; } @end }"#,
    )
    .unwrap();
    std::fs::write(
        root.path().join("localization/en.ftl"),
        "name-line = Name { $item }\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("localization/id.ftl"),
        "name-line = Nama { $item }\n",
    )
    .unwrap();
    let player = SemanticPlayer::from_project(root.path(), &prefs(&["en"]), HARD_LIMITS).unwrap();
    let save = player.export_save().unwrap();
    let mut restored =
        SemanticPlayer::restore_project(root.path(), &save, &prefs(&["id"]), HARD_LIMITS).unwrap();
    restored.advance().unwrap();
    assert!(
        matches!(&restored.current().current, SemanticEvent::Narration(v) if v.contains("Nama") && v.contains("Ada"))
    );
    restored.advance().unwrap();
    assert!(
        matches!(&restored.current().current, SemanticEvent::Narration(v) if v.contains("Nama") && v.contains("Lin"))
    );
}

const SOURCE: &str = r#"
* INIT {
    $visits as integer = 0
    $roll as integer = 0
    @start first
}
* first {
    #PREP
    $visits += 1
    $roll = rand(1, 1000000)
    #STORY
    "visits=${visits}"
    "roll=${roll}"
    @choice { "Next" -> second }
}
* second {
    #STORY
    "done"
    @end
}
"#;

#[test]
fn source_save_roundtrip_resumes_without_replaying_prep() {
    let mut original = SemanticPlayer::from_source(SOURCE, HARD_LIMITS).unwrap();
    assert!(matches!(
        original.current().current,
        SemanticEvent::SceneTransition(_)
    ));
    let save = original.export_save().unwrap();
    let mut restored = SemanticPlayer::restore_source(SOURCE, &save, HARD_LIMITS).unwrap();
    assert_eq!(original.current(), restored.current());
    let next_original = original.advance().unwrap().clone();
    let next_restored = restored.advance().unwrap().clone();
    assert_eq!(next_original, next_restored);
    assert!(
        matches!(next_restored.current, SemanticEvent::Narration(ref text) if text == "visits=1")
    );
    assert_eq!(original.advance().unwrap(), restored.advance().unwrap());
}

#[test]
fn choices_history_and_finished_state_survive_repeated_roundtrips() {
    let mut player = SemanticPlayer::from_source(SOURCE, HARD_LIMITS).unwrap();
    player.advance().unwrap();
    player.advance().unwrap();
    player.advance().unwrap();
    assert!(matches!(
        player.current().current,
        SemanticEvent::Choices(_)
    ));
    for _ in 0..3 {
        let bytes = player.export_save().unwrap();
        player = SemanticPlayer::restore_source(SOURCE, &bytes, HARD_LIMITS).unwrap();
    }
    assert_eq!(player.history_page(0, 20).entries.len(), 3);
    player.choose(0).unwrap();
    player.advance().unwrap();
    player.advance().unwrap();
    assert!(matches!(player.current().current, SemanticEvent::End));
    let save = player.export_save().unwrap();
    let restored = SemanticPlayer::restore_source(SOURCE, &save, HARD_LIMITS).unwrap();
    assert_eq!(restored.current(), player.current());
}

#[test]
fn formatting_and_comments_do_not_change_semantic_identity() {
    let player = SemanticPlayer::from_source(SOURCE, HARD_LIMITS).unwrap();
    let save = player.export_save().unwrap();
    let reformatted = format!("// harmless comment\n{SOURCE}\n// trailing comment\n");
    assert!(SemanticPlayer::restore_source(&reformatted, &save, HARD_LIMITS).is_ok());
    let changed = SOURCE.replace("\"done\"", "\"changed\"");
    assert_eq!(
        SemanticPlayer::restore_source(&changed, &save, HARD_LIMITS)
            .unwrap_err()
            .code,
        "R_SAVE_INCOMPATIBLE"
    );
}
