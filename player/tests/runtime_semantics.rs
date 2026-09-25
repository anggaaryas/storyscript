use storyscript_parser::ast::{
    ActorDecl, BgmValue, Dialogue, DialogueForm, Expr, InitBlock, PortraitEntry, Position,
    PrepBlock, PrepStatement, Scene, Script, StartDirective, StoryBlock, StoryStatement, VarDecl,
    VarType,
};
use storyscript_player::contract::{HARD_LIMITS, MediaEffect, PlayerLimits, SemanticEvent};
use storyscript_player::engine::{Engine, StepResult, Value};
use storyscript_player::runtime::{SemanticPlayer, StoryPlayer};
use storyscript_player::session_rng::{ALGORITHM_VERSION, SessionRng};

fn random_story() -> Script {
    Script {
        init: InitBlock {
            variables: vec![VarDecl {
                name: "initial".into(),
                var_type: VarType::Integer,
                value: Expr::Call {
                    name: "rand".into(),
                    args: vec![],
                    line: 0,
                    column: 0,
                },
                line: 0,
                column: 0,
            }],
            actors: vec![],
            includes: vec![],
            start: StartDirective {
                target: "first".into(),
                line: 0,
                column: 0,
            },
            line: 0,
            column: 0,
        },
        logic_blocks: vec![],
        scenes: ["first", "second"]
            .into_iter()
            .map(|label| Scene {
                label: label.into(),
                prep: Some(PrepBlock {
                    statements: vec![PrepStatement::VarDecl(VarDecl {
                        name: "local".into(),
                        var_type: VarType::Integer,
                        value: Expr::Call {
                            name: "rand".into(),
                            args: vec![],
                            line: 0,
                            column: 0,
                        },
                        line: 0,
                        column: 0,
                    })],
                    line: 0,
                    column: 0,
                }),
                story: StoryBlock {
                    statements: vec![
                        StoryStatement::VarOutput {
                            name: "local".into(),
                            line: 0,
                            column: 0,
                        },
                        if label == "first" {
                            StoryStatement::Jump {
                                target: "second".into(),
                                line: 0,
                                column: 0,
                            }
                        } else {
                            StoryStatement::End { line: 0, column: 0 }
                        },
                    ],
                    line: 0,
                    column: 0,
                },
                line: 0,
                column: 0,
            })
            .collect(),
    }
}

fn output(player: &StoryPlayer) -> String {
    match player.current().unwrap() {
        StepResult::Narration(text) => text.clone(),
        StepResult::End => "end".into(),
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn seed_covers_init_prep_and_subsequent_scene_entries() {
    let story = random_story();
    let mut a = StoryPlayer::new_seeded("a", &story, [17; 32]);
    let mut b = StoryPlayer::new_seeded("b", &story, [17; 32]);
    assert_eq!(
        a.engine().variables.get("initial"),
        b.engine().variables.get("initial")
    );
    assert!(matches!(
        a.engine().variables.get("initial"),
        Some(Value::Int(_))
    ));
    for _ in 0..5 {
        assert_eq!(output(&a), output(&b));
        assert_eq!(a.engine().rng_state(), b.engine().rng_state());
        a.advance();
        b.advance();
    }
}

#[test]
fn rng_state_reconstructs_stream_and_rejects_unknown_version() {
    let mut rng = SessionRng::from_seed([42; 32]);
    let _: i64 = rng.sample();
    let _: usize = rng.range(0..9);
    let state = rng.state();
    assert_eq!(state.algorithm_version, ALGORITHM_VERSION);
    assert_ne!(state.word_position, 0);
    let mut restored = SessionRng::from_state(state).unwrap();
    for _ in 0..20 {
        assert_eq!(
            rng.range(i64::MIN..=i64::MAX),
            restored.range(i64::MIN..=i64::MAX)
        );
        assert_eq!(rng.state(), restored.state());
    }
    let mut unsupported = state;
    unsupported.algorithm_version += 1;
    assert!(SessionRng::from_state(unsupported).is_err());
    let mut noncanonical = state;
    noncanonical.word_position = 1u128 << 68;
    assert!(SessionRng::from_state(noncanonical).is_err());
}

#[test]
fn semantic_stream_resolves_prep_effects_story_sfx_and_portrait_without_tui_headers() {
    let mut story = random_story();
    story.init.actors.push(ActorDecl {
        id: "A".into(),
        display_name: "Alice".into(),
        portraits: vec![PortraitEntry {
            emotion: "calm".into(),
            path: "portraits/calm.png".into(),
            line: 0,
            column: 0,
        }],
        line: 0,
        column: 0,
    });
    story.scenes[0].prep.as_mut().unwrap().statements.splice(
        0..0,
        [
            PrepStatement::BgDirective {
                path: "background.png".into(),
                line: 0,
                column: 0,
            },
            PrepStatement::BgmDirective {
                value: BgmValue::Path("music.ogg".into()),
                line: 0,
                column: 0,
            },
            PrepStatement::SfxDirective {
                path: "entry.wav".into(),
                line: 0,
                column: 0,
            },
        ],
    );
    story.scenes[0].story.statements.insert(
        0,
        StoryStatement::Dialogue(Dialogue {
            actor_id: "A".into(),
            form: DialogueForm::Portrait {
                emotion: "calm".into(),
                position: Position::Left,
            },
            text: "Hello".into(),
            line: 0,
            column: 0,
        }),
    );
    story.scenes[0].story.statements.insert(
        2,
        StoryStatement::SfxDirective {
            path: "story.wav".into(),
            line: 0,
            column: 0,
        },
    );
    let mut engine = Engine::new_seeded(&story, [3; 32]);
    assert_eq!(
        engine.step_semantic(),
        Some((
            SemanticEvent::SceneTransition("first".into()),
            vec![
                MediaEffect::Background("background.png".into()),
                MediaEffect::Bgm("music.ogg".into()),
                MediaEffect::Sfx("entry.wav".into())
            ],
        ))
    );
    assert!(
        matches!(engine.step_semantic(), Some((SemanticEvent::Dialogue { portrait_path: Some(path), position: Some(position), .. }, effects)) if path == "portraits/calm.png" && position == "Left" && effects.is_empty())
    );
    assert!(matches!(
        engine.step_semantic(),
        Some((SemanticEvent::Narration(_), _))
    ));
    assert_eq!(
        engine.step_semantic(),
        Some((
            SemanticEvent::Media(MediaEffect::Sfx("story.wav".into())),
            vec![]
        ))
    );
    assert_eq!(
        engine.step_semantic(),
        Some((SemanticEvent::SceneTransition("second".into()), vec![]))
    );

    let mut legacy = Engine::new_seeded(&story, [3; 32]);
    assert!(
        matches!(legacy.step(), Some(StepResult::Narration(text)) if text == "─── Scene: first ───")
    );
    assert!(matches!(legacy.step(), Some(StepResult::Dialogue { text, .. }) if text == "Hello"));
    assert!(matches!(legacy.step(), Some(StepResult::Narration(_))));
    assert!(
        matches!(legacy.step(), Some(StepResult::Narration(text)) if text == "─── Scene: second ───")
    );
}

#[test]
fn stopping_bgm_is_a_distinct_effect_not_a_missing_effect() {
    let mut story = random_story();
    story.scenes[0].prep.as_mut().unwrap().statements.insert(
        0,
        PrepStatement::BgmDirective {
            value: BgmValue::Stop,
            line: 0,
            column: 0,
        },
    );
    let mut engine = Engine::new_seeded(&story, [3; 32]);
    assert_eq!(
        engine.step_semantic(),
        Some((
            SemanticEvent::SceneTransition("first".into()),
            vec![MediaEffect::BgmStop],
        ))
    );
}

#[test]
fn compact_session_pages_bounded_history_and_never_includes_transcript_in_delta() {
    let story = random_story();
    let limits = PlayerLimits {
        history_entries: 2,
        ..HARD_LIMITS
    };
    let mut player = SemanticPlayer::new_seeded(&story, [5; 32], limits).unwrap();
    assert_eq!(player.current().sequence, 0);
    assert!(matches!(
        player.current().current,
        SemanticEvent::SceneTransition(_)
    ));
    for sequence in 1..=4 {
        assert_eq!(player.advance().unwrap().sequence, sequence);
    }
    assert_eq!(player.current().first_retained_sequence, 2);
    assert_eq!(player.current().omitted_history_count, 2);
    let page = player.history_page(0, 1);
    assert_eq!(page.entries[0].sequence, 2);
    assert_eq!(
        player.history_page(page.next_sequence, 1).entries[0].sequence,
        3
    );
    assert_eq!(
        player.current().status,
        storyscript_player::contract::SessionStatus::Finished
    );
    let before = player.current().clone();
    assert_eq!(player.advance().unwrap_err().code, "R_INVALID_ACTION");
    assert_eq!(player.current(), &before);
}
