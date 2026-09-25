use storyscript_player::SemanticPlayer;
use storyscript_player::contract::{HARD_LIMITS, SemanticEvent};

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
