use crate::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use crate::interpolation::ESCAPED_DOLLAR_MARKER;
use crate::token::{Spanned, Token};
use rust_decimal::Decimal;

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            diagnostics: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Spanned> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.token == Token::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(c) = self.peek() {
                if c.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }
            // Skip line comments
            if self.peek() == Some('/') && self.peek_next() == Some('/') {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }
            break;
        }
    }

    fn read_string(&mut self) -> Token {
        // Opening quote already consumed
        let mut s = String::new();
        loop {
            match self.advance() {
                Some('"') => break,
                Some('\\') => match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some('$') => s.push(ESCAPED_DOLLAR_MARKER),
                    Some(c) => s.push(c),
                    None => {
                        self.diagnostics.push(Diagnostic::new(
                            DiagnosticCode::ESyntax,
                            "Unterminated string literal",
                            Phase::Lex,
                            "GLOBAL",
                            self.line,
                            self.column,
                        ));
                        break;
                    }
                },
                Some(c) => s.push(c),
                None => {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Unterminated string literal",
                        Phase::Lex,
                        "GLOBAL",
                        self.line,
                        self.column,
                    ));
                    break;
                }
            }
        }
        Token::StringLit(s)
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            s.push('.');
            self.advance();

            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    s.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            return Token::DecimalLit(Decimal::from_str_exact(&s).unwrap());
        }

        Token::IntLit(s.parse().unwrap())
    }

    fn read_ident(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match s.as_str() {
            "INIT" => Token::Init,
            "if" => Token::If,
            "else" => Token::Else,
            "as" => Token::As,
            "integer" => Token::TypeInteger,
            "string" => Token::TypeString,
            "boolean" => Token::TypeBoolean,
            "decimal" => Token::TypeDecimal,
            "true" => Token::BoolLit(true),
            "false" => Token::BoolLit(false),
            "STOP" => Token::Stop,
            _ => Token::Ident(s),
        }
    }

    fn read_directive(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match s.as_str() {
            "actor" => Token::AtActor,
            "bg" => Token::AtBg,
            "bgm" => Token::AtBgm,
            "sfx" => Token::AtSfx,
            "choice" => Token::AtChoice,
            "jump" => Token::AtJump,
            "end" => Token::AtEnd,
            "start" => Token::AtStart,
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    format!("Unknown directive '@{}'", s),
                    Phase::Lex,
                    "GLOBAL",
                    self.line,
                    self.column,
                ));
                Token::Ident(format!("@{}", s))
            }
        }
    }

    fn read_hash_keyword(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match s.as_str() {
            "PREP" => Token::HashPrep,
            "STORY" => Token::HashStory,
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    format!("Unknown phase '#{}'", s),
                    Phase::Lex,
                    "GLOBAL",
                    self.line,
                    self.column,
                ));
                Token::Ident(format!("#{}", s))
            }
        }
    }

    fn next_token(&mut self) -> Spanned {
        self.skip_whitespace_and_comments();

        let line = self.line;
        let col = self.column;

        let ch = match self.advance() {
            Some(c) => c,
            None => return Spanned::new(Token::Eof, line, col),
        };

        let token = match ch {
            '*' => Token::Star,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '$' => Token::Dollar,
            '"' => self.read_string(),
            '#' => self.read_hash_keyword(),
            '@' => self.read_directive(),

            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::MinusEq
                } else {
                    Token::Minus
                }
            }
            '+' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::PlusEq
                } else {
                    Token::Plus
                }
            }
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::EqEq
                } else {
                    Token::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::NotEq
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Unexpected character '!'",
                        Phase::Lex,
                        "GLOBAL",
                        line,
                        col,
                    ));
                    return self.next_token();
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LtEq
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GtEq
                } else {
                    Token::Gt
                }
            }

            c if c.is_ascii_digit() => self.read_number(c),
            c if c.is_alphabetic() || c == '_' => self.read_ident(c),

            other => {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    format!("Unexpected character '{}'", other),
                    Phase::Lex,
                    "GLOBAL",
                    line,
                    col,
                ));
                return self.next_token();
            }
        };

        Spanned::new(token, line, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("* INIT { }");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token, Token::Star);
        assert_eq!(tokens[1].token, Token::Init);
        assert_eq!(tokens[2].token, Token::LBrace);
        assert_eq!(tokens[3].token, Token::RBrace);
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token, Token::StringLit("hello world".to_string()));
    }

    #[test]
    fn test_directives() {
        let mut lexer = Lexer::new("@actor @bg @bgm @sfx @choice @jump @end @start");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token, Token::AtActor);
        assert_eq!(tokens[1].token, Token::AtBg);
        assert_eq!(tokens[2].token, Token::AtBgm);
        assert_eq!(tokens[3].token, Token::AtSfx);
        assert_eq!(tokens[4].token, Token::AtChoice);
        assert_eq!(tokens[5].token, Token::AtJump);
        assert_eq!(tokens[6].token, Token::AtEnd);
        assert_eq!(tokens[7].token, Token::AtStart);
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("= == != < <= > >= + - / % += -= [ ]");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token, Token::Eq);
        assert_eq!(tokens[1].token, Token::EqEq);
        assert_eq!(tokens[2].token, Token::NotEq);
        assert_eq!(tokens[3].token, Token::Lt);
        assert_eq!(tokens[4].token, Token::LtEq);
        assert_eq!(tokens[5].token, Token::Gt);
        assert_eq!(tokens[6].token, Token::GtEq);
        assert_eq!(tokens[7].token, Token::Plus);
        assert_eq!(tokens[8].token, Token::Minus);
        assert_eq!(tokens[9].token, Token::Slash);
        assert_eq!(tokens[10].token, Token::Percent);
        assert_eq!(tokens[11].token, Token::PlusEq);
        assert_eq!(tokens[12].token, Token::MinusEq);
        assert_eq!(tokens[13].token, Token::LBracket);
        assert_eq!(tokens[14].token, Token::RBracket);
    }

    #[test]
    fn test_decimal_literal() {
        let mut lexer = Lexer::new("42 3.14");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token, Token::IntLit(42));
        assert_eq!(
            tokens[1].token,
            Token::DecimalLit(Decimal::from_str_exact("3.14").unwrap())
        );
    }

    #[test]
    fn test_type_keywords() {
        let mut lexer = Lexer::new("as integer string boolean decimal");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token, Token::As);
        assert_eq!(tokens[1].token, Token::TypeInteger);
        assert_eq!(tokens[2].token, Token::TypeString);
        assert_eq!(tokens[3].token, Token::TypeBoolean);
        assert_eq!(tokens[4].token, Token::TypeDecimal);
    }

    #[test]
    fn test_comments_skipped() {
        let mut lexer = Lexer::new("// this is a comment\n* INIT");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].token, Token::Star);
        assert_eq!(tokens[1].token, Token::Init);
    }
}
