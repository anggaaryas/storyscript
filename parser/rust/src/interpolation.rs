/// Internal marker inserted by the lexer for source-level `\$` escapes.
/// This lets interpolation treat escaped dollar signs as literal `$`.
pub const ESCAPED_DOLLAR_MARKER: char = '\u{E000}';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placeholder {
    pub name: String,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpolationError {
    pub message: String,
    pub offset: usize,
}

impl InterpolationError {
    fn new(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }
}

pub fn scan_placeholders(input: &str) -> Result<Vec<Placeholder>, InterpolationError> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch == ESCAPED_DOLLAR_MARKER {
            i += 1;
            continue;
        }

        if ch == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
            let (name, end_idx) = parse_placeholder(&chars, i)?;
            out.push(Placeholder { name, offset: i });
            i = end_idx + 1;
            continue;
        }

        i += 1;
    }

    Ok(out)
}

pub fn render_interpolated<F>(input: &str, mut resolve: F) -> Result<String, InterpolationError>
where
    F: FnMut(&str) -> Option<String>,
{
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == ESCAPED_DOLLAR_MARKER {
            out.push('$');
            i += 1;
            continue;
        }

        if ch == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
            let (name, end_idx) = parse_placeholder(&chars, i)?;
            match resolve(&name) {
                Some(value) => out.push_str(&value),
                None => {
                    return Err(InterpolationError::new(
                        format!("Interpolation variable '${{{}}}' is not declared", name),
                        i,
                    ));
                }
            }
            i = end_idx + 1;
            continue;
        }

        out.push(ch);
        i += 1;
    }

    Ok(out)
}

fn parse_placeholder(
    chars: &[char],
    start_idx: usize,
) -> Result<(String, usize), InterpolationError> {
    let mut i = start_idx + 2; // skip `${`
    if i >= chars.len() {
        return Err(InterpolationError::new(
            "Unterminated interpolation placeholder, expected '}'",
            start_idx,
        ));
    }

    if chars[i] == '}' {
        return Err(InterpolationError::new(
            "Empty interpolation placeholder '${}' is not allowed",
            i,
        ));
    }

    if !is_ident_start(chars[i]) {
        return Err(InterpolationError::new(
            format!(
                "Invalid interpolation variable name start '{}'; expected [A-Za-z_]",
                chars[i]
            ),
            i,
        ));
    }

    let mut name = String::new();
    name.push(chars[i]);
    i += 1;

    while i < chars.len() && is_ident_continue(chars[i]) {
        name.push(chars[i]);
        i += 1;
    }

    if i >= chars.len() {
        return Err(InterpolationError::new(
            "Unterminated interpolation placeholder, expected '}'",
            start_idx,
        ));
    }

    if chars[i] != '}' {
        return Err(InterpolationError::new(
            format!(
                "Invalid character '{}' in interpolation variable name",
                chars[i]
            ),
            i,
        ));
    }

    Ok((name, i))
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_placeholders() {
        let placeholders = scan_placeholders("HP: ${hp}, Name: ${hero_name}").unwrap();
        assert_eq!(placeholders.len(), 2);
        assert_eq!(placeholders[0].name, "hp");
        assert_eq!(placeholders[1].name, "hero_name");
    }

    #[test]
    fn rejects_malformed_placeholder() {
        let err = scan_placeholders("Bad: ${1x}").unwrap_err();
        assert!(
            err.message
                .contains("Invalid interpolation variable name start")
        );
    }

    #[test]
    fn keeps_escaped_dollar_literal() {
        let src = format!("cost={}{{amount}}", ESCAPED_DOLLAR_MARKER);
        let rendered = render_interpolated(&src, |_| None).unwrap();
        assert_eq!(rendered, "cost=${amount}");
    }
}
