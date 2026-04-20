use rust_decimal::Decimal;

/// Abstract Syntax Tree types for StoryScript.

#[derive(Debug, Clone)]
pub struct Script {
    pub init: InitBlock,
    pub logic_blocks: Vec<LogicBlock>,
    pub scenes: Vec<Scene>,
}

// ---------------------------------------------------------------------------
// INIT block
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct InitBlock {
    pub variables: Vec<VarDecl>,
    pub actors: Vec<ActorDecl>,
    pub includes: Vec<IncludeDirective>,
    pub start: StartDirective,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct IncludeDirective {
    pub path: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ChildModule {
    pub require: RequireBlock,
    pub logic_blocks: Vec<LogicBlock>,
    pub scenes: Vec<Scene>,
}

#[derive(Debug, Clone)]
pub struct LogicBlock {
    pub name: String,
    pub params: Vec<LogicParam>,
    pub return_type: Option<VarType>,
    pub body: Vec<PrepStatement>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct LogicParam {
    pub name: String,
    pub var_type: VarType,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct RequireBlock {
    pub variables: Vec<RequireVarDecl>,
    pub actors: Vec<RequireActorRef>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct RequireVarDecl {
    pub name: String,
    pub var_type: VarType,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct RequireActorRef {
    pub id: String,
    pub emotions: Vec<RequireEmotionRef>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct RequireEmotionRef {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub var_type: VarType,
    pub value: Expr,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct PortraitEntry {
    pub emotion: String,
    pub path: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct StartDirective {
    pub target: String,
    pub line: usize,
    pub column: usize,
}

// ---------------------------------------------------------------------------
// Scenes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Scene {
    pub label: String,
    pub prep: Option<PrepBlock>,
    pub story: StoryBlock,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct PrepBlock {
    pub statements: Vec<PrepStatement>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum PrepStatement {
    BgDirective {
        path: String,
        line: usize,
        column: usize,
    },
    BgmDirective {
        value: BgmValue,
        line: usize,
        column: usize,
    },
    SfxDirective {
        path: String,
        line: usize,
        column: usize,
    },
    VarDecl(VarDecl),
    VarAssign(VarAssign),
    Call {
        name: String,
        args: Vec<Expr>,
        line: usize,
        column: usize,
    },
    IfElse(PrepIfElse),
    ForSnapshot(PrepForSnapshot),
    Repeat(PrepRepeat),
    Break {
        line: usize,
        column: usize,
    },
    Continue {
        line: usize,
        column: usize,
    },
    Return {
        value: Option<Expr>,
        line: usize,
        column: usize,
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
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Set,   // =
    AddEq, // +=
    SubEq, // -=
}

#[derive(Debug, Clone)]
pub struct PrepIfElse {
    pub condition: Expr,
    pub then_branch: Vec<PrepStatement>,
    pub else_branch: Option<Vec<PrepStatement>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct PrepForSnapshot {
    pub item_name: String,
    pub array_name: String,
    pub body: Vec<PrepStatement>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct PrepRepeat {
    pub count: RepeatCount,
    pub body: Vec<PrepStatement>,
    pub line: usize,
    pub column: usize,
}

// ---------------------------------------------------------------------------
// #STORY
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StoryBlock {
    pub statements: Vec<StoryStatement>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum StoryStatement {
    Narration {
        text: String,
        line: usize,
        column: usize,
    },
    VarOutput {
        name: String,
        line: usize,
        column: usize,
    },
    Dialogue(Dialogue),
    IfElse(StoryIfElse),
    Choice(ChoiceBlock),
    Jump {
        target: String,
        line: usize,
        column: usize,
    },
    End {
        line: usize,
        column: usize,
    },
    SfxDirective {
        path: String,
        line: usize,
        column: usize,
    },
    ForSnapshot(StoryForSnapshot),
    Repeat(StoryRepeat),
    Break {
        line: usize,
        column: usize,
    },
    Continue {
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Dialogue {
    pub actor_id: String,
    pub form: DialogueForm,
    pub text: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum DialogueForm {
    NameOnly,
    Portrait { emotion: String, position: Position },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    Left,
    Center,
    Right,
}

impl Position {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Left" | "L" => Some(Position::Left),
            "Center" | "C" => Some(Position::Center),
            "Right" | "R" => Some(Position::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoryIfElse {
    pub condition: Expr,
    pub then_branch: Vec<StoryStatement>,
    pub else_branch: Option<Vec<StoryStatement>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ChoiceBlock {
    pub entries: Vec<ChoiceEntry>,
    pub line: usize,
    pub column: usize,
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
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ChoiceIfEntry {
    pub condition: Expr,
    pub body: Vec<ChoiceEntry>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ChoiceRepeatEntry {
    pub count: RepeatCount,
    pub body: Vec<ChoiceEntry>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ChoiceForSnapshotEntry {
    pub item_name: String,
    pub array_name: String,
    pub body: Vec<ChoiceEntry>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct StoryForSnapshot {
    pub item_name: String,
    pub array_name: String,
    pub body: Vec<StoryStatement>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct StoryRepeat {
    pub count: RepeatCount,
    pub body: Vec<StoryStatement>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum RepeatCount {
    IntLiteral {
        value: i64,
        line: usize,
        column: usize,
    },
    Variable {
        name: String,
        line: usize,
        column: usize,
    },
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    DecimalLit(Decimal),
    BoolLit(bool),
    StringLit(String),
    VarRef {
        name: String,
        line: usize,
        column: usize,
    },
    BinOp {
        left: Box<Expr>,
        op: BinOperator,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        line: usize,
        column: usize,
    },
    ListLit {
        items: Vec<Expr>,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
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
