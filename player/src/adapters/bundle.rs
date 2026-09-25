//! Adapter from the non-forgeable verified StoryBundle capability.

use rust_decimal::Decimal;
use storyscript_bundle::loader::LoadedBundle;
use storyscript_bundle::proto::storybundle::v1 as b;
use storyscript_parser::interpolation::ESCAPED_DOLLAR_MARKER;

use crate::contract::{BundleSigner, Origin, RuntimeError};
use crate::model as m;
use crate::save::{RUNTIME_IDENTITY, semantic_sha256};

pub fn adapt_verified(bundle: &LoadedBundle) -> Result<(m::StoryModel, Origin), RuntimeError> {
    let model = adapt_story(bundle.story())?;
    let manifest = bundle.manifest();
    let status = bundle.verification_status();
    let signer = match (&status.signer_key_id, status.is_unsigned_development) {
        (Some(id), false) => BundleSigner::KeyId(id.clone()),
        (None, true) => BundleSigner::UnsignedDevelopment,
        _ => return Err(error("verified bundle has impossible signer status")),
    };
    let origin = Origin::Bundle {
        format_version: manifest.format_version,
        compiler_version: manifest.compiler_version.clone(),
        schema_sha256: manifest.schema_sha256.clone(),
        project_id: manifest.project.id.clone(),
        project_version: manifest.project.version.clone(),
        compiled_entry_sha256: bundle.compiled_entry_sha256().to_string(),
        signer,
        runtime_identity: RUNTIME_IDENTITY.into(),
        semantic_sha256: semantic_sha256(&model),
    };
    Ok((model, origin))
}

fn adapt_story(story: &b::CompiledStory) -> Result<m::StoryModel, RuntimeError> {
    let init = required(story.initialization.as_ref(), "initialization")?;
    Ok(m::StoryModel {
        init: m::InitBlock {
            variables: init
                .variables
                .iter()
                .map(variable)
                .collect::<Result<_, _>>()?,
            actors: init.actors.iter().map(actor).collect::<Result<_, _>>()?,
            start: init.start_scene.clone(),
            span: m::SourceSpan::UNKNOWN,
        },
        logic_blocks: story
            .logic_blocks
            .iter()
            .map(logic)
            .collect::<Result<_, _>>()?,
        scenes: story.scenes.iter().map(scene).collect::<Result<_, _>>()?,
    })
}

fn variable(value: &b::VariableDefinition) -> Result<m::VarDecl, RuntimeError> {
    Ok(m::VarDecl {
        name: value.name.clone(),
        var_type: var_type(value.r#type)?,
        value: expression(required(value.value.as_ref(), "variable value")?)?,
        span: m::SourceSpan::UNKNOWN,
    })
}

fn actor(value: &b::Actor) -> Result<m::ActorDecl, RuntimeError> {
    Ok(m::ActorDecl {
        id: value.id.clone(),
        display_name: template(required(value.display_name.as_ref(), "actor display name")?)?,
        portraits: value
            .portraits
            .iter()
            .map(|portrait| {
                Ok(m::PortraitEntry {
                    emotion: portrait.emotion.clone(),
                    path: template(required(portrait.asset_path.as_ref(), "portrait path")?)?,
                    span: m::SourceSpan::UNKNOWN,
                })
            })
            .collect::<Result<_, RuntimeError>>()?,
        span: m::SourceSpan::UNKNOWN,
    })
}

fn logic(value: &b::LogicBlock) -> Result<m::LogicBlock, RuntimeError> {
    Ok(m::LogicBlock {
        name: value.name.clone(),
        params: value
            .parameters
            .iter()
            .map(|parameter| {
                Ok(m::LogicParam {
                    name: parameter.name.clone(),
                    var_type: var_type(parameter.r#type)?,
                    span: m::SourceSpan::UNKNOWN,
                })
            })
            .collect::<Result<_, RuntimeError>>()?,
        return_type: value.return_type.map(var_type).transpose()?,
        body: value.body.iter().map(prep).collect::<Result<_, _>>()?,
        span: m::SourceSpan::UNKNOWN,
    })
}

fn scene(value: &b::Scene) -> Result<m::Scene, RuntimeError> {
    Ok(m::Scene {
        label: value.label.clone(),
        prep: value
            .prep
            .as_ref()
            .map(|block| block.statements.iter().map(prep).collect())
            .transpose()?
            .unwrap_or_default(),
        story: required(value.story.as_ref(), "scene story")?
            .statements
            .iter()
            .map(story)
            .collect::<Result<_, _>>()?,
        span: m::SourceSpan::UNKNOWN,
    })
}

fn prep(value: &b::PrepStatement) -> Result<m::PrepStatement, RuntimeError> {
    use b::prep_statement::Value;
    Ok(match required(value.value.as_ref(), "PREP statement")? {
        Value::Background(value) => m::PrepStatement::BgDirective {
            path: template(required(value.asset_path.as_ref(), "background path")?)?,
            span: m::SourceSpan::UNKNOWN,
        },
        Value::Bgm(value) => m::PrepStatement::BgmDirective {
            value: match required(value.value.as_ref(), "BGM value")? {
                b::bgm_directive::Value::AssetPath(path) => m::BgmValue::Path(template(path)?),
                b::bgm_directive::Value::Stop(_) => m::BgmValue::Stop,
            },
            span: m::SourceSpan::UNKNOWN,
        },
        Value::Sfx(value) => m::PrepStatement::SfxDirective {
            path: template(required(value.asset_path.as_ref(), "SFX path")?)?,
            span: m::SourceSpan::UNKNOWN,
        },
        Value::VariableDefinition(value) => m::PrepStatement::VarDecl(variable(value)?),
        Value::VariableAssignment(value) => m::PrepStatement::VarAssign(m::VarAssign {
            name: value.name.clone(),
            op: assign_op(value.operator)?,
            value: expression(required(value.value.as_ref(), "assignment value")?)?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::Call(value) => m::PrepStatement::Call {
            name: value.name.clone(),
            args: value
                .arguments
                .iter()
                .map(expression)
                .collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        },
        Value::IfElse(value) => m::PrepStatement::IfElse(m::PrepIfElse {
            condition: expression(required(value.condition.as_ref(), "PREP if condition")?)?,
            then_branch: required(value.then_branch.as_ref(), "PREP then branch")?
                .statements
                .iter()
                .map(prep)
                .collect::<Result<_, _>>()?,
            else_branch: value
                .else_branch
                .as_ref()
                .map(|branch| branch.statements.iter().map(prep).collect())
                .transpose()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::ForSnapshot(value) => m::PrepStatement::ForSnapshot(m::PrepForSnapshot {
            item_name: value.item_name.clone(),
            array_name: value.array_name.clone(),
            body: value.body.iter().map(prep).collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::Repeat(value) => m::PrepStatement::Repeat(m::PrepRepeat {
            count: repeat(required(value.count.as_ref(), "PREP repeat count")?)?,
            body: value.body.iter().map(prep).collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::BreakLoop(_) => m::PrepStatement::Break {
            span: m::SourceSpan::UNKNOWN,
        },
        Value::ContinueLoop(_) => m::PrepStatement::Continue {
            span: m::SourceSpan::UNKNOWN,
        },
        Value::ReturnStatement(value) => m::PrepStatement::Return {
            value: value.value.as_ref().map(expression).transpose()?,
            span: m::SourceSpan::UNKNOWN,
        },
    })
}

fn story(value: &b::StoryStatement) -> Result<m::StoryStatement, RuntimeError> {
    use b::story_statement::Value;
    Ok(match required(value.value.as_ref(), "STORY statement")? {
        Value::Narration(value) => m::StoryStatement::Narration {
            text: template(required(value.text.as_ref(), "narration text")?)?,
            span: m::SourceSpan::UNKNOWN,
        },
        Value::VariableOutput(value) => m::StoryStatement::VarOutput {
            name: value.name.clone(),
            span: m::SourceSpan::UNKNOWN,
        },
        Value::Dialogue(value) => m::StoryStatement::Dialogue(m::Dialogue {
            actor_id: value.actor_id.clone(),
            form: match required(value.form.as_ref(), "dialogue form")? {
                b::dialogue::Form::NameOnly(_) => m::DialogueForm::NameOnly,
                b::dialogue::Form::Portrait(value) => m::DialogueForm::Portrait {
                    emotion: value.emotion.clone(),
                    position: position(value.position)?,
                },
            },
            text: template(required(value.text.as_ref(), "dialogue text")?)?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::IfElse(value) => m::StoryStatement::IfElse(m::StoryIfElse {
            condition: expression(required(value.condition.as_ref(), "STORY if condition")?)?,
            then_branch: required(value.then_branch.as_ref(), "STORY then branch")?
                .statements
                .iter()
                .map(story)
                .collect::<Result<_, _>>()?,
            else_branch: value
                .else_branch
                .as_ref()
                .map(|branch| branch.statements.iter().map(story).collect())
                .transpose()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::Choice(value) => m::StoryStatement::Choice(m::ChoiceBlock {
            entries: value.entries.iter().map(choice).collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::Jump(value) => m::StoryStatement::Jump {
            target: value.target.clone(),
            span: m::SourceSpan::UNKNOWN,
        },
        Value::End(_) => m::StoryStatement::End {
            span: m::SourceSpan::UNKNOWN,
        },
        Value::Sfx(value) => m::StoryStatement::SfxDirective {
            path: template(required(value.asset_path.as_ref(), "SFX path")?)?,
            span: m::SourceSpan::UNKNOWN,
        },
        Value::ForSnapshot(value) => m::StoryStatement::ForSnapshot(m::StoryForSnapshot {
            item_name: value.item_name.clone(),
            array_name: value.array_name.clone(),
            body: value.body.iter().map(story).collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::Repeat(value) => m::StoryStatement::Repeat(m::StoryRepeat {
            count: repeat(required(value.count.as_ref(), "STORY repeat count")?)?,
            body: value.body.iter().map(story).collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::BreakLoop(_) => m::StoryStatement::Break {
            span: m::SourceSpan::UNKNOWN,
        },
        Value::ContinueLoop(_) => m::StoryStatement::Continue {
            span: m::SourceSpan::UNKNOWN,
        },
    })
}

fn choice(value: &b::ChoiceEntry) -> Result<m::ChoiceEntry, RuntimeError> {
    use b::choice_entry::Value;
    Ok(match required(value.value.as_ref(), "choice entry")? {
        Value::Option(value) => m::ChoiceEntry::Option(m::ChoiceOption {
            text: template(required(value.text.as_ref(), "choice text")?)?,
            target: value.target.clone(),
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::IfEntry(value) => m::ChoiceEntry::If(m::ChoiceIfEntry {
            condition: expression(required(value.condition.as_ref(), "choice condition")?)?,
            body: required(value.body.as_ref(), "choice body")?
                .entries
                .iter()
                .map(choice)
                .collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::Repeat(value) => m::ChoiceEntry::Repeat(m::ChoiceRepeatEntry {
            count: repeat(required(value.count.as_ref(), "choice repeat count")?)?,
            body: required(value.body.as_ref(), "choice repeat body")?
                .entries
                .iter()
                .map(choice)
                .collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        }),
        Value::ForSnapshot(value) => m::ChoiceEntry::ForSnapshot(m::ChoiceForSnapshotEntry {
            item_name: value.item_name.clone(),
            array_name: value.array_name.clone(),
            body: required(value.body.as_ref(), "choice snapshot body")?
                .entries
                .iter()
                .map(choice)
                .collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        }),
    })
}

fn repeat(value: &b::RepeatCount) -> Result<m::RepeatCount, RuntimeError> {
    use b::repeat_count::Value;
    Ok(match required(value.value.as_ref(), "repeat count")? {
        Value::Integer(value) => m::RepeatCount::IntLiteral {
            value: *value,
            span: m::SourceSpan::UNKNOWN,
        },
        Value::Variable(value) => m::RepeatCount::Variable {
            name: value.clone(),
            span: m::SourceSpan::UNKNOWN,
        },
    })
}

fn expression(value: &b::Expression) -> Result<m::Expr, RuntimeError> {
    use b::expression::Value;
    Ok(match required(value.value.as_ref(), "expression")? {
        Value::Integer(value) => m::Expr::IntLit(*value),
        Value::Decimal(value) => m::Expr::DecimalLit(
            Decimal::from_str_exact(&value.canonical)
                .map_err(|_| error("invalid compiled decimal"))?,
        ),
        Value::Boolean(value) => m::Expr::BoolLit(*value),
        Value::StringValue(value) => m::Expr::StringLit(template(value)?),
        Value::Variable(value) => m::Expr::VarRef {
            name: value.name.clone(),
            span: m::SourceSpan::UNKNOWN,
        },
        Value::Binary(value) => m::Expr::BinOp {
            left: Box::new(expression(required(value.left.as_deref(), "binary left")?)?),
            op: binary_op(value.operator)?,
            right: Box::new(expression(required(
                value.right.as_deref(),
                "binary right",
            )?)?),
        },
        Value::Call(value) => m::Expr::Call {
            name: value.name.clone(),
            args: value
                .arguments
                .iter()
                .map(expression)
                .collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        },
        Value::List(value) => m::Expr::ListLit {
            items: value
                .items
                .iter()
                .map(expression)
                .collect::<Result<_, _>>()?,
            span: m::SourceSpan::UNKNOWN,
        },
    })
}

fn template(value: &b::InterpolatedString) -> Result<String, RuntimeError> {
    use b::string_segment::Value;
    let mut output = String::new();
    for segment in &value.segments {
        match required(segment.value.as_ref(), "string segment")? {
            Value::Literal(value) => {
                for character in value.chars() {
                    output.push(if character == '$' {
                        ESCAPED_DOLLAR_MARKER
                    } else {
                        character
                    });
                }
            }
            Value::Variable(value) => {
                output.push_str("${");
                output.push_str(value);
                output.push('}');
            }
        }
    }
    Ok(output)
}

fn var_type(value: i32) -> Result<m::VarType, RuntimeError> {
    match b::VariableType::try_from(value) {
        Ok(b::VariableType::Integer) => Ok(m::VarType::Integer),
        Ok(b::VariableType::String) => Ok(m::VarType::String),
        Ok(b::VariableType::Boolean) => Ok(m::VarType::Boolean),
        Ok(b::VariableType::Decimal) => Ok(m::VarType::Decimal),
        Ok(b::VariableType::ArrayInteger) => Ok(m::VarType::ArrayInteger),
        Ok(b::VariableType::ArrayString) => Ok(m::VarType::ArrayString),
        Ok(b::VariableType::ArrayBoolean) => Ok(m::VarType::ArrayBoolean),
        Ok(b::VariableType::ArrayDecimal) => Ok(m::VarType::ArrayDecimal),
        _ => Err(error("unknown compiled variable type")),
    }
}

fn assign_op(value: i32) -> Result<m::AssignOp, RuntimeError> {
    match b::AssignmentOperator::try_from(value) {
        Ok(b::AssignmentOperator::Set) => Ok(m::AssignOp::Set),
        Ok(b::AssignmentOperator::Add) => Ok(m::AssignOp::AddEq),
        Ok(b::AssignmentOperator::Subtract) => Ok(m::AssignOp::SubEq),
        _ => Err(error("unknown compiled assignment operator")),
    }
}

fn binary_op(value: i32) -> Result<m::BinOperator, RuntimeError> {
    match b::BinaryOperator::try_from(value) {
        Ok(b::BinaryOperator::Add) => Ok(m::BinOperator::Add),
        Ok(b::BinaryOperator::Subtract) => Ok(m::BinOperator::Sub),
        Ok(b::BinaryOperator::Multiply) => Ok(m::BinOperator::Mul),
        Ok(b::BinaryOperator::Divide) => Ok(m::BinOperator::Div),
        Ok(b::BinaryOperator::Modulo) => Ok(m::BinOperator::Mod),
        Ok(b::BinaryOperator::Equal) => Ok(m::BinOperator::EqEq),
        Ok(b::BinaryOperator::NotEqual) => Ok(m::BinOperator::NotEq),
        Ok(b::BinaryOperator::Less) => Ok(m::BinOperator::Lt),
        Ok(b::BinaryOperator::LessEqual) => Ok(m::BinOperator::LtEq),
        Ok(b::BinaryOperator::Greater) => Ok(m::BinOperator::Gt),
        Ok(b::BinaryOperator::GreaterEqual) => Ok(m::BinOperator::GtEq),
        _ => Err(error("unknown compiled binary operator")),
    }
}

fn position(value: i32) -> Result<m::Position, RuntimeError> {
    match b::Position::try_from(value) {
        Ok(b::Position::Left) => Ok(m::Position::Left),
        Ok(b::Position::Center) => Ok(m::Position::Center),
        Ok(b::Position::Right) => Ok(m::Position::Right),
        _ => Err(error("unknown compiled dialogue position")),
    }
}

fn required<'a, T>(value: Option<&'a T>, name: &str) -> Result<&'a T, RuntimeError> {
    value.ok_or_else(|| error(&format!("validated bundle is missing {name}")))
}

fn error(message: &str) -> RuntimeError {
    RuntimeError {
        code: "R_BUNDLE_MODEL_INVALID".into(),
        scene: String::new(),
        message: message.into(),
        resource: None,
        actual: None,
        limit: None,
    }
}
