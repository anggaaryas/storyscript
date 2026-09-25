use storyscript_parser::ast::{
    InitBlock, Scene, Script, StartDirective, StoryBlock, StoryStatement,
};
use storyscript_player::contract::HARD_LIMITS;
use storyscript_player::contract::SemanticEvent;
use storyscript_player::engine::Engine;
use storyscript_player::{SemanticPlayer, StepResult, StoryPlayer};

const SOURCE: &str = r#"
* INIT { @start first }
* first {
    #STORY
    "Hello"
    @choice { "Next" -> second; }
}
* second {
    #STORY
    "Goodbye"
    @end
}
"#;

#[test]
fn scene_headers_choice_markers_history_and_end_are_still_legacy_events() {
    let mut player = StoryPlayer::from_source("compatibility", SOURCE).unwrap();
    assert!(
        matches!(player.current(), Some(StepResult::Narration(s)) if s == "─── Scene: first ───")
    );
    player.advance();
    assert!(matches!(player.current(), Some(StepResult::Narration(s)) if s == "Hello"));
    player.advance();
    assert!(
        matches!(player.current(), Some(StepResult::Choices(options)) if options.len() == 1 && options[0].text == "Next")
    );
    assert!(!player.select_choice(1));
    assert_eq!(player.history().len(), 2);
    assert!(player.select_choice(0));
    assert!(
        matches!(player.current(), Some(StepResult::Narration(s)) if s == "─── Scene: second ───")
    );
    assert!(matches!(player.history().last(), Some(StepResult::Narration(s)) if s == "▸ Next"));
    player.advance();
    assert!(matches!(player.current(), Some(StepResult::Narration(s)) if s == "Goodbye"));
    player.advance();
    assert!(matches!(player.current(), Some(StepResult::End)));
}

#[test]
fn structured_errors_are_formatted_only_for_legacy_callers() {
    let script = Script {
        init: InitBlock {
            variables: vec![],
            actors: vec![],
            includes: vec![],
            start: StartDirective {
                target: "broken".into(),
                line: 0,
                column: 0,
            },
            line: 0,
            column: 0,
        },
        logic_blocks: vec![],
        scenes: vec![Scene {
            label: "broken".into(),
            prep: None,
            story: StoryBlock {
                statements: vec![StoryStatement::VarOutput {
                    name: "unknown".into(),
                    line: 0,
                    column: 0,
                }],
                line: 0,
                column: 0,
            },
            line: 0,
            column: 0,
        }],
    };
    let mut semantic = Engine::new_seeded(&script, [0; 32]);
    assert!(
        matches!(semantic.step_semantic(), Some((SemanticEvent::Error(e), _)) if e.code == "RUNTIME" && e.scene == "broken")
    );
    let mut legacy = StoryPlayer::new("broken", &script);
    assert!(
        matches!(legacy.current(), Some(StepResult::Narration(text)) if text.starts_with("[RUNTIME]"))
    );
    legacy.advance();
    assert!(matches!(legacy.current(), Some(StepResult::End)));
}

#[test]
fn semantic_source_constructor_compiles_and_keeps_choices_structured() {
    let mut player = SemanticPlayer::from_source(SOURCE, HARD_LIMITS).unwrap();
    assert!(
        matches!(player.current().current, SemanticEvent::SceneTransition(ref scene) if scene == "first")
    );
    player.advance().unwrap();
    assert!(
        matches!(player.current().current, SemanticEvent::Narration(ref text) if text == "Hello")
    );
    player.advance().unwrap();
    assert!(
        matches!(player.current().current, SemanticEvent::Choices(ref choices) if choices.len() == 1)
    );
    let prior = player.current().clone();
    assert_eq!(player.choose(1).unwrap_err().code, "R_INVALID_CHOICE");
    assert_eq!(player.current(), &prior);
    player.choose(0).unwrap();
    assert!(
        matches!(player.current().current, SemanticEvent::SceneTransition(ref scene) if scene == "second")
    );
    assert_eq!(player.history_page(0, 10).entries.len(), 3);
}
