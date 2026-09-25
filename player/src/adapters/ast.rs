//! Exhaustive conversion from parser AST into the execution-owned model.

use crate::model as m;
use storyscript_parser::ast as a;

fn span(line: usize, column: usize) -> m::SourceSpan {
    m::SourceSpan { line, column }
}

pub fn adapt(script: &a::Script) -> m::StoryModel {
    m::StoryModel {
        init: m::InitBlock {
            variables: script.init.variables.iter().map(var_decl).collect(),
            actors: script.init.actors.iter().map(actor).collect(),
            start: script.init.start.target.clone(),
            span: span(script.init.line, script.init.column),
        },
        logic_blocks: script.logic_blocks.iter().map(logic).collect(),
        scenes: script.scenes.iter().map(scene).collect(),
    }
}

fn var_type(value: a::VarType) -> m::VarType {
    match value {
        a::VarType::Integer => m::VarType::Integer,
        a::VarType::String => m::VarType::String,
        a::VarType::Boolean => m::VarType::Boolean,
        a::VarType::Decimal => m::VarType::Decimal,
        a::VarType::ArrayInteger => m::VarType::ArrayInteger,
        a::VarType::ArrayString => m::VarType::ArrayString,
        a::VarType::ArrayBoolean => m::VarType::ArrayBoolean,
        a::VarType::ArrayDecimal => m::VarType::ArrayDecimal,
    }
}

fn var_decl(value: &a::VarDecl) -> m::VarDecl {
    m::VarDecl {
        name: value.name.clone(),
        var_type: var_type(value.var_type),
        value: expr(&value.value),
        span: span(value.line, value.column),
    }
}

fn actor(value: &a::ActorDecl) -> m::ActorDecl {
    m::ActorDecl {
        id: value.id.clone(),
        display_name: value.display_name.clone(),
        portraits: value
            .portraits
            .iter()
            .map(|item| m::PortraitEntry {
                emotion: item.emotion.clone(),
                path: item.path.clone(),
                span: span(item.line, item.column),
            })
            .collect(),
        span: span(value.line, value.column),
    }
}

fn logic(value: &a::LogicBlock) -> m::LogicBlock {
    m::LogicBlock {
        name: value.name.clone(),
        params: value
            .params
            .iter()
            .map(|item| m::LogicParam {
                name: item.name.clone(),
                var_type: var_type(item.var_type),
                span: span(item.line, item.column),
            })
            .collect(),
        return_type: value.return_type.map(var_type),
        body: value.body.iter().map(prep).collect(),
        span: span(value.line, value.column),
    }
}

fn scene(value: &a::Scene) -> m::Scene {
    m::Scene {
        label: value.label.clone(),
        prep: value
            .prep
            .as_ref()
            .map(|block| block.statements.iter().map(prep).collect())
            .unwrap_or_default(),
        story: value.story.statements.iter().map(story).collect(),
        span: span(value.line, value.column),
    }
}

fn prep(value: &a::PrepStatement) -> m::PrepStatement {
    match value {
        a::PrepStatement::BgDirective { path, line, column } => m::PrepStatement::BgDirective {
            path: path.clone(),
            span: span(*line, *column),
        },
        a::PrepStatement::BgmDirective {
            value,
            line,
            column,
        } => m::PrepStatement::BgmDirective {
            value: match value {
                a::BgmValue::Path(path) => m::BgmValue::Path(path.clone()),
                a::BgmValue::Stop => m::BgmValue::Stop,
            },
            span: span(*line, *column),
        },
        a::PrepStatement::SfxDirective { path, line, column } => m::PrepStatement::SfxDirective {
            path: path.clone(),
            span: span(*line, *column),
        },
        a::PrepStatement::VarDecl(value) => m::PrepStatement::VarDecl(var_decl(value)),
        a::PrepStatement::VarAssign(value) => m::PrepStatement::VarAssign(m::VarAssign {
            name: value.name.clone(),
            op: match value.op {
                a::AssignOp::Set => m::AssignOp::Set,
                a::AssignOp::AddEq => m::AssignOp::AddEq,
                a::AssignOp::SubEq => m::AssignOp::SubEq,
            },
            value: expr(&value.value),
            span: span(value.line, value.column),
        }),
        a::PrepStatement::Call {
            name,
            args,
            line,
            column,
        } => m::PrepStatement::Call {
            name: name.clone(),
            args: args.iter().map(expr).collect(),
            span: span(*line, *column),
        },
        a::PrepStatement::IfElse(value) => m::PrepStatement::IfElse(m::PrepIfElse {
            condition: expr(&value.condition),
            then_branch: value.then_branch.iter().map(prep).collect(),
            else_branch: value
                .else_branch
                .as_ref()
                .map(|items| items.iter().map(prep).collect()),
            span: span(value.line, value.column),
        }),
        a::PrepStatement::ForSnapshot(value) => m::PrepStatement::ForSnapshot(m::PrepForSnapshot {
            item_name: value.item_name.clone(),
            array_name: value.array_name.clone(),
            body: value.body.iter().map(prep).collect(),
            span: span(value.line, value.column),
        }),
        a::PrepStatement::Repeat(value) => m::PrepStatement::Repeat(m::PrepRepeat {
            count: repeat(&value.count),
            body: value.body.iter().map(prep).collect(),
            span: span(value.line, value.column),
        }),
        a::PrepStatement::Break { line, column } => m::PrepStatement::Break {
            span: span(*line, *column),
        },
        a::PrepStatement::Continue { line, column } => m::PrepStatement::Continue {
            span: span(*line, *column),
        },
        a::PrepStatement::Return {
            value,
            line,
            column,
        } => m::PrepStatement::Return {
            value: value.as_ref().map(expr),
            span: span(*line, *column),
        },
    }
}

fn story(value: &a::StoryStatement) -> m::StoryStatement {
    match value {
        a::StoryStatement::Narration { text, line, column } => m::StoryStatement::Narration {
            text: text.clone(),
            span: span(*line, *column),
        },
        a::StoryStatement::VarOutput { name, line, column } => m::StoryStatement::VarOutput {
            name: name.clone(),
            span: span(*line, *column),
        },
        a::StoryStatement::Dialogue(value) => m::StoryStatement::Dialogue(m::Dialogue {
            actor_id: value.actor_id.clone(),
            form: match &value.form {
                a::DialogueForm::NameOnly => m::DialogueForm::NameOnly,
                a::DialogueForm::Portrait { emotion, position } => m::DialogueForm::Portrait {
                    emotion: emotion.clone(),
                    position: match position {
                        a::Position::Left => m::Position::Left,
                        a::Position::Center => m::Position::Center,
                        a::Position::Right => m::Position::Right,
                    },
                },
            },
            text: value.text.clone(),
            span: span(value.line, value.column),
        }),
        a::StoryStatement::IfElse(value) => m::StoryStatement::IfElse(m::StoryIfElse {
            condition: expr(&value.condition),
            then_branch: value.then_branch.iter().map(story).collect(),
            else_branch: value
                .else_branch
                .as_ref()
                .map(|items| items.iter().map(story).collect()),
            span: span(value.line, value.column),
        }),
        a::StoryStatement::Choice(value) => m::StoryStatement::Choice(m::ChoiceBlock {
            entries: value.entries.iter().map(choice).collect(),
            span: span(value.line, value.column),
        }),
        a::StoryStatement::Jump {
            target,
            line,
            column,
        } => m::StoryStatement::Jump {
            target: target.clone(),
            span: span(*line, *column),
        },
        a::StoryStatement::End { line, column } => m::StoryStatement::End {
            span: span(*line, *column),
        },
        a::StoryStatement::SfxDirective { path, line, column } => m::StoryStatement::SfxDirective {
            path: path.clone(),
            span: span(*line, *column),
        },
        a::StoryStatement::ForSnapshot(value) => {
            m::StoryStatement::ForSnapshot(m::StoryForSnapshot {
                item_name: value.item_name.clone(),
                array_name: value.array_name.clone(),
                body: value.body.iter().map(story).collect(),
                span: span(value.line, value.column),
            })
        }
        a::StoryStatement::Repeat(value) => m::StoryStatement::Repeat(m::StoryRepeat {
            count: repeat(&value.count),
            body: value.body.iter().map(story).collect(),
            span: span(value.line, value.column),
        }),
        a::StoryStatement::Break { line, column } => m::StoryStatement::Break {
            span: span(*line, *column),
        },
        a::StoryStatement::Continue { line, column } => m::StoryStatement::Continue {
            span: span(*line, *column),
        },
    }
}

fn choice(value: &a::ChoiceEntry) -> m::ChoiceEntry {
    match value {
        a::ChoiceEntry::Option(value) => m::ChoiceEntry::Option(m::ChoiceOption {
            text: value.text.clone(),
            target: value.target.clone(),
            span: span(value.line, value.column),
        }),
        a::ChoiceEntry::If(value) => m::ChoiceEntry::If(m::ChoiceIfEntry {
            condition: expr(&value.condition),
            body: value.body.iter().map(choice).collect(),
            span: span(value.line, value.column),
        }),
        a::ChoiceEntry::Repeat(value) => m::ChoiceEntry::Repeat(m::ChoiceRepeatEntry {
            count: repeat(&value.count),
            body: value.body.iter().map(choice).collect(),
            span: span(value.line, value.column),
        }),
        a::ChoiceEntry::ForSnapshot(value) => {
            m::ChoiceEntry::ForSnapshot(m::ChoiceForSnapshotEntry {
                item_name: value.item_name.clone(),
                array_name: value.array_name.clone(),
                body: value.body.iter().map(choice).collect(),
                span: span(value.line, value.column),
            })
        }
    }
}

fn repeat(value: &a::RepeatCount) -> m::RepeatCount {
    match value {
        a::RepeatCount::IntLiteral {
            value,
            line,
            column,
        } => m::RepeatCount::IntLiteral {
            value: *value,
            span: span(*line, *column),
        },
        a::RepeatCount::Variable { name, line, column } => m::RepeatCount::Variable {
            name: name.clone(),
            span: span(*line, *column),
        },
    }
}

fn expr(value: &a::Expr) -> m::Expr {
    match value {
        a::Expr::IntLit(value) => m::Expr::IntLit(*value),
        a::Expr::DecimalLit(value) => m::Expr::DecimalLit(*value),
        a::Expr::BoolLit(value) => m::Expr::BoolLit(*value),
        a::Expr::StringLit(value) => m::Expr::StringLit(value.clone()),
        a::Expr::VarRef { name, line, column } => m::Expr::VarRef {
            name: name.clone(),
            span: span(*line, *column),
        },
        a::Expr::BinOp { left, op, right } => m::Expr::BinOp {
            left: Box::new(expr(left)),
            op: match op {
                a::BinOperator::Add => m::BinOperator::Add,
                a::BinOperator::Sub => m::BinOperator::Sub,
                a::BinOperator::Mul => m::BinOperator::Mul,
                a::BinOperator::Div => m::BinOperator::Div,
                a::BinOperator::Mod => m::BinOperator::Mod,
                a::BinOperator::EqEq => m::BinOperator::EqEq,
                a::BinOperator::NotEq => m::BinOperator::NotEq,
                a::BinOperator::Lt => m::BinOperator::Lt,
                a::BinOperator::LtEq => m::BinOperator::LtEq,
                a::BinOperator::Gt => m::BinOperator::Gt,
                a::BinOperator::GtEq => m::BinOperator::GtEq,
            },
            right: Box::new(expr(right)),
        },
        a::Expr::Call {
            name,
            args,
            line,
            column,
        } => m::Expr::Call {
            name: name.clone(),
            args: args.iter().map(expr).collect(),
            span: span(*line, *column),
        },
        a::Expr::ListLit {
            items,
            line,
            column,
        } => m::Expr::ListLit {
            items: items.iter().map(expr).collect(),
            span: span(*line, *column),
        },
    }
}
