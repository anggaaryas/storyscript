use storyscript_parser::interpolation::{ESCAPED_DOLLAR_MARKER, scan_placeholders};

use crate::proto::storybundle::v1 as pb;
use crate::{BundleError, Result};

pub fn compile(input: &str) -> Result<pb::InterpolatedString> {
    scan_placeholders(input).map_err(|error| {
        BundleError::Contract(format!("invalid interpolation: {}", error.message))
    })?;

    let chars = input.chars().collect::<Vec<_>>();
    let mut segments = Vec::new();
    let mut literal = String::new();
    let mut index = 0;
    while index < chars.len() {
        match chars[index] {
            ESCAPED_DOLLAR_MARKER => {
                literal.push('$');
                index += 1;
            }
            '$' if chars.get(index + 1) == Some(&'{') => {
                if !literal.is_empty() {
                    segments.push(literal_segment(std::mem::take(&mut literal)));
                }
                let end = chars[index + 2..]
                    .iter()
                    .position(|character| *character == '}')
                    .map(|offset| index + 2 + offset)
                    .ok_or_else(|| {
                        BundleError::Contract("unterminated interpolation".to_string())
                    })?;
                let name = chars[index + 2..end].iter().collect::<String>();
                segments.push(pb::StringSegment {
                    value: Some(pb::string_segment::Value::Variable(name)),
                });
                index = end + 1;
            }
            character => {
                literal.push(character);
                index += 1;
            }
        }
    }
    if !literal.is_empty() {
        segments.push(literal_segment(literal));
    }

    Ok(pb::InterpolatedString { segments })
}

fn literal_segment(value: String) -> pb::StringSegment {
    pb::StringSegment {
        value: Some(pb::string_segment::Value::Literal(value)),
    }
}
