use storyscript_parser::ast;

use crate::config::Project;
use crate::proto::storybundle::v1 as pb;
use crate::{FORMAT_VERSION, Result, template};

pub fn convert(script: &ast::Script, project: &Project) -> Result<pb::CompiledStory> {
    Ok(pb::CompiledStory {
        format_version: FORMAT_VERSION,
        project: Some(pb::ProjectMetadata {
            id: project.id.clone(),
            name: project.name.clone(),
            version: project.version.clone(),
        }),
        initialization: Some(pb::Initialization {
            variables: script
                .init
                .variables
                .iter()
                .map(variable_definition)
                .collect::<Result<_>>()?,
            actors: script
                .init
                .actors
                .iter()
                .map(actor)
                .collect::<Result<_>>()?,
            start_scene: script.init.start.target.clone(),
        }),
        logic_blocks: script
            .logic_blocks
            .iter()
            .map(logic_block)
            .collect::<Result<_>>()?,
        scenes: script.scenes.iter().map(scene).collect::<Result<_>>()?,
    })
}

fn actor(value: &ast::ActorDecl) -> Result<pb::Actor> {
    Ok(pb::Actor {
        id: value.id.clone(),
        display_name: Some(template::compile(&value.display_name)?),
        portraits: value
            .portraits
            .iter()
            .map(|portrait| {
                Ok(pb::Portrait {
                    emotion: portrait.emotion.clone(),
                    asset_path: Some(template::compile(&portrait.path)?),
                })
            })
            .collect::<Result<_>>()?,
    })
}

fn variable_definition(value: &ast::VarDecl) -> Result<pb::VariableDefinition> {
    Ok(pb::VariableDefinition {
        name: value.name.clone(),
        r#type: variable_type(value.var_type).into(),
        value: Some(expression(&value.value)?),
    })
}

fn variable_type(value: ast::VarType) -> pb::VariableType {
    match value {
        ast::VarType::Integer => pb::VariableType::Integer,
        ast::VarType::String => pb::VariableType::String,
        ast::VarType::Boolean => pb::VariableType::Boolean,
        ast::VarType::Decimal => pb::VariableType::Decimal,
        ast::VarType::ArrayInteger => pb::VariableType::ArrayInteger,
        ast::VarType::ArrayString => pb::VariableType::ArrayString,
        ast::VarType::ArrayBoolean => pb::VariableType::ArrayBoolean,
        ast::VarType::ArrayDecimal => pb::VariableType::ArrayDecimal,
    }
}

fn logic_block(value: &ast::LogicBlock) -> Result<pb::LogicBlock> {
    Ok(pb::LogicBlock {
        name: value.name.clone(),
        parameters: value
            .params
            .iter()
            .map(|parameter| pb::LogicParameter {
                name: parameter.name.clone(),
                r#type: variable_type(parameter.var_type).into(),
            })
            .collect(),
        return_type: value.return_type.map(|value| variable_type(value).into()),
        body: value
            .body
            .iter()
            .map(prep_statement)
            .collect::<Result<_>>()?,
    })
}

fn scene(value: &ast::Scene) -> Result<pb::Scene> {
    Ok(pb::Scene {
        label: value.label.clone(),
        prep: value
            .prep
            .as_ref()
            .map(|prep| {
                Ok::<pb::PrepBlock, crate::BundleError>(pb::PrepBlock {
                    statements: prep
                        .statements
                        .iter()
                        .map(prep_statement)
                        .collect::<Result<_>>()?,
                })
            })
            .transpose()?,
        story: Some(pb::StoryBlock {
            statements: value
                .story
                .statements
                .iter()
                .map(story_statement)
                .collect::<Result<_>>()?,
        }),
    })
}

fn prep_statement(value: &ast::PrepStatement) -> Result<pb::PrepStatement> {
    use ast::PrepStatement as A;
    use pb::prep_statement::Value;
    let value = match value {
        A::BgDirective { path, .. } => Value::Background(pb::BackgroundDirective {
            asset_path: Some(template::compile(path)?),
        }),
        A::BgmDirective { value, .. } => Value::Bgm(pb::BgmDirective {
            value: Some(match value {
                ast::BgmValue::Path(path) => {
                    pb::bgm_directive::Value::AssetPath(template::compile(path)?)
                }
                ast::BgmValue::Stop => pb::bgm_directive::Value::Stop(()),
            }),
        }),
        A::SfxDirective { path, .. } => Value::Sfx(pb::SfxDirective {
            asset_path: Some(template::compile(path)?),
        }),
        A::VarDecl(declaration) => Value::VariableDefinition(variable_definition(declaration)?),
        A::VarAssign(assignment) => Value::VariableAssignment(variable_assignment(assignment)?),
        A::Call { name, args, .. } => Value::Call(call_expression(name, args)?),
        A::IfElse(branch) => Value::IfElse(pb::PrepIfElse {
            condition: Some(expression(&branch.condition)?),
            then_branch: Some(pb::PrepStatementList {
                statements: branch
                    .then_branch
                    .iter()
                    .map(prep_statement)
                    .collect::<Result<_>>()?,
            }),
            else_branch: branch
                .else_branch
                .as_ref()
                .map(|statements| {
                    Ok::<pb::PrepStatementList, crate::BundleError>(pb::PrepStatementList {
                        statements: statements
                            .iter()
                            .map(prep_statement)
                            .collect::<Result<_>>()?,
                    })
                })
                .transpose()?,
        }),
        A::ForSnapshot(loop_value) => Value::ForSnapshot(pb::PrepForSnapshot {
            item_name: loop_value.item_name.clone(),
            array_name: loop_value.array_name.clone(),
            body: loop_value
                .body
                .iter()
                .map(prep_statement)
                .collect::<Result<_>>()?,
        }),
        A::Repeat(loop_value) => Value::Repeat(pb::PrepRepeat {
            count: Some(repeat_count(&loop_value.count)),
            body: loop_value
                .body
                .iter()
                .map(prep_statement)
                .collect::<Result<_>>()?,
        }),
        A::Break { .. } => Value::BreakLoop(()),
        A::Continue { .. } => Value::ContinueLoop(()),
        A::Return { value, .. } => Value::ReturnStatement(pb::ReturnStatement {
            value: value.as_ref().map(expression).transpose()?,
        }),
    };
    Ok(pb::PrepStatement { value: Some(value) })
}

fn story_statement(value: &ast::StoryStatement) -> Result<pb::StoryStatement> {
    use ast::StoryStatement as A;
    use pb::story_statement::Value;
    let value = match value {
        A::Narration { text, .. } => Value::Narration(pb::Narration {
            text: Some(template::compile(text)?),
        }),
        A::VarOutput { name, .. } => {
            Value::VariableOutput(pb::VariableOutput { name: name.clone() })
        }
        A::Dialogue(dialogue) => Value::Dialogue(pb::Dialogue {
            actor_id: dialogue.actor_id.clone(),
            form: Some(match &dialogue.form {
                ast::DialogueForm::NameOnly => pb::dialogue::Form::NameOnly(()),
                ast::DialogueForm::Portrait { emotion, position } => {
                    pb::dialogue::Form::Portrait(pb::PortraitDialogue {
                        emotion: emotion.clone(),
                        position: match position {
                            ast::Position::Left => pb::Position::Left,
                            ast::Position::Center => pb::Position::Center,
                            ast::Position::Right => pb::Position::Right,
                        }
                        .into(),
                    })
                }
            }),
            text: Some(template::compile(&dialogue.text)?),
        }),
        A::IfElse(branch) => Value::IfElse(pb::StoryIfElse {
            condition: Some(expression(&branch.condition)?),
            then_branch: Some(pb::StoryStatementList {
                statements: branch
                    .then_branch
                    .iter()
                    .map(story_statement)
                    .collect::<Result<_>>()?,
            }),
            else_branch: branch
                .else_branch
                .as_ref()
                .map(|statements| {
                    Ok::<pb::StoryStatementList, crate::BundleError>(pb::StoryStatementList {
                        statements: statements
                            .iter()
                            .map(story_statement)
                            .collect::<Result<_>>()?,
                    })
                })
                .transpose()?,
        }),
        A::Choice(choice) => Value::Choice(pb::ChoiceBlock {
            entries: choice
                .entries
                .iter()
                .map(choice_entry)
                .collect::<Result<_>>()?,
        }),
        A::Jump { target, .. } => Value::Jump(pb::Jump {
            target: target.clone(),
        }),
        A::End { .. } => Value::End(()),
        A::SfxDirective { path, .. } => Value::Sfx(pb::SfxDirective {
            asset_path: Some(template::compile(path)?),
        }),
        A::ForSnapshot(loop_value) => Value::ForSnapshot(pb::StoryForSnapshot {
            item_name: loop_value.item_name.clone(),
            array_name: loop_value.array_name.clone(),
            body: loop_value
                .body
                .iter()
                .map(story_statement)
                .collect::<Result<_>>()?,
        }),
        A::Repeat(loop_value) => Value::Repeat(pb::StoryRepeat {
            count: Some(repeat_count(&loop_value.count)),
            body: loop_value
                .body
                .iter()
                .map(story_statement)
                .collect::<Result<_>>()?,
        }),
        A::Break { .. } => Value::BreakLoop(()),
        A::Continue { .. } => Value::ContinueLoop(()),
    };
    Ok(pb::StoryStatement { value: Some(value) })
}

fn choice_entry(value: &ast::ChoiceEntry) -> Result<pb::ChoiceEntry> {
    use ast::ChoiceEntry as A;
    use pb::choice_entry::Value;
    let value = match value {
        A::Option(option) => Value::Option(pb::ChoiceOption {
            text: Some(template::compile(&option.text)?),
            target: option.target.clone(),
        }),
        A::If(branch) => Value::IfEntry(pb::ChoiceIf {
            condition: Some(expression(&branch.condition)?),
            body: Some(choice_list(&branch.body)?),
        }),
        A::Repeat(repeat) => Value::Repeat(pb::ChoiceRepeat {
            count: Some(repeat_count(&repeat.count)),
            body: Some(choice_list(&repeat.body)?),
        }),
        A::ForSnapshot(loop_value) => Value::ForSnapshot(pb::ChoiceForSnapshot {
            item_name: loop_value.item_name.clone(),
            array_name: loop_value.array_name.clone(),
            body: Some(choice_list(&loop_value.body)?),
        }),
    };
    Ok(pb::ChoiceEntry { value: Some(value) })
}

fn choice_list(values: &[ast::ChoiceEntry]) -> Result<pb::ChoiceEntryList> {
    Ok(pb::ChoiceEntryList {
        entries: values.iter().map(choice_entry).collect::<Result<_>>()?,
    })
}

fn repeat_count(value: &ast::RepeatCount) -> pb::RepeatCount {
    pb::RepeatCount {
        value: Some(match value {
            ast::RepeatCount::IntLiteral { value, .. } => pb::repeat_count::Value::Integer(*value),
            ast::RepeatCount::Variable { name, .. } => {
                pb::repeat_count::Value::Variable(name.clone())
            }
        }),
    }
}

fn variable_assignment(value: &ast::VarAssign) -> Result<pb::VariableAssignment> {
    Ok(pb::VariableAssignment {
        name: value.name.clone(),
        operator: match value.op {
            ast::AssignOp::Set => pb::AssignmentOperator::Set,
            ast::AssignOp::AddEq => pb::AssignmentOperator::Add,
            ast::AssignOp::SubEq => pb::AssignmentOperator::Subtract,
        }
        .into(),
        value: Some(expression(&value.value)?),
    })
}

fn expression(value: &ast::Expr) -> Result<pb::Expression> {
    use ast::Expr as A;
    use pb::expression::Value;
    let value = match value {
        A::IntLit(value) => Value::Integer(*value),
        A::DecimalLit(value) => Value::Decimal(pb::DecimalValue {
            canonical: value.to_string(),
        }),
        A::BoolLit(value) => Value::Boolean(*value),
        A::StringLit(value) => Value::StringValue(template::compile(value)?),
        A::VarRef { name, .. } => Value::Variable(pb::VariableReference { name: name.clone() }),
        A::BinOp { left, op, right } => Value::Binary(Box::new(pb::BinaryExpression {
            left: Some(Box::new(expression(left)?)),
            operator: binary_operator(op).into(),
            right: Some(Box::new(expression(right)?)),
        })),
        A::Call { name, args, .. } => Value::Call(call_expression(name, args)?),
        A::ListLit { items, .. } => Value::List(pb::ListExpression {
            items: items.iter().map(expression).collect::<Result<_>>()?,
        }),
    };
    Ok(pb::Expression { value: Some(value) })
}

fn call_expression(name: &str, args: &[ast::Expr]) -> Result<pb::CallExpression> {
    Ok(pb::CallExpression {
        name: name.to_string(),
        arguments: args.iter().map(expression).collect::<Result<_>>()?,
    })
}

fn binary_operator(value: &ast::BinOperator) -> pb::BinaryOperator {
    match value {
        ast::BinOperator::Add => pb::BinaryOperator::Add,
        ast::BinOperator::Sub => pb::BinaryOperator::Subtract,
        ast::BinOperator::Mul => pb::BinaryOperator::Multiply,
        ast::BinOperator::Div => pb::BinaryOperator::Divide,
        ast::BinOperator::Mod => pb::BinaryOperator::Modulo,
        ast::BinOperator::EqEq => pb::BinaryOperator::Equal,
        ast::BinOperator::NotEq => pb::BinaryOperator::NotEqual,
        ast::BinOperator::Lt => pb::BinaryOperator::Less,
        ast::BinOperator::LtEq => pb::BinaryOperator::LessEqual,
        ast::BinOperator::Gt => pb::BinaryOperator::Greater,
        ast::BinOperator::GtEq => pb::BinaryOperator::GreaterEqual,
    }
}
