use storyscript_parser::ast::{
    AssignOp, Expr, InitBlock, LogicBlock, PrepBlock, PrepRepeat, PrepStatement, RepeatCount,
    Scene, Script, StartDirective, StoryBlock, StoryStatement, VarAssign, VarDecl, VarType,
};
use storyscript_player::contract::{HARD_LIMITS, MediaEffect, PlayerLimits, SemanticEvent};
use storyscript_player::engine::{ChoiceDisplay, Engine, Value};
use storyscript_player::history::HistoryBuffer;

#[test]
fn history_retains_a_bounded_suffix_with_absolute_sequences_and_pages() {
    let limits = PlayerLimits {
        history_entries: 2,
        history_bytes: 200,
        ..HARD_LIMITS
    };
    let mut history = HistoryBuffer::new(limits).unwrap();
    for n in 0..6 {
        history
            .append(
                SemanticEvent::Narration(format!("event {n}")),
                vec![],
                "scene".into(),
            )
            .unwrap();
    }
    assert_eq!(history.next_sequence(), 6);
    assert_eq!(history.first_retained_sequence(), 4);
    assert_eq!(history.omitted_history_count(), 4);
    assert!(history.retained_bytes() <= 200);
    let first = history.page(0, 1);
    assert_eq!(first.entries.len(), 1);
    assert_eq!(first.entries[0].sequence, 4);
    assert_eq!(first.next_sequence, 5);
    assert_eq!(history.page(first.next_sequence, 1).entries[0].sequence, 5);
    assert!(history.page(6, 1).entries.is_empty());
}

#[test]
fn oversized_entry_evicts_prior_entries_without_creating_holes() {
    let limits = PlayerLimits {
        history_bytes: 125,
        ..HARD_LIMITS
    };
    let mut history = HistoryBuffer::new(limits).unwrap();
    history
        .append(SemanticEvent::End, vec![], "scene".into())
        .unwrap();
    history
        .append(
            SemanticEvent::Narration("x".repeat(100)),
            vec![],
            "scene".into(),
        )
        .unwrap();
    assert_eq!(history.first_retained_sequence(), 2);
    assert_eq!(history.omitted_history_count(), 2);
    history
        .append(SemanticEvent::End, vec![], "scene".into())
        .unwrap();
    assert_eq!(history.page(0, 256).entries[0].sequence, 2);
}

#[test]
fn invalid_limits_and_oversized_rendered_event_leave_history_unchanged() {
    assert!(
        HistoryBuffer::new(PlayerLimits {
            history_entries: 0,
            ..HARD_LIMITS
        })
        .is_err()
    );
    let limits = PlayerLimits {
        rendered_bytes: 16,
        ..HARD_LIMITS
    };
    let mut history = HistoryBuffer::new(limits).unwrap();
    assert!(
        history
            .append(
                SemanticEvent::Narration("a".repeat(20)),
                vec![],
                "scene".into()
            )
            .is_err()
    );
    assert!(
        history
            .append(
                SemanticEvent::End,
                vec![MediaEffect::Sfx("b".repeat(20))],
                "scene".into()
            )
            .is_err()
    );
    assert_eq!(history.next_sequence(), 0);
    assert_eq!(history.omitted_history_count(), 0);
    assert_eq!(history.first_retained_sequence(), 0);
}

fn script(prep: Vec<PrepStatement>, story: Vec<StoryStatement>) -> Script {
    Script {
        init: InitBlock {
            variables: vec![VarDecl {
                name: "count".into(),
                var_type: VarType::Integer,
                value: Expr::IntLit(0),
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
        scenes: vec![Scene {
            label: "first".into(),
            prep: Some(PrepBlock {
                statements: prep,
                line: 0,
                column: 0,
            }),
            story: StoryBlock {
                statements: story,
                line: 0,
                column: 0,
            },
            line: 0,
            column: 0,
        }],
    }
}

#[test]
fn failed_open_does_not_expose_partially_mutated_init_or_prep() {
    let story = script(
        vec![
            PrepStatement::VarAssign(VarAssign {
                name: "count".into(),
                op: AssignOp::Set,
                value: Expr::IntLit(42),
                line: 0,
                column: 0,
            }),
            PrepStatement::Repeat(PrepRepeat {
                count: RepeatCount::IntLiteral {
                    value: 1_000_000,
                    line: 0,
                    column: 0,
                },
                body: vec![],
                line: 0,
                column: 0,
            }),
        ],
        vec![StoryStatement::End { line: 0, column: 0 }],
    );
    let limits = PlayerLimits {
        operations_per_interaction: 20,
        ..HARD_LIMITS
    };
    let error = match Engine::open_checked(&story, [1; 32], limits) {
        Ok(_) => panic!("budget failure must not expose an engine"),
        Err(error) => error,
    };
    assert_eq!(error.code, "R_EXECUTION_LIMIT");
    assert_eq!(
        error.resource.as_deref(),
        Some("operations_per_interaction")
    );
    assert_eq!((error.actual, error.limit), (Some(21), Some(20)));
}

#[test]
fn checked_choice_rolls_back_state_when_target_prep_exhausts_budget() {
    let mut story = script(
        vec![],
        vec![StoryStatement::Choice(
            storyscript_parser::ast::ChoiceBlock {
                entries: vec![storyscript_parser::ast::ChoiceEntry::Option(
                    storyscript_parser::ast::ChoiceOption {
                        text: "next".into(),
                        target: "second".into(),
                        line: 0,
                        column: 0,
                    },
                )],
                line: 0,
                column: 0,
            },
        )],
    );
    let mut second = story.scenes[0].clone();
    second.label = "second".into();
    second.prep.as_mut().unwrap().statements = vec![
        PrepStatement::VarAssign(VarAssign {
            name: "count".into(),
            op: AssignOp::Set,
            value: Expr::IntLit(123),
            line: 0,
            column: 0,
        }),
        PrepStatement::Repeat(PrepRepeat {
            count: RepeatCount::IntLiteral {
                value: 10_000,
                line: 0,
                column: 0,
            },
            body: vec![],
            line: 0,
            column: 0,
        }),
    ];
    story.scenes.push(second);
    let mut engine = Engine::open_checked(
        &story,
        [1; 32],
        PlayerLimits {
            operations_per_interaction: 20,
            ..HARD_LIMITS
        },
    )
    .unwrap();
    assert!(matches!(
        engine.advance_checked(),
        Ok(Some((SemanticEvent::SceneTransition(_), _)))
    ));
    assert!(matches!(
        engine.advance_checked(),
        Ok(Some((SemanticEvent::Choices(_), _)))
    ));
    assert_eq!(
        engine.advance_checked().unwrap_err().code,
        "R_INVALID_ACTION"
    );
    assert_eq!(
        engine
            .choose_checked(&ChoiceDisplay {
                text: "forged".into(),
                target: "second".into()
            })
            .unwrap_err()
            .code,
        "R_INVALID_CHOICE"
    );
    let state = engine.rng_state();
    let error = match engine.choose_checked(&ChoiceDisplay {
        text: "next".into(),
        target: "second".into(),
    }) {
        Ok(_) => panic!("limit should fail"),
        Err(error) => error,
    };
    assert_eq!(error.code, "R_EXECUTION_LIMIT");
    assert_eq!(engine.current_scene, "first");
    assert_eq!(engine.variables.get("count"), Some(&Value::Int(0)));
    assert_eq!(engine.rng_state(), state);
}

#[test]
fn pending_and_render_limits_reject_unbounded_scene_output() {
    let many_events = script(
        vec![],
        vec![
            StoryStatement::Narration {
                text: "one".into(),
                line: 0,
                column: 0,
            },
            StoryStatement::Narration {
                text: "two".into(),
                line: 0,
                column: 0,
            },
            StoryStatement::End { line: 0, column: 0 },
        ],
    );
    let error = match Engine::open_checked(
        &many_events,
        [1; 32],
        PlayerLimits {
            pending_events_per_scene: 2,
            ..HARD_LIMITS
        },
    ) {
        Ok(_) => panic!("pending limit should fail"),
        Err(error) => error,
    };
    assert_eq!(error.resource.as_deref(), Some("pending_events_per_scene"));
    let long_text = script(
        vec![],
        vec![
            StoryStatement::Narration {
                text: "x".repeat(100),
                line: 0,
                column: 0,
            },
            StoryStatement::End { line: 0, column: 0 },
        ],
    );
    let error = match Engine::open_checked(
        &long_text,
        [1; 32],
        PlayerLimits {
            rendered_bytes: 20,
            ..HARD_LIMITS
        },
    ) {
        Ok(_) => panic!("render limit should fail"),
        Err(error) => error,
    };
    assert_eq!(error.resource.as_deref(), Some("rendered_bytes"));
}

#[test]
fn recursion_and_array_expansion_obey_lowered_limits() {
    let mut recursive = script(
        vec![PrepStatement::Call {
            name: "recur".into(),
            args: vec![],
            line: 0,
            column: 0,
        }],
        vec![StoryStatement::End { line: 0, column: 0 }],
    );
    recursive.logic_blocks.push(LogicBlock {
        name: "recur".into(),
        params: vec![],
        return_type: None,
        body: vec![PrepStatement::Call {
            name: "recur".into(),
            args: vec![],
            line: 0,
            column: 0,
        }],
        line: 0,
        column: 0,
    });
    let error = match Engine::open_checked(
        &recursive,
        [1; 32],
        PlayerLimits {
            logic_depth: 3,
            ..HARD_LIMITS
        },
    ) {
        Ok(_) => panic!("recursive logic should fail"),
        Err(error) => error,
    };
    assert_eq!(error.resource.as_deref(), Some("logic_depth"));

    let arrays = script(
        vec![PrepStatement::VarDecl(VarDecl {
            name: "items".into(),
            var_type: VarType::ArrayInteger,
            value: Expr::ListLit {
                items: vec![Expr::IntLit(1), Expr::IntLit(2), Expr::IntLit(3)],
                line: 0,
                column: 0,
            },
            line: 0,
            column: 0,
        })],
        vec![StoryStatement::End { line: 0, column: 0 }],
    );
    let error = match Engine::open_checked(
        &arrays,
        [1; 32],
        PlayerLimits {
            array_elements: 2,
            ..HARD_LIMITS
        },
    ) {
        Ok(_) => panic!("array literal should fail"),
        Err(error) => error,
    };
    assert_eq!(error.resource.as_deref(), Some("array_elements"));
}
