use crate::ast::*;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use crate::token::{Spanned, Token};

pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|s| &s.token)
            .unwrap_or(&Token::Eof)
    }

    fn current_span(&self) -> (usize, usize) {
        self.tokens
            .get(self.pos)
            .map(|s| (s.line, s.column))
            .unwrap_or((0, 0))
    }

    fn peek_n(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.pos + offset)
            .map(|s| &s.token)
            .unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Spanned {
        let s = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        s
    }

    fn expect(&mut self, expected: &Token) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            let (l, c) = self.current_span();
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                format!("Expected {}, found {}", expected.name(), self.peek().name()),
                Phase::Parse,
                "GLOBAL",
                l,
                c,
            ));
            false
        }
    }

    fn eat_optional_semicolon(&mut self) {
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
    }

    // -----------------------------------------------------------------------
    // Top-level
    // -----------------------------------------------------------------------

    pub fn parse(&mut self) -> Option<Script> {
        let init = self.parse_init_block()?;
        let mut scenes = Vec::new();
        while self.peek() != &Token::Eof {
            if let Some(scene) = self.parse_scene() {
                scenes.push(scene);
            } else {
                // Recovery: skip to next '*'
                self.advance();
            }
        }
        Some(Script { init, scenes })
    }

    // -----------------------------------------------------------------------
    // INIT block
    // -----------------------------------------------------------------------

    fn parse_init_block(&mut self) -> Option<InitBlock> {
        let (line, column) = self.current_span();
        if !self.expect(&Token::Star) {
            return None;
        }
        if !self.expect(&Token::Init) {
            return None;
        }
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut variables = Vec::new();
        let mut actors = Vec::new();
        let mut start: Option<StartDirective> = None;

        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Dollar => {
                    if let Some(v) = self.parse_var_decl() {
                        variables.push(v);
                    }
                }
                Token::AtActor => {
                    if let Some(a) = self.parse_actor_decl() {
                        actors.push(a);
                    }
                }
                Token::AtStart => {
                    let (sl, sc) = self.current_span();
                    self.advance(); // consume @start
                    if let Token::Ident(name) = self.peek().clone() {
                        self.advance();
                        self.eat_optional_semicolon();
                        start = Some(StartDirective {
                            target: name,
                            line: sl,
                            column: sc,
                        });
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            DiagnosticCode::ESyntax,
                            "Expected scene label after @start",
                            Phase::Parse,
                            "INIT",
                            sl,
                            sc,
                        ));
                    }
                }
                _ => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        format!("Unexpected token {} in INIT block", self.peek().name()),
                        Phase::Parse,
                        "INIT",
                        l,
                        c,
                    ));
                    self.advance();
                }
            }
        }

        self.expect(&Token::RBrace);

        let start = match start {
            Some(s) => s,
            None => {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EStartCount,
                    "INIT block must contain exactly one @start directive",
                    Phase::Parse,
                    "INIT",
                    line,
                    column,
                ));
                StartDirective {
                    target: String::new(),
                    line,
                    column,
                }
            }
        };

        Some(InitBlock {
            variables,
            actors,
            start,
            line,
            column,
        })
    }

    fn parse_var_decl(&mut self) -> Option<VarDecl> {
        let (line, column) = self.current_span();
        self.advance(); // $
        if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            if !self.expect(&Token::As) {
                return None;
            }
            let var_type = self.parse_var_type()?;
            if !self.expect(&Token::Eq) {
                return None;
            }
            let value = self.parse_expression()?;
            self.eat_optional_semicolon();
            Some(VarDecl {
                name,
                var_type,
                value,
                line,
                column,
            })
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected variable name after '$'",
                Phase::Parse,
                "GLOBAL",
                line,
                column,
            ));
            None
        }
    }

    fn parse_var_type(&mut self) -> Option<VarType> {
        let (line, column) = self.current_span();
        let var_type = match self.peek() {
            Token::TypeInteger => VarType::Integer,
            Token::TypeString => VarType::String,
            Token::TypeBoolean => VarType::Boolean,
            Token::TypeDecimal => VarType::Decimal,
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    format!(
                        "Expected variable type (integer, string, boolean, decimal), found {}",
                        self.peek().name()
                    ),
                    Phase::Parse,
                    "INIT",
                    line,
                    column,
                ));
                return None;
            }
        };
        self.advance();
        Some(var_type)
    }

    fn parse_actor_decl(&mut self) -> Option<ActorDecl> {
        let (line, column) = self.current_span();
        self.advance(); // @actor

        let id = if let Token::Ident(s) = self.peek().clone() {
            self.advance();
            s
        } else {
            let (l, c) = self.current_span();
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected actor ID after @actor",
                Phase::Parse,
                "INIT",
                l,
                c,
            ));
            return None;
        };

        let display_name = if let Token::StringLit(s) = self.peek().clone() {
            self.advance();
            s
        } else {
            let (l, c) = self.current_span();
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected display name string after actor ID",
                Phase::Parse,
                "INIT",
                l,
                c,
            ));
            return None;
        };

        let mut portraits = Vec::new();

        // Check for portrait block or semicolon
        if self.peek() == &Token::LBrace {
            self.advance(); // {
            while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                let (pl, pc) = self.current_span();
                if let Token::Ident(emotion) = self.peek().clone() {
                    self.advance();
                    if !self.expect(&Token::Arrow) {
                        break;
                    }
                    if let Token::StringLit(path) = self.peek().clone() {
                        self.advance();
                        portraits.push(PortraitEntry {
                            emotion,
                            path,
                            line: pl,
                            column: pc,
                        });
                    } else {
                        self.diagnostics.push(Diagnostic::new(
                            DiagnosticCode::ESyntax,
                            "Expected portrait path string",
                            Phase::Parse,
                            "INIT",
                            pl,
                            pc,
                        ));
                        break;
                    }
                } else {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected emotion key in portrait map",
                        Phase::Parse,
                        "INIT",
                        l,
                        c,
                    ));
                    break;
                }
            }
            self.expect(&Token::RBrace);
        } else {
            self.eat_optional_semicolon();
        }

        Some(ActorDecl {
            id,
            display_name,
            portraits,
            line,
            column,
        })
    }

    // -----------------------------------------------------------------------
    // Scene
    // -----------------------------------------------------------------------

    fn parse_scene(&mut self) -> Option<Scene> {
        let (line, column) = self.current_span();
        if !self.expect(&Token::Star) {
            return None;
        }
        let label = if let Token::Ident(s) = self.peek().clone() {
            self.advance();
            s
        } else {
            let (l, c) = self.current_span();
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected scene label after '*'",
                Phase::Parse,
                "GLOBAL",
                l,
                c,
            ));
            return None;
        };

        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut prep: Option<PrepBlock> = None;
        let mut story: Option<StoryBlock> = None;

        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::HashPrep => {
                    let (pl, pc) = self.current_span();
                    self.advance();
                    if prep.is_some() {
                        self.diagnostics.push(Diagnostic::new(
                            DiagnosticCode::ESceneStructure,
                            "Duplicate #PREP phase in scene",
                            Phase::Parse,
                            &label,
                            pl,
                            pc,
                        ));
                    }
                    if story.is_some() {
                        self.diagnostics.push(Diagnostic::new(
                            DiagnosticCode::ESceneStructure,
                            "#PREP must appear before #STORY",
                            Phase::Parse,
                            &label,
                            pl,
                            pc,
                        ));
                    }
                    prep = Some(self.parse_prep_block(&label, pl, pc));
                }
                Token::HashStory => {
                    let (sl, sc) = self.current_span();
                    self.advance();
                    if story.is_some() {
                        self.diagnostics.push(Diagnostic::new(
                            DiagnosticCode::ESceneStructure,
                            "Duplicate #STORY phase in scene",
                            Phase::Parse,
                            &label,
                            sl,
                            sc,
                        ));
                    }
                    story = Some(self.parse_story_block(&label, sl, sc));
                }
                _ => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESceneStructure,
                        format!("Expected #PREP or #STORY, found {}", self.peek().name()),
                        Phase::Parse,
                        &label,
                        l,
                        c,
                    ));
                    self.advance();
                }
            }
        }

        self.expect(&Token::RBrace);

        let story = match story {
            Some(s) => s,
            None => {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESceneStructure,
                    "Scene must contain a #STORY phase",
                    Phase::Parse,
                    &label,
                    line,
                    column,
                ));
                StoryBlock {
                    statements: Vec::new(),
                    line,
                    column,
                }
            }
        };

        Some(Scene {
            label,
            prep,
            story,
            line,
            column,
        })
    }

    // -----------------------------------------------------------------------
    // #PREP
    // -----------------------------------------------------------------------

    fn parse_prep_block(&mut self, scene: &str, line: usize, column: usize) -> PrepBlock {
        let mut statements = Vec::new();

        while !matches!(self.peek(), Token::HashStory | Token::RBrace | Token::Eof) {
            if let Some(stmt) = self.parse_prep_statement(scene) {
                statements.push(stmt);
            } else {
                self.advance();
            }
        }

        PrepBlock {
            statements,
            line,
            column,
        }
    }

    fn parse_prep_statement(&mut self, scene: &str) -> Option<PrepStatement> {
        match self.peek().clone() {
            Token::AtBg => {
                let (l, c) = self.current_span();
                self.advance();
                if let Token::StringLit(path) = self.peek().clone() {
                    self.advance();
                    self.eat_optional_semicolon();
                    Some(PrepStatement::BgDirective {
                        path,
                        line: l,
                        column: c,
                    })
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected path string after @bg",
                        Phase::Parse,
                        scene,
                        l,
                        c,
                    ));
                    None
                }
            }
            Token::AtBgm => {
                let (l, c) = self.current_span();
                self.advance();
                let value = if self.peek() == &Token::Stop {
                    self.advance();
                    BgmValue::Stop
                } else if let Token::StringLit(path) = self.peek().clone() {
                    self.advance();
                    BgmValue::Path(path)
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected path string or STOP after @bgm",
                        Phase::Parse,
                        scene,
                        l,
                        c,
                    ));
                    return None;
                };
                self.eat_optional_semicolon();
                Some(PrepStatement::BgmDirective {
                    value,
                    line: l,
                    column: c,
                })
            }
            Token::AtSfx => {
                let (l, c) = self.current_span();
                self.advance();
                if let Token::StringLit(path) = self.peek().clone() {
                    self.advance();
                    self.eat_optional_semicolon();
                    Some(PrepStatement::SfxDirective {
                        path,
                        line: l,
                        column: c,
                    })
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected path string after @sfx",
                        Phase::Parse,
                        scene,
                        l,
                        c,
                    ));
                    None
                }
            }
            Token::Dollar => {
                if let Some(assign) = self.parse_var_assign(scene) {
                    Some(PrepStatement::VarAssign(assign))
                } else {
                    None
                }
            }
            Token::If => {
                if let Some(if_else) = self.parse_prep_if_else(scene) {
                    Some(PrepStatement::IfElse(if_else))
                } else {
                    None
                }
            }
            _ => {
                let (l, c) = self.current_span();
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EPhaseTokenForbidden,
                    format!("Token {} is not allowed in #PREP", self.peek().name()),
                    Phase::Parse,
                    scene,
                    l,
                    c,
                ));
                None
            }
        }
    }

    fn parse_var_assign(&mut self, scene: &str) -> Option<VarAssign> {
        let (line, column) = self.current_span();
        self.advance(); // $

        if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            let op = match self.peek() {
                Token::Eq => {
                    self.advance();
                    AssignOp::Set
                }
                Token::PlusEq => {
                    self.advance();
                    AssignOp::AddEq
                }
                Token::MinusEq => {
                    self.advance();
                    AssignOp::SubEq
                }
                _ => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected assignment operator (=, +=, -=)",
                        Phase::Parse,
                        scene,
                        l,
                        c,
                    ));
                    return None;
                }
            };
            let value = self.parse_expression()?;
            self.eat_optional_semicolon();
            Some(VarAssign {
                name,
                op,
                value,
                line,
                column,
            })
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected variable name after '$'",
                Phase::Parse,
                scene,
                line,
                column,
            ));
            None
        }
    }

    fn parse_prep_if_else(&mut self, scene: &str) -> Option<PrepIfElse> {
        let (line, column) = self.current_span();
        self.advance(); // if

        if !self.expect(&Token::LParen) {
            return None;
        }
        let condition = self.parse_expression()?;
        if !self.expect(&Token::RParen) {
            return None;
        }
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut then_branch = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_prep_statement(scene) {
                then_branch.push(stmt);
            } else {
                self.advance();
            }
        }
        self.expect(&Token::RBrace);

        let else_branch = if self.peek() == &Token::Else {
            self.advance();
            if !self.expect(&Token::LBrace) {
                return None;
            }
            let mut stmts = Vec::new();
            while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                if let Some(stmt) = self.parse_prep_statement(scene) {
                    stmts.push(stmt);
                } else {
                    self.advance();
                }
            }
            self.expect(&Token::RBrace);
            Some(stmts)
        } else {
            None
        };

        Some(PrepIfElse {
            condition,
            then_branch,
            else_branch,
            line,
            column,
        })
    }

    // -----------------------------------------------------------------------
    // #STORY
    // -----------------------------------------------------------------------

    fn parse_story_block(&mut self, scene: &str, line: usize, column: usize) -> StoryBlock {
        let mut statements = Vec::new();

        while !matches!(
            self.peek(),
            Token::RBrace | Token::Eof | Token::HashPrep | Token::HashStory
        ) {
            if let Some(stmt) = self.parse_story_statement(scene) {
                statements.push(stmt);
            } else {
                // Avoid infinite loops on unrecoverable tokens
                if matches!(self.peek(), Token::RBrace | Token::Eof) {
                    break;
                }
                self.advance();
            }
        }

        StoryBlock {
            statements,
            line,
            column,
        }
    }

    fn parse_story_statement(&mut self, scene: &str) -> Option<StoryStatement> {
        match self.peek().clone() {
            Token::StringLit(text) => {
                let (l, c) = self.current_span();
                self.advance();
                self.eat_optional_semicolon();
                Some(StoryStatement::Narration {
                    text,
                    line: l,
                    column: c,
                })
            }
            Token::AtChoice => self.parse_choice_block(scene),
            Token::AtJump => {
                let (l, c) = self.current_span();
                self.advance();
                if let Token::Ident(target) = self.peek().clone() {
                    self.advance();
                    self.eat_optional_semicolon();
                    Some(StoryStatement::Jump {
                        target,
                        line: l,
                        column: c,
                    })
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected scene label after @jump",
                        Phase::Parse,
                        scene,
                        l,
                        c,
                    ));
                    None
                }
            }
            Token::AtEnd => {
                let (l, c) = self.current_span();
                self.advance();
                self.eat_optional_semicolon();
                Some(StoryStatement::End { line: l, column: c })
            }
            Token::If => self.parse_story_if_else(scene),
            Token::Ident(_) => self.parse_dialogue(scene),
            Token::Dollar => self.parse_story_var_output(scene),
            Token::AtBg | Token::AtBgm | Token::AtSfx => {
                let (l, c) = self.current_span();
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EPhaseTokenForbidden,
                    format!("{} is only allowed in #PREP", self.peek().name()),
                    Phase::Parse,
                    scene,
                    l,
                    c,
                ));
                self.advance();
                // Skip the argument
                if matches!(self.peek(), Token::StringLit(_) | Token::Stop) {
                    self.advance();
                }
                self.eat_optional_semicolon();
                None
            }
            _ => {
                let (l, c) = self.current_span();
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    format!("Unexpected token {} in #STORY", self.peek().name()),
                    Phase::Parse,
                    scene,
                    l,
                    c,
                ));
                None
            }
        }
    }

    fn parse_story_var_output(&mut self, scene: &str) -> Option<StoryStatement> {
        let (line, column) = self.current_span();
        self.advance(); // $

        let name = if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            name
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected variable name after '$' in #STORY",
                Phase::Parse,
                scene,
                line,
                column,
            ));
            return None;
        };

        if matches!(self.peek(), Token::Eq | Token::PlusEq | Token::MinusEq) {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::EPhaseTokenForbidden,
                "Variable mutation is forbidden in #STORY",
                Phase::Parse,
                scene,
                line,
                column,
            ));
            self.advance(); // assignment operator
            let _ = self.parse_expression();
            self.eat_optional_semicolon();
            return None;
        }

        if matches!(
            self.peek(),
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::EqEq
                | Token::NotEq
                | Token::Lt
                | Token::LtEq
                | Token::Gt
                | Token::GtEq
                | Token::LParen
                | Token::LBracket
                | Token::Arrow
        ) {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Standalone variable output in #STORY must be exactly '$name'",
                Phase::Parse,
                scene,
                line,
                column,
            ));
            self.recover_story_statement_tail();
            return None;
        }

        self.eat_optional_semicolon();

        Some(StoryStatement::VarOutput { name, line, column })
    }

    fn recover_story_statement_tail(&mut self) {
        while !matches!(
            self.peek(),
            Token::Semicolon
                | Token::RBrace
                | Token::Eof
                | Token::HashPrep
                | Token::HashStory
                | Token::AtChoice
                | Token::AtJump
                | Token::AtEnd
                | Token::If
                | Token::AtBg
                | Token::AtBgm
                | Token::AtSfx
                | Token::StringLit(_)
                | Token::Dollar
        ) {
            self.advance();
        }

        if self.peek() == &Token::Semicolon {
            self.advance();
        }
    }

    fn parse_dialogue(&mut self, scene: &str) -> Option<StoryStatement> {
        let (line, column) = self.current_span();
        let actor_id = if let Token::Ident(s) = self.peek().clone() {
            self.advance();
            s
        } else {
            return None;
        };

        let form = if self.peek() == &Token::LParen {
            // Portrait form: ActorID(<emotion>, <Position>)
            self.advance(); // (

            let emotion = if let Token::Ident(e) = self.peek().clone() {
                self.advance();
                e
            } else {
                let (l, c) = self.current_span();
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EDialogueShapeInvalid,
                    "Expected emotion key in portrait-form dialogue",
                    Phase::Parse,
                    scene,
                    l,
                    c,
                ));
                return None;
            };

            if !self.expect(&Token::Comma) {
                return None;
            }

            let position_str = if let Token::Ident(p) = self.peek().clone() {
                self.advance();
                p
            } else {
                let (l, c) = self.current_span();
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EDialogueShapeInvalid,
                    "Expected position in portrait-form dialogue",
                    Phase::Parse,
                    scene,
                    l,
                    c,
                ));
                return None;
            };

            let position = match Position::from_str(&position_str) {
                Some(p) => p,
                None => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::EPositionInvalid,
                        format!(
                            "Invalid position '{}'. Valid: Left, Center, Right, L, C, R",
                            position_str
                        ),
                        Phase::Parse,
                        scene,
                        l,
                        c,
                    ));
                    Position::Left // fallback
                }
            };

            if !self.expect(&Token::RParen) {
                return None;
            }

            DialogueForm::Portrait { emotion, position }
        } else {
            DialogueForm::NameOnly
        };

        if !self.expect(&Token::Colon) {
            return None;
        }

        let text = if let Token::StringLit(s) = self.peek().clone() {
            self.advance();
            s
        } else {
            let (l, c) = self.current_span();
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected dialogue text string",
                Phase::Parse,
                scene,
                l,
                c,
            ));
            return None;
        };

        self.eat_optional_semicolon();

        Some(StoryStatement::Dialogue(Dialogue {
            actor_id,
            form,
            text,
            line,
            column,
        }))
    }

    fn parse_choice_block(&mut self, scene: &str) -> Option<StoryStatement> {
        let (line, column) = self.current_span();
        self.advance(); // @choice
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut options = Vec::new();

        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            match self.peek().clone() {
                Token::StringLit(_) => {
                    if let Some(opt) = self.parse_choice_option(scene, None) {
                        options.push(opt);
                    }
                }
                Token::If => {
                    let (il, ic) = self.current_span();
                    self.advance(); // if
                    if !self.expect(&Token::LParen) {
                        continue;
                    }
                    let cond = match self.parse_expression() {
                        Some(e) => e,
                        None => continue,
                    };
                    if !self.expect(&Token::RParen) {
                        continue;
                    }
                    if !self.expect(&Token::LBrace) {
                        continue;
                    }
                    while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                        if let Token::StringLit(_) = self.peek().clone() {
                            if let Some(opt) = self.parse_choice_option(scene, Some(cond.clone())) {
                                options.push(opt);
                            }
                        } else {
                            let (l, c) = self.current_span();
                            self.diagnostics.push(Diagnostic::new(
                                DiagnosticCode::ESyntax,
                                format!(
                                    "Expected choice option string inside conditional, found {}",
                                    self.peek().name()
                                ),
                                Phase::Parse,
                                scene,
                                l,
                                c,
                            ));
                            self.advance();
                        }
                    }
                    self.expect(&Token::RBrace);
                    let _ = (il, ic); // suppress unused
                }
                _ => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        format!("Unexpected token {} in @choice block", self.peek().name()),
                        Phase::Parse,
                        scene,
                        l,
                        c,
                    ));
                    self.advance();
                }
            }
        }

        self.expect(&Token::RBrace);

        Some(StoryStatement::Choice(ChoiceBlock {
            options,
            line,
            column,
        }))
    }

    fn parse_choice_option(
        &mut self,
        scene: &str,
        condition: Option<Expr>,
    ) -> Option<ChoiceOption> {
        let (line, column) = self.current_span();
        let text = if let Token::StringLit(s) = self.peek().clone() {
            self.advance();
            s
        } else {
            return None;
        };

        if !self.expect(&Token::Arrow) {
            return None;
        }

        let target = if let Token::Ident(s) = self.peek().clone() {
            self.advance();
            s
        } else {
            let (l, c) = self.current_span();
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected scene label after '->'",
                Phase::Parse,
                scene,
                l,
                c,
            ));
            return None;
        };

        self.eat_optional_semicolon();

        Some(ChoiceOption {
            text,
            target,
            condition,
            line,
            column,
        })
    }

    fn parse_story_if_else(&mut self, scene: &str) -> Option<StoryStatement> {
        let (line, column) = self.current_span();
        self.advance(); // if

        if !self.expect(&Token::LParen) {
            return None;
        }
        let condition = self.parse_expression()?;
        if !self.expect(&Token::RParen) {
            return None;
        }
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut then_branch = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_story_statement(scene) {
                then_branch.push(stmt);
            } else {
                if matches!(self.peek(), Token::RBrace | Token::Eof) {
                    break;
                }
                self.advance();
            }
        }
        self.expect(&Token::RBrace);

        let else_branch = if self.peek() == &Token::Else {
            self.advance();
            if !self.expect(&Token::LBrace) {
                return None;
            }
            let mut stmts = Vec::new();
            while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                if let Some(stmt) = self.parse_story_statement(scene) {
                    stmts.push(stmt);
                } else {
                    if matches!(self.peek(), Token::RBrace | Token::Eof) {
                        break;
                    }
                    self.advance();
                }
            }
            self.expect(&Token::RBrace);
            Some(stmts)
        } else {
            None
        };

        Some(StoryStatement::IfElse(StoryIfElse {
            condition,
            then_branch,
            else_branch,
            line,
            column,
        }))
    }

    // -----------------------------------------------------------------------
    // Expressions
    // -----------------------------------------------------------------------

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut left = self.parse_additive()?;

        loop {
            let op = match self.peek() {
                Token::EqEq => BinOperator::EqEq,
                Token::NotEq => BinOperator::NotEq,
                Token::Lt => BinOperator::Lt,
                Token::LtEq => BinOperator::LtEq,
                Token::Gt => BinOperator::Gt,
                Token::GtEq => BinOperator::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_additive(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.peek() {
                Token::Plus => BinOperator::Add,
                Token::Minus => BinOperator::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_multiplicative(&mut self) -> Option<Expr> {
        let mut left = self.parse_primary()?;

        loop {
            let op = match self.peek() {
                Token::Star => BinOperator::Mul,
                Token::Slash => BinOperator::Div,
                Token::Percent => BinOperator::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_primary()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.peek().clone() {
            Token::IntLit(n) => {
                self.advance();
                Some(Expr::IntLit(n))
            }
            Token::DecimalLit(n) => {
                self.advance();
                Some(Expr::DecimalLit(n))
            }
            Token::BoolLit(b) => {
                self.advance();
                Some(Expr::BoolLit(b))
            }
            Token::StringLit(s) => {
                self.advance();
                Some(Expr::StringLit(s))
            }
            Token::Ident(name) => self.parse_call_expr(name),
            Token::Dollar => {
                let (line, column) = self.current_span();
                self.advance(); // $
                if let Token::Ident(name) = self.peek().clone() {
                    self.advance();
                    Some(Expr::VarRef { name, line, column })
                } else {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected variable name after '$'",
                        Phase::Parse,
                        "GLOBAL",
                        l,
                        c,
                    ));
                    None
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&Token::RParen);
                Some(expr)
            }
            Token::LBracket => self.parse_list_literal(),
            _ => {
                let (l, c) = self.current_span();
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    format!("Expected expression, found {}", self.peek().name()),
                    Phase::Parse,
                    "GLOBAL",
                    l,
                    c,
                ));
                None
            }
        }
    }

    fn parse_call_expr(&mut self, name: String) -> Option<Expr> {
        let (line, column) = self.current_span();

        if self.peek_n(1) != &Token::LParen {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                format!(
                    "Unexpected identifier '{}' in expression; did you mean '${}' or a function call?",
                    name, name
                ),
                Phase::Parse,
                "GLOBAL",
                line,
                column,
            ));
            self.advance();
            return None;
        }

        self.advance(); // function name
        self.advance(); // '('

        let mut args = Vec::new();
        if self.peek() != &Token::RParen {
            loop {
                let arg = self.parse_expression()?;
                args.push(arg);

                if self.peek() == &Token::Comma {
                    self.advance();
                    continue;
                }
                break;
            }
        }

        if !self.expect(&Token::RParen) {
            return None;
        }

        Some(Expr::Call {
            name,
            args,
            line,
            column,
        })
    }

    fn parse_list_literal(&mut self) -> Option<Expr> {
        let (line, column) = self.current_span();
        self.advance(); // '['

        let mut items = Vec::new();
        if self.peek() != &Token::RBracket {
            loop {
                let value = self.parse_expression()?;
                items.push(value);

                if self.peek() == &Token::Comma {
                    self.advance();
                    continue;
                }
                break;
            }
        }

        if !self.expect(&Token::RBracket) {
            return None;
        }

        Some(Expr::ListLit {
            items,
            line,
            column,
        })
    }
}
