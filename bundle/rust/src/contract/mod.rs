use rust_decimal::Decimal;
use std::collections::HashSet;

use crate::limits::ResourceLimits;
use crate::proto::storybundle::v1 as pb;
use crate::{BundleError, FORMAT_VERSION, Result};

pub fn validate_story(story: &pb::CompiledStory) -> Result<()> {
    if story.format_version != FORMAT_VERSION {
        return contract_error("format_version must be 1");
    }
    let project = story
        .project
        .as_ref()
        .ok_or_else(|| BundleError::Contract("project is required".to_string()))?;
    if project.id.is_empty() || project.name.is_empty() || project.version.is_empty() {
        return contract_error("project id, name, and version are required");
    }
    let initialization = story
        .initialization
        .as_ref()
        .ok_or_else(|| BundleError::Contract("initialization is required".to_string()))?;
    if initialization.start_scene.is_empty() {
        return contract_error("initialization.start_scene is required");
    }
    for variable in &initialization.variables {
        validate_variable(variable, 1)?;
    }
    for actor in &initialization.actors {
        if actor.id.is_empty() {
            return contract_error("actor id is required");
        }
        validate_interpolated(actor.display_name.as_ref(), 1)?;
        for portrait in &actor.portraits {
            if portrait.emotion.is_empty() {
                return contract_error("portrait emotion is required");
            }
            validate_interpolated(portrait.asset_path.as_ref(), 1)?;
        }
    }
    for logic in &story.logic_blocks {
        if logic.name.is_empty() {
            return contract_error("logic block name is required");
        }
        for parameter in &logic.parameters {
            validate_variable_type(parameter.r#type)?;
        }
        if let Some(return_type) = logic.return_type {
            validate_variable_type(return_type)?;
        }
        for statement in &logic.body {
            validate_prep(statement, 1)?;
        }
    }
    let mut scene_labels = HashSet::new();
    for scene in &story.scenes {
        if scene.label.is_empty() {
            return contract_error("scene label is required");
        }
        if !scene_labels.insert(scene.label.as_str()) {
            return contract_error("scene labels must be unique");
        }
        if let Some(prep) = &scene.prep {
            for statement in &prep.statements {
                validate_prep(statement, 1)?;
            }
        }
        let story_block = scene
            .story
            .as_ref()
            .ok_or_else(|| BundleError::Contract("scene story block is required".to_string()))?;
        for statement in &story_block.statements {
            validate_story_statement(statement, 1)?;
        }
    }
    if !scene_labels.contains(initialization.start_scene.as_str()) {
        return contract_error("initialization.start_scene must identify a scene");
    }
    Ok(())
}

fn validate_variable(variable: &pb::VariableDefinition, depth: usize) -> Result<()> {
    if variable.name.is_empty() {
        return contract_error("variable name is required");
    }
    validate_variable_type(variable.r#type)?;
    validate_expression(variable.value.as_ref(), depth)
}

fn validate_variable_type(value: i32) -> Result<()> {
    match pb::VariableType::try_from(value) {
        Ok(pb::VariableType::Unspecified) | Err(_) => {
            contract_error("variable type must be recognized and specified")
        }
        Ok(_) => Ok(()),
    }
}

fn validate_interpolated(value: Option<&pb::InterpolatedString>, depth: usize) -> Result<()> {
    check_depth(depth)?;
    let value = value
        .ok_or_else(|| BundleError::Contract("interpolated string is required".to_string()))?;
    for segment in &value.segments {
        if segment.value.is_none() {
            return contract_error("string segment value is required");
        }
    }
    Ok(())
}

fn validate_expression(value: Option<&pb::Expression>, depth: usize) -> Result<()> {
    check_depth(depth)?;
    use pb::expression::Value;
    match value.and_then(|expression| expression.value.as_ref()) {
        Some(Value::Integer(_)) | Some(Value::Boolean(_)) => Ok(()),
        Some(Value::Decimal(decimal)) => validate_decimal(&decimal.canonical),
        Some(Value::StringValue(string)) => validate_interpolated(Some(string), depth + 1),
        Some(Value::Variable(variable)) if !variable.name.is_empty() => Ok(()),
        Some(Value::Binary(binary)) => {
            if matches!(
                pb::BinaryOperator::try_from(binary.operator),
                Ok(pb::BinaryOperator::Unspecified) | Err(_)
            ) {
                return contract_error("binary operator must be recognized and specified");
            }
            validate_expression(binary.left.as_deref(), depth + 1)?;
            validate_expression(binary.right.as_deref(), depth + 1)
        }
        Some(Value::Call(call)) => {
            if call.name.is_empty() {
                return contract_error("call name is required");
            }
            for argument in &call.arguments {
                validate_expression(Some(argument), depth + 1)?;
            }
            Ok(())
        }
        Some(Value::List(list)) => {
            for item in &list.items {
                validate_expression(Some(item), depth + 1)?;
            }
            Ok(())
        }
        Some(Value::Variable(_)) => contract_error("variable reference name is required"),
        None => contract_error("expression value is required"),
    }
}

fn validate_decimal(value: &str) -> Result<()> {
    if value.is_empty() || value.contains(['e', 'E', '+']) {
        return contract_error("decimal must be a non-exponent canonical string");
    }
    let parsed = Decimal::from_str_exact(value)
        .map_err(|_| BundleError::Contract("decimal is not exactly representable".to_string()))?;
    if parsed.to_string() != value {
        return contract_error("decimal string is not canonical or does not preserve scale");
    }
    Ok(())
}

fn validate_prep(statement: &pb::PrepStatement, depth: usize) -> Result<()> {
    check_depth(depth)?;
    use pb::prep_statement::Value;
    match statement.value.as_ref() {
        Some(Value::Background(value)) => {
            validate_interpolated(value.asset_path.as_ref(), depth + 1)
        }
        Some(Value::Bgm(value)) => match value.value.as_ref() {
            Some(pb::bgm_directive::Value::AssetPath(path)) => {
                validate_interpolated(Some(path), depth + 1)
            }
            Some(pb::bgm_directive::Value::Stop(_)) => Ok(()),
            None => contract_error("BGM value is required"),
        },
        Some(Value::Sfx(value)) => validate_interpolated(value.asset_path.as_ref(), depth + 1),
        Some(Value::VariableDefinition(value)) => validate_variable(value, depth + 1),
        Some(Value::VariableAssignment(value)) => {
            if value.name.is_empty()
                || matches!(
                    pb::AssignmentOperator::try_from(value.operator),
                    Ok(pb::AssignmentOperator::Unspecified) | Err(_)
                )
            {
                return contract_error("assignment name and operator are required");
            }
            validate_expression(value.value.as_ref(), depth + 1)
        }
        Some(Value::Call(call)) => validate_call(call, depth + 1),
        Some(Value::IfElse(value)) => {
            validate_expression(value.condition.as_ref(), depth + 1)?;
            let then_branch = value
                .then_branch
                .as_ref()
                .ok_or_else(|| BundleError::Contract("PREP then branch is required".to_string()))?;
            for child in &then_branch.statements {
                validate_prep(child, depth + 1)?;
            }
            if let Some(else_branch) = &value.else_branch {
                for child in &else_branch.statements {
                    validate_prep(child, depth + 1)?;
                }
            }
            Ok(())
        }
        Some(Value::ForSnapshot(value)) => {
            if value.item_name.is_empty() || value.array_name.is_empty() {
                return contract_error("PREP snapshot loop names are required");
            }
            for child in &value.body {
                validate_prep(child, depth + 1)?;
            }
            Ok(())
        }
        Some(Value::Repeat(value)) => {
            validate_repeat(value.count.as_ref())?;
            for child in &value.body {
                validate_prep(child, depth + 1)?;
            }
            Ok(())
        }
        Some(Value::BreakLoop(_)) | Some(Value::ContinueLoop(_)) => Ok(()),
        Some(Value::ReturnStatement(value)) => {
            if let Some(expression) = &value.value {
                validate_expression(Some(expression), depth + 1)?;
            }
            Ok(())
        }
        None => contract_error("PREP statement value is required"),
    }
}

fn validate_story_statement(statement: &pb::StoryStatement, depth: usize) -> Result<()> {
    check_depth(depth)?;
    use pb::story_statement::Value;
    match statement.value.as_ref() {
        Some(Value::Narration(value)) => validate_interpolated(value.text.as_ref(), depth + 1),
        Some(Value::VariableOutput(value)) if !value.name.is_empty() => Ok(()),
        Some(Value::Dialogue(value)) => {
            if value.actor_id.is_empty() || value.form.is_none() {
                return contract_error("dialogue actor and form are required");
            }
            if let Some(pb::dialogue::Form::Portrait(portrait)) = &value.form
                && (portrait.emotion.is_empty()
                    || matches!(
                        pb::Position::try_from(portrait.position),
                        Ok(pb::Position::Unspecified) | Err(_)
                    ))
            {
                return contract_error("portrait dialogue emotion and position are required");
            }
            validate_interpolated(value.text.as_ref(), depth + 1)
        }
        Some(Value::IfElse(value)) => {
            validate_expression(value.condition.as_ref(), depth + 1)?;
            let then_branch = value.then_branch.as_ref().ok_or_else(|| {
                BundleError::Contract("STORY then branch is required".to_string())
            })?;
            for child in &then_branch.statements {
                validate_story_statement(child, depth + 1)?;
            }
            if let Some(else_branch) = &value.else_branch {
                for child in &else_branch.statements {
                    validate_story_statement(child, depth + 1)?;
                }
            }
            Ok(())
        }
        Some(Value::Choice(value)) => {
            for entry in &value.entries {
                validate_choice(entry, depth + 1)?;
            }
            Ok(())
        }
        Some(Value::Jump(value)) if !value.target.is_empty() => Ok(()),
        Some(Value::End(_)) | Some(Value::BreakLoop(_)) | Some(Value::ContinueLoop(_)) => Ok(()),
        Some(Value::Sfx(value)) => validate_interpolated(value.asset_path.as_ref(), depth + 1),
        Some(Value::ForSnapshot(value)) => {
            if value.item_name.is_empty() || value.array_name.is_empty() {
                return contract_error("STORY snapshot loop names are required");
            }
            for child in &value.body {
                validate_story_statement(child, depth + 1)?;
            }
            Ok(())
        }
        Some(Value::Repeat(value)) => {
            validate_repeat(value.count.as_ref())?;
            for child in &value.body {
                validate_story_statement(child, depth + 1)?;
            }
            Ok(())
        }
        Some(Value::VariableOutput(_)) => contract_error("variable output name is required"),
        Some(Value::Jump(_)) => contract_error("jump target is required"),
        None => contract_error("STORY statement value is required"),
    }
}

fn validate_choice(entry: &pb::ChoiceEntry, depth: usize) -> Result<()> {
    check_depth(depth)?;
    use pb::choice_entry::Value;
    match entry.value.as_ref() {
        Some(Value::Option(value)) => {
            if value.target.is_empty() {
                return contract_error("choice target is required");
            }
            validate_interpolated(value.text.as_ref(), depth + 1)
        }
        Some(Value::IfEntry(value)) => {
            validate_expression(value.condition.as_ref(), depth + 1)?;
            validate_choice_list(value.body.as_ref(), depth + 1)
        }
        Some(Value::Repeat(value)) => {
            validate_repeat(value.count.as_ref())?;
            validate_choice_list(value.body.as_ref(), depth + 1)
        }
        Some(Value::ForSnapshot(value)) => {
            if value.item_name.is_empty() || value.array_name.is_empty() {
                return contract_error("choice snapshot loop names are required");
            }
            validate_choice_list(value.body.as_ref(), depth + 1)
        }
        None => contract_error("choice entry value is required"),
    }
}

fn validate_choice_list(value: Option<&pb::ChoiceEntryList>, depth: usize) -> Result<()> {
    let value =
        value.ok_or_else(|| BundleError::Contract("choice entry body is required".to_string()))?;
    for entry in &value.entries {
        validate_choice(entry, depth + 1)?;
    }
    Ok(())
}

fn validate_call(call: &pb::CallExpression, depth: usize) -> Result<()> {
    if call.name.is_empty() {
        return contract_error("call name is required");
    }
    for argument in &call.arguments {
        validate_expression(Some(argument), depth + 1)?;
    }
    Ok(())
}

fn validate_repeat(value: Option<&pb::RepeatCount>) -> Result<()> {
    match value.and_then(|count| count.value.as_ref()) {
        Some(pb::repeat_count::Value::Integer(_)) => Ok(()),
        Some(pb::repeat_count::Value::Variable(name)) if !name.is_empty() => Ok(()),
        _ => contract_error("repeat count is required"),
    }
}

fn check_depth(depth: usize) -> Result<()> {
    if depth > ResourceLimits::HARD.max_semantic_depth {
        return Err(BundleError::Limit(format!(
            "semantic nesting exceeds {}",
            ResourceLimits::HARD.max_semantic_depth
        )));
    }
    Ok(())
}

fn contract_error<T>(message: &str) -> Result<T> {
    Err(BundleError::Contract(message.to_string()))
}
