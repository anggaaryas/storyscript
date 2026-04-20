use rust_decimal::Decimal;

/// Token types for the StoryScript lexer.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Structural
    Star,      // *
    LBrace,    // {
    RBrace,    // }
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    Semicolon, // ;
    Colon,     // :
    Arrow,     // ->
    Comma,     // ,
    Dollar,    // $

    // Literals
    StringLit(String), // "..."
    IntLit(i64),
    DecimalLit(Decimal),
    BoolLit(bool),

    // Identifiers & keywords
    Ident(String),
    Init,      // INIT
    Require,   // REQUIRE
    HashPrep,  // #PREP
    HashStory, // #STORY
    Logic,
    Return,

    // Directives
    AtActor,  // @actor
    AtBg,     // @bg
    AtBgm,    // @bgm
    AtSfx,    // @sfx
    AtChoice, // @choice
    AtJump,   // @jump
    AtEnd,    // @end
    AtStart,  // @start
    AtInclude, // @include

    // Keywords
    If,
    Else,
    For,
    Repeat,
    In,
    Snapshot,
    Break,
    Continue,
    As,
    TypeInteger,
    TypeString,
    TypeBoolean,
    TypeDecimal,
    TypeArray,
    Stop, // STOP (for @bgm STOP)

    // Operators
    Eq,      // =
    EqEq,    // ==
    NotEq,   // !=
    Lt,      // <
    LtEq,    // <=
    Gt,      // >
    GtEq,    // >=
    Plus,    // +
    Minus,   // -
    Slash,   // /
    Percent, // %
    PlusEq,  // +=
    MinusEq, // -=

    // Special
    Eof,
}

impl Token {
    pub fn name(&self) -> &str {
        match self {
            Token::Star => "'*'",
            Token::LBrace => "'{'",
            Token::RBrace => "'}'",
            Token::LParen => "'('",
            Token::RParen => "')'",
            Token::LBracket => "'['",
            Token::RBracket => "']'",
            Token::Semicolon => "';'",
            Token::Colon => "':'",
            Token::Arrow => "'->'",
            Token::Comma => "','",
            Token::Dollar => "'$'",
            Token::StringLit(_) => "string",
            Token::IntLit(_) => "integer",
            Token::DecimalLit(_) => "decimal",
            Token::BoolLit(_) => "boolean",
            Token::Ident(_) => "identifier",
            Token::Init => "'INIT'",
            Token::Require => "'REQUIRE'",
            Token::HashPrep => "'#PREP'",
            Token::HashStory => "'#STORY'",
            Token::Logic => "'logic'",
            Token::Return => "'return'",
            Token::AtActor => "'@actor'",
            Token::AtBg => "'@bg'",
            Token::AtBgm => "'@bgm'",
            Token::AtSfx => "'@sfx'",
            Token::AtChoice => "'@choice'",
            Token::AtJump => "'@jump'",
            Token::AtEnd => "'@end'",
            Token::AtStart => "'@start'",
            Token::AtInclude => "'@include'",
            Token::If => "'if'",
            Token::Else => "'else'",
            Token::For => "'for'",
            Token::Repeat => "'repeat'",
            Token::In => "'in'",
            Token::Snapshot => "'snapshot'",
            Token::Break => "'break'",
            Token::Continue => "'continue'",
            Token::As => "'as'",
            Token::TypeInteger => "'integer'",
            Token::TypeString => "'string'",
            Token::TypeBoolean => "'boolean'",
            Token::TypeDecimal => "'decimal'",
            Token::TypeArray => "'array'",
            Token::Stop => "'STOP'",
            Token::Eq => "'='",
            Token::EqEq => "'=='",
            Token::NotEq => "'!='",
            Token::Lt => "'<'",
            Token::LtEq => "'<='",
            Token::Gt => "'>'",
            Token::GtEq => "'>='",
            Token::Plus => "'+'",
            Token::Minus => "'-'",
            Token::Slash => "'/'",
            Token::Percent => "'%'",
            Token::PlusEq => "'+='",
            Token::MinusEq => "'-='",
            Token::Eof => "EOF",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

impl Spanned {
    pub fn new(token: Token, line: usize, column: usize) -> Self {
        Self {
            token,
            line,
            column,
        }
    }
}
