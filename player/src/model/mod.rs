//! Parser-independent immutable execution model.
//!
//! Source adapters retain locations for diagnostics. Other adapters use
//! `SourceSpan::UNKNOWN`; spans never participate in semantic identity.

use rust_decimal::Decimal;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Debug for SourceSpan {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Semantic model debug output is used as the v1 canonical identity.
        // Deliberately omit source coordinates.
        formatter.write_str("SourceSpan")
    }
}

impl SourceSpan {
    pub const UNKNOWN: Self = Self { line: 0, column: 0 };
}

#[derive(Debug, Clone)]
pub struct StoryModel {
    pub init: InitBlock,
    pub logic_blocks: Vec<LogicBlock>,
    pub scenes: Vec<Scene>,
}

#[derive(Debug, Clone)]
pub struct InitBlock {
    pub variables: Vec<VarDecl>,
    pub actors: Vec<ActorDecl>,
    pub start: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct LogicBlock {
    pub name: String,
    pub params: Vec<LogicParam>,
    pub return_type: Option<VarType>,
    pub body: Vec<PrepStatement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct LogicParam {
    pub name: String,
    pub var_type: VarType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub var_type: VarType,
    pub value: Expr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VarType {
    Integer,
    String,
    Boolean,
    Decimal,
    ArrayInteger,
    ArrayString,
    ArrayBoolean,
    ArrayDecimal,
}

#[derive(Debug, Clone)]
pub struct ActorDecl {
    pub id: String,
    pub display_name: String,
    pub portraits: Vec<PortraitEntry>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct PortraitEntry {
    pub emotion: String,
    pub path: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub label: String,
    pub prep: Vec<PrepStatement>,
    pub story: Vec<StoryStatement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum PrepStatement {
    BgDirective {
        path: String,
        span: SourceSpan,
    },
    BgmDirective {
        value: BgmValue,
        span: SourceSpan,
    },
    SfxDirective {
        path: String,
        span: SourceSpan,
    },
    VarDecl(VarDecl),
    VarAssign(VarAssign),
    Call {
        name: String,
        args: Vec<Expr>,
        span: SourceSpan,
    },
    IfElse(PrepIfElse),
    ForSnapshot(PrepForSnapshot),
    Repeat(PrepRepeat),
    Break {
        span: SourceSpan,
    },
    Continue {
        span: SourceSpan,
    },
    Return {
        value: Option<Expr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone)]
pub enum BgmValue {
    Path(String),
    Stop,
}

#[derive(Debug, Clone)]
pub struct VarAssign {
    pub name: String,
    pub op: AssignOp,
    pub value: Expr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Set,
    AddEq,
    SubEq,
}

#[derive(Debug, Clone)]
pub struct PrepIfElse {
    pub condition: Expr,
    pub then_branch: Vec<PrepStatement>,
    pub else_branch: Option<Vec<PrepStatement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct PrepForSnapshot {
    pub item_name: String,
    pub array_name: String,
    pub body: Vec<PrepStatement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct PrepRepeat {
    pub count: RepeatCount,
    pub body: Vec<PrepStatement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum StoryStatement {
    Narration { text: String, span: SourceSpan },
    VarOutput { name: String, span: SourceSpan },
    Dialogue(Dialogue),
    IfElse(StoryIfElse),
    Choice(ChoiceBlock),
    Jump { target: String, span: SourceSpan },
    End { span: SourceSpan },
    SfxDirective { path: String, span: SourceSpan },
    ForSnapshot(StoryForSnapshot),
    Repeat(StoryRepeat),
    Break { span: SourceSpan },
    Continue { span: SourceSpan },
}

#[derive(Debug, Clone)]
pub struct Dialogue {
    pub actor_id: String,
    pub form: DialogueForm,
    pub text: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum DialogueForm {
    NameOnly,
    Portrait { emotion: String, position: Position },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct StoryIfElse {
    pub condition: Expr,
    pub then_branch: Vec<StoryStatement>,
    pub else_branch: Option<Vec<StoryStatement>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ChoiceBlock {
    pub entries: Vec<ChoiceEntry>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum ChoiceEntry {
    Option(ChoiceOption),
    If(ChoiceIfEntry),
    Repeat(ChoiceRepeatEntry),
    ForSnapshot(ChoiceForSnapshotEntry),
}

#[derive(Debug, Clone)]
pub struct ChoiceOption {
    pub text: String,
    pub target: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ChoiceIfEntry {
    pub condition: Expr,
    pub body: Vec<ChoiceEntry>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ChoiceRepeatEntry {
    pub count: RepeatCount,
    pub body: Vec<ChoiceEntry>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ChoiceForSnapshotEntry {
    pub item_name: String,
    pub array_name: String,
    pub body: Vec<ChoiceEntry>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct StoryForSnapshot {
    pub item_name: String,
    pub array_name: String,
    pub body: Vec<StoryStatement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct StoryRepeat {
    pub count: RepeatCount,
    pub body: Vec<StoryStatement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum RepeatCount {
    IntLiteral { value: i64, span: SourceSpan },
    Variable { name: String, span: SourceSpan },
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    DecimalLit(Decimal),
    BoolLit(bool),
    StringLit(String),
    VarRef {
        name: String,
        span: SourceSpan,
    },
    BinOp {
        left: Box<Expr>,
        op: BinOperator,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        span: SourceSpan,
    },
    ListLit {
        items: Vec<Expr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}
