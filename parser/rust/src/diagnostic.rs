use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticCode {
    // Compile-time errors
    ESyntax,
    EInitCount,
    EInitOrder,
    EStartCount,
    ESceneDuplicate,
    EActorDuplicate,
    EEmotionDuplicate,
    EGlobalDuplicate,
    EStartTargetMissing,
    EJumpTargetMissing,
    EChoiceTargetMissing,
    ESceneStructure,
    EPhaseTokenForbidden,
    EActorUnknown,
    EDialogueShapeInvalid,
    EPositionInvalid,
    EEmotionUnknown,
    EPortraitModeInvalid,
    EVariableUndeclaredRead,
    EVariableUndeclaredWrite,
    EVariableTypeMismatch,
    EVariableCompoundAssignInvalid,
    EExpressionTypeInvalid,
    EConditionTypeInvalid,
    EChoiceStaticEmpty,
    EStoryUnterminatedPath,

    // Warnings
    WChoicePossiblyEmpty,

    // Runtime
    RChoiceExhausted,
    RAssetNotFound,
    RAssetLoadFailed,
    RAudioDeviceFailure,
    RSaveStateCorrupt,
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ESyntax => "E_SYNTAX",
            Self::EInitCount => "E_INIT_COUNT",
            Self::EInitOrder => "E_INIT_ORDER",
            Self::EStartCount => "E_START_COUNT",
            Self::ESceneDuplicate => "E_SCENE_DUPLICATE",
            Self::EActorDuplicate => "E_ACTOR_DUPLICATE",
            Self::EEmotionDuplicate => "E_EMOTION_DUPLICATE",
            Self::EGlobalDuplicate => "E_GLOBAL_DUPLICATE",
            Self::EStartTargetMissing => "E_START_TARGET_MISSING",
            Self::EJumpTargetMissing => "E_JUMP_TARGET_MISSING",
            Self::EChoiceTargetMissing => "E_CHOICE_TARGET_MISSING",
            Self::ESceneStructure => "E_SCENE_STRUCTURE",
            Self::EPhaseTokenForbidden => "E_PHASE_TOKEN_FORBIDDEN",
            Self::EActorUnknown => "E_ACTOR_UNKNOWN",
            Self::EDialogueShapeInvalid => "E_DIALOGUE_SHAPE_INVALID",
            Self::EPositionInvalid => "E_POSITION_INVALID",
            Self::EEmotionUnknown => "E_EMOTION_UNKNOWN",
            Self::EPortraitModeInvalid => "E_PORTRAIT_MODE_INVALID",
            Self::EVariableUndeclaredRead => "E_VARIABLE_UNDECLARED_READ",
            Self::EVariableUndeclaredWrite => "E_VARIABLE_UNDECLARED_WRITE",
            Self::EVariableTypeMismatch => "E_VARIABLE_TYPE_MISMATCH",
            Self::EVariableCompoundAssignInvalid => "E_VARIABLE_COMPOUND_ASSIGN_INVALID",
            Self::EExpressionTypeInvalid => "E_EXPRESSION_TYPE_INVALID",
            Self::EConditionTypeInvalid => "E_CONDITION_TYPE_INVALID",
            Self::EChoiceStaticEmpty => "E_CHOICE_STATIC_EMPTY",
            Self::EStoryUnterminatedPath => "E_STORY_UNTERMINATED_PATH",
            Self::WChoicePossiblyEmpty => "W_CHOICE_POSSIBLY_EMPTY",
            Self::RChoiceExhausted => "R_CHOICE_EXHAUSTED",
            Self::RAssetNotFound => "R_ASSET_NOT_FOUND",
            Self::RAssetLoadFailed => "R_ASSET_LOAD_FAILED",
            Self::RAudioDeviceFailure => "R_AUDIO_DEVICE_FAILURE",
            Self::RSaveStateCorrupt => "R_SAVE_STATE_CORRUPT",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Lex,
    Parse,
    Validation,
    Prep,
    Story,
    Runtime,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Lex => "LEX",
            Self::Parse => "PARSE",
            Self::Validation => "VALIDATION",
            Self::Prep => "PREP",
            Self::Story => "STORY",
            Self::Runtime => "RUNTIME",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub phase: Phase,
    pub scene: String,
    pub line: usize,
    pub column: usize,
}

impl Diagnostic {
    pub fn new(
        code: DiagnosticCode,
        message: impl Into<String>,
        phase: Phase,
        scene: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            phase,
            scene: scene.into(),
            line,
            column,
        }
    }

    pub fn is_error(&self) -> bool {
        let code_str = self.code.to_string();
        code_str.starts_with("E_")
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"code":"{}","message":"{}","phase":"{}","scene":"{}","line":{},"column":{}}}"#,
            self.code,
            self.message.replace('\\', "\\\\").replace('"', "\\\""),
            self.phase,
            self.scene,
            self.line,
            self.column
        )
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] {}:{}:{} {}",
            self.code, self.phase, self.scene, self.line, self.column, self.message
        )
    }
}

impl Ord for Diagnostic {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line
            .cmp(&other.line)
            .then(self.column.cmp(&other.column))
            .then(self.code.to_string().cmp(&other.code.to_string()))
    }
}

impl PartialOrd for Diagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Diagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.column == other.column && self.code == other.code
    }
}

impl Eq for Diagnostic {}
