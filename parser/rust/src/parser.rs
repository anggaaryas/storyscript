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
        let mut logic_blocks = Vec::new();
        let mut scenes = Vec::new();
        while self.peek() != &Token::Eof {
            match self.peek() {
                Token::Logic => {
                    if let Some(logic_block) = self.parse_logic_block("GLOBAL") {
                        logic_blocks.push(logic_block);
                    } else {
                        self.advance();
                    }
                }
                Token::Star => {
                    if let Some(scene) = self.parse_scene() {
                        scenes.push(scene);
                    } else {
                        // Recovery: skip to next '*'
                        self.advance();
                    }
                }
                _ => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        format!("Unexpected token {} at top-level", self.peek().name()),
                        Phase::Parse,
                        "GLOBAL",
                        l,
                        c,
                    ));
                    self.advance();
                }
            }
        }
        Some(Script {
            init,
            logic_blocks,
            scenes,
        })
    }

    pub fn parse_child_module(&mut self) -> Option<ChildModule> {
        let mut require: Option<RequireBlock> = None;
        let mut logic_blocks = Vec::new();
        let mut scenes = Vec::new();

        while self.peek() != &Token::Eof {
            match self.peek() {
                Token::Star => match self.peek_n(1) {
                    Token::Require => {
                        if let Some(req) = self.parse_require_block() {
                            if require.is_some() {
                                self.diagnostics.push(Diagnostic::new(
                                    DiagnosticCode::ERequireCount,
                                    "Child file must contain exactly one * REQUIRE block",
                                    Phase::Parse,
                                    "GLOBAL",
                                    req.line,
                                    req.column,
                                ));
                            } else {
                                require = Some(req);
                            }
                        }
                    }
                    Token::Init => {
                        let (l, c) = self.current_span();
                        self.diagnostics.push(Diagnostic::new(
                            DiagnosticCode::EIncludeChildInitForbidden,
                            "Included child file must not declare * INIT",
                            Phase::Parse,
                            "GLOBAL",
                            l,
                            c,
                        ));
                        self.skip_top_level_block();
                    }
                    Token::Ident(_) => {
                        if let Some(scene) = self.parse_scene() {
                            scenes.push(scene);
                        } else {
                            self.advance();
                        }
                    }
                    _ => {
                        let (l, c) = self.current_span();
                        self.diagnostics.push(Diagnostic::new(
                            DiagnosticCode::ESyntax,
                            format!(
                                "Expected '* REQUIRE' or scene label, found * {}",
                                self.peek_n(1).name()
                            ),
                            Phase::Parse,
                            "GLOBAL",
                            l,
                            c,
                        ));
                        self.advance();
                    }
                },
                Token::Logic => {
                    if let Some(logic_block) = self.parse_logic_block("GLOBAL") {
                        logic_blocks.push(logic_block);
                    } else {
                        self.advance();
                    }
                }
                Token::AtInclude => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::EPhaseTokenForbidden,
                        "@include is only allowed inside root * INIT",
                        Phase::Parse,
                        "GLOBAL",
                        l,
                        c,
                    ));
                    let _ = self.parse_include_directive("GLOBAL");
                }
                Token::AtStart => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::EPhaseTokenForbidden,
                        "@start is only allowed inside root * INIT",
                        Phase::Parse,
                        "GLOBAL",
                        l,
                        c,
                    ));
                    self.advance();
                    if let Token::Ident(_) = self.peek() {
                        self.advance();
                    }
                    self.eat_optional_semicolon();
                }
                _ => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        format!("Unexpected token {} at top-level", self.peek().name()),
                        Phase::Parse,
                        "GLOBAL",
                        l,
                        c,
                    ));
                    self.advance();
                }
            }
        }

        let require = match require {
            Some(req) => req,
            None => {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ERequireCount,
                    "Included child file must contain exactly one * REQUIRE block",
                    Phase::Parse,
                    "GLOBAL",
                    1,
                    1,
                ));
                RequireBlock {
                    variables: Vec::new(),
                    actors: Vec::new(),
                    line: 1,
                    column: 1,
                }
            }
        };

        Some(ChildModule {
            require,
            logic_blocks,
            scenes,
        })
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
        let mut includes = Vec::new();
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
                        if start.is_some() {
                            self.diagnostics.push(Diagnostic::new(
                                DiagnosticCode::EStartCount,
                                "INIT block must contain exactly one @start directive",
                                Phase::Parse,
                                "INIT",
                                sl,
                                sc,
                            ));
                        } else {
                            start = Some(StartDirective {
                                target: name,
                                line: sl,
                                column: sc,
                            });
                        }
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
                Token::AtInclude => {
                    includes.extend(self.parse_include_directive("INIT"));
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
            includes,
            start,
            line,
            column,
        })
    }

    fn parse_include_directive(&mut self, scene: &str) -> Vec<IncludeDirective> {
        self.advance(); // @include

        if !self.expect(&Token::LBracket) {
            return Vec::new();
        }

        let mut includes = Vec::new();

        if self.peek() != &Token::RBracket {
            loop {
                let (pl, pc) = self.current_span();
                if let Token::StringLit(path) = self.peek().clone() {
                    self.advance();
                    includes.push(IncludeDirective {
                        path,
                        line: pl,
                        column: pc,
                    });
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected string path inside @include manifest",
                        Phase::Parse,
                        scene,
                        pl,
                        pc,
                    ));
                    break;
                }

                if self.peek() == &Token::Comma {
                    self.advance();
                    continue;
                }
                break;
            }
        }

        self.expect(&Token::RBracket);
        self.eat_optional_semicolon();
        includes
    }

    fn parse_require_block(&mut self) -> Option<RequireBlock> {
        let (line, column) = self.current_span();

        if !self.expect(&Token::Star) {
            return None;
        }
        if !self.expect(&Token::Require) {
            return None;
        }
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut variables = Vec::new();
        let mut actors = Vec::new();

        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Dollar => {
                    if let Some(var) = self.parse_require_var_decl() {
                        variables.push(var);
                    }
                }
                Token::AtActor => {
                    if let Some(actor) = self.parse_require_actor_ref() {
                        actors.push(actor);
                    }
                }
                _ => {
                    let (l, c) = self.current_span();
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        format!("Unexpected token {} in REQUIRE block", self.peek().name()),
                        Phase::Parse,
                        "REQUIRE",
                        l,
                        c,
                    ));
                    self.advance();
                }
            }
        }

        self.expect(&Token::RBrace);

        Some(RequireBlock {
            variables,
            actors,
            line,
            column,
        })
    }

    fn parse_require_var_decl(&mut self) -> Option<RequireVarDecl> {
        let (line, column) = self.current_span();
        self.advance(); // $

        let name = if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            name
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected variable name after '$' in REQUIRE",
                Phase::Parse,
                "REQUIRE",
                line,
                column,
            ));
            return None;
        };

        if !self.expect(&Token::As) {
            return None;
        }

        let var_type = self.parse_var_type("REQUIRE")?;

        if self.peek() == &Token::Eq {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "REQUIRE variable declaration must not have initializer",
                Phase::Parse,
                "REQUIRE",
                line,
                column,
            ));
            self.advance();
            let _ = self.parse_expression();
        }

        self.eat_optional_semicolon();

        Some(RequireVarDecl {
            name,
            var_type,
            line,
            column,
        })
    }

    fn parse_require_actor_ref(&mut self) -> Option<RequireActorRef> {
        let (line, column) = self.current_span();
        self.advance(); // @actor

        let id = if let Token::Ident(id) = self.peek().clone() {
            self.advance();
            id
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected actor ID after @actor in REQUIRE",
                Phase::Parse,
                "REQUIRE",
                line,
                column,
            ));
            return None;
        };

        if !self.expect(&Token::LBracket) {
            return None;
        }

        let mut emotions = Vec::new();
        while self.peek() != &Token::RBracket && self.peek() != &Token::Eof {
            let (el, ec) = self.current_span();
            if let Token::Ident(name) = self.peek().clone() {
                self.advance();
                emotions.push(RequireEmotionRef {
                    name,
                    line: el,
                    column: ec,
                });
                if self.peek() == &Token::Comma {
                    self.advance();
                }
            } else {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    "Expected emotion key inside REQUIRE actor array",
                    Phase::Parse,
                    "REQUIRE",
                    el,
                    ec,
                ));
                self.advance();
            }
        }

        self.expect(&Token::RBracket);
        self.eat_optional_semicolon();

        Some(RequireActorRef {
            id,
            emotions,
            line,
            column,
        })
    }

    fn skip_top_level_block(&mut self) {
        if self.peek() == &Token::Star {
            self.advance();
        }

        if !matches!(self.peek(), Token::LBrace) {
            self.advance();
        }

        if self.peek() == &Token::LBrace {
            self.advance();
            let mut depth = 1usize;
            while self.peek() != &Token::Eof && depth > 0 {
                match self.peek() {
                    Token::LBrace => depth += 1,
                    Token::RBrace => depth -= 1,
                    _ => {}
                }
                self.advance();
            }
        }
    }

    fn parse_var_decl(&mut self) -> Option<VarDecl> {
        let (line, column) = self.current_span();
        self.advance(); // $
        if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            if !self.expect(&Token::As) {
                return None;
            }
            let var_type = self.parse_var_type("INIT")?;
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

    fn parse_var_type(&mut self, scene: &str) -> Option<VarType> {
        if self.peek() == &Token::TypeArray {
            let (line, column) = self.current_span();
            self.advance();

            if !self.expect(&Token::Lt) {
                return None;
            }

            let scalar_type = self.parse_scalar_var_type(scene, "array element type")?;

            if !self.expect(&Token::Gt) {
                return None;
            }

            let array_type = match scalar_type {
                VarType::Integer => VarType::ArrayInteger,
                VarType::String => VarType::ArrayString,
                VarType::Boolean => VarType::ArrayBoolean,
                VarType::Decimal => VarType::ArrayDecimal,
                _ => {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "array<type> supports only scalar element types (integer, string, boolean, decimal)",
                        Phase::Parse,
                        scene,
                        line,
                        column,
                    ));
                    return None;
                }
            };

            return Some(array_type);
        }

        self.parse_scalar_var_type(scene, "variable type")
    }

    fn parse_scalar_var_type(&mut self, scene: &str, context: &str) -> Option<VarType> {
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
                        "Expected {} (integer, string, boolean, decimal), found {}",
                        context,
                        self.peek().name()
                    ),
                    Phase::Parse,
                    scene,
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
    // Top-level logic blocks
    // -----------------------------------------------------------------------

    fn parse_logic_block(&mut self, scene: &str) -> Option<LogicBlock> {
        let (line, column) = self.current_span();
        if !self.expect(&Token::Logic) {
            return None;
        }

        let name = if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            name
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected logic function name after 'logic'",
                Phase::Parse,
                scene,
                line,
                column,
            ));
            return None;
        };

        if !self.expect(&Token::LParen) {
            return None;
        }
        let params = self.parse_logic_param_list(scene)?;
        if !self.expect(&Token::RParen) {
            return None;
        }

        let return_type = if self.peek() == &Token::Arrow {
            self.advance();
            Some(self.parse_var_type(scene)?)
        } else {
            None
        };

        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_logic_statement(&name) {
                body.push(stmt);
            } else {
                self.advance();
            }
        }
        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(LogicBlock {
            name,
            params,
            return_type,
            body,
            line,
            column,
        })
    }

    fn parse_logic_param_list(&mut self, scene: &str) -> Option<Vec<LogicParam>> {
        let mut params = Vec::new();

        if self.peek() == &Token::RParen {
            return Some(params);
        }

        loop {
            let (line, column) = self.current_span();
            if !self.expect(&Token::Dollar) {
                return None;
            }

            let name = if let Token::Ident(name) = self.peek().clone() {
                self.advance();
                name
            } else {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    "Expected parameter name after '$'",
                    Phase::Parse,
                    scene,
                    line,
                    column,
                ));
                return None;
            };

            if !self.expect(&Token::As) {
                return None;
            }
            let var_type = self.parse_var_type(scene)?;

            params.push(LogicParam {
                name,
                var_type,
                line,
                column,
            });

            if self.peek() == &Token::Comma {
                self.advance();
                continue;
            }
            break;
        }

        Some(params)
    }

    fn parse_logic_statement(&mut self, scene: &str) -> Option<PrepStatement> {
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
                if matches!(self.peek_n(1), Token::Ident(_)) && self.peek_n(2) == &Token::As {
                    if let Some(decl) = self.parse_scene_var_decl(scene) {
                        Some(PrepStatement::VarDecl(decl))
                    } else {
                        None
                    }
                } else if let Some(assign) = self.parse_var_assign(scene) {
                    Some(PrepStatement::VarAssign(assign))
                } else {
                    None
                }
            }
            Token::If => {
                if let Some(if_else) = self.parse_logic_if_else(scene) {
                    Some(PrepStatement::IfElse(if_else))
                } else {
                    None
                }
            }
            Token::For => self.parse_logic_for_snapshot(scene),
            Token::Repeat => self.parse_logic_repeat(scene),
            Token::Break => {
                let (line, column) = self.current_span();
                self.advance();
                self.eat_optional_semicolon();
                Some(PrepStatement::Break { line, column })
            }
            Token::Continue => {
                let (line, column) = self.current_span();
                self.advance();
                self.eat_optional_semicolon();
                Some(PrepStatement::Continue { line, column })
            }
            Token::Return => self.parse_return_statement(scene),
            Token::Ident(_) => self.parse_prep_call_statement(scene),
            _ => {
                let (l, c) = self.current_span();
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EPhaseTokenForbidden,
                    format!("Token {} is not allowed in logic block", self.peek().name()),
                    Phase::Parse,
                    scene,
                    l,
                    c,
                ));
                None
            }
        }
    }

    fn parse_logic_if_else(&mut self, scene: &str) -> Option<PrepIfElse> {
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
            if let Some(stmt) = self.parse_logic_statement(scene) {
                then_branch.push(stmt);
            } else {
                self.advance();
            }
        }
        self.expect(&Token::RBrace);

        let else_branch = if self.peek() == &Token::Else {
            self.advance();
            if self.peek() == &Token::If {
                let nested_if = self.parse_logic_if_else(scene)?;
                Some(vec![PrepStatement::IfElse(nested_if)])
            } else {
                if !self.expect(&Token::LBrace) {
                    return None;
                } else {
                    let mut stmts = Vec::new();
                    while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                        if let Some(stmt) = self.parse_logic_statement(scene) {
                            stmts.push(stmt);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(&Token::RBrace);
                    Some(stmts)
                }
            }
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

    fn parse_logic_for_snapshot(&mut self, scene: &str) -> Option<PrepStatement> {
        let (line, column) = self.current_span();
        self.advance(); // for

        let (item_name, array_name) = self.parse_for_snapshot_header(scene, line, column)?;
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_logic_statement(scene) {
                body.push(stmt);
            } else {
                self.advance();
            }
        }
        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(PrepStatement::ForSnapshot(PrepForSnapshot {
            item_name,
            array_name,
            body,
            line,
            column,
        }))
    }

    fn parse_logic_repeat(&mut self, scene: &str) -> Option<PrepStatement> {
        let (line, column) = self.current_span();
        self.advance(); // repeat

        if !self.expect(&Token::LParen) {
            return None;
        }
        let count = self.parse_repeat_count(scene)?;
        if !self.expect(&Token::RParen) {
            return None;
        }
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_logic_statement(scene) {
                body.push(stmt);
            } else {
                self.advance();
            }
        }
        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(PrepStatement::Repeat(PrepRepeat {
            count,
            body,
            line,
            column,
        }))
    }

    fn parse_return_statement(&mut self, _scene: &str) -> Option<PrepStatement> {
        let (line, column) = self.current_span();
        self.advance(); // return

        let value = if matches!(self.peek(), Token::Semicolon | Token::RBrace) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.eat_optional_semicolon();
        Some(PrepStatement::Return {
            value,
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
                if matches!(self.peek_n(1), Token::Ident(_)) && self.peek_n(2) == &Token::As {
                    if let Some(decl) = self.parse_scene_var_decl(scene) {
                        Some(PrepStatement::VarDecl(decl))
                    } else {
                        None
                    }
                } else if let Some(assign) = self.parse_var_assign(scene) {
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
            Token::For => self.parse_prep_for_snapshot(scene),
            Token::Repeat => self.parse_prep_repeat(scene),
            Token::Break => {
                let (line, column) = self.current_span();
                self.advance();
                self.eat_optional_semicolon();
                Some(PrepStatement::Break { line, column })
            }
            Token::Continue => {
                let (line, column) = self.current_span();
                self.advance();
                self.eat_optional_semicolon();
                Some(PrepStatement::Continue { line, column })
            }
            Token::Return => {
                let (l, c) = self.current_span();
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::EPhaseTokenForbidden,
                    "return is only allowed inside logic blocks",
                    Phase::Parse,
                    scene,
                    l,
                    c,
                ));
                self.advance();
                if !matches!(self.peek(), Token::Semicolon | Token::RBrace) {
                    let _ = self.parse_expression();
                }
                self.eat_optional_semicolon();
                None
            }
            Token::Ident(_) => self.parse_prep_call_statement(scene),
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

    fn parse_prep_call_statement(&mut self, _scene: &str) -> Option<PrepStatement> {
        let (line, column) = self.current_span();
        let name = if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            name
        } else {
            return None;
        };

        if !self.expect(&Token::LParen) {
            return None;
        }

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

        self.eat_optional_semicolon();

        Some(PrepStatement::Call {
            name,
            args,
            line,
            column,
        })
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

    fn parse_scene_var_decl(&mut self, scene: &str) -> Option<VarDecl> {
        let (line, column) = self.current_span();
        self.advance(); // $

        if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            if !self.expect(&Token::As) {
                return None;
            }
            let var_type = self.parse_var_type(scene)?;
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
            if self.peek() == &Token::If {
                let nested_if = self.parse_prep_if_else(scene)?;
                Some(vec![PrepStatement::IfElse(nested_if)])
            } else {
                if !self.expect(&Token::LBrace) {
                    return None;
                } else {
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
                }
            }
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

    fn parse_repeat_count(&mut self, scene: &str) -> Option<RepeatCount> {
        let (line, column) = self.current_span();
        match self.peek().clone() {
            Token::IntLit(value) => {
                self.advance();
                Some(RepeatCount::IntLiteral {
                    value,
                    line,
                    column,
                })
            }
            Token::Dollar => {
                self.advance();
                if let Token::Ident(name) = self.peek().clone() {
                    self.advance();
                    Some(RepeatCount::Variable { name, line, column })
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        DiagnosticCode::ESyntax,
                        "Expected variable name after '$' in repeat count",
                        Phase::Parse,
                        scene,
                        line,
                        column,
                    ));
                    None
                }
            }
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    DiagnosticCode::ESyntax,
                    "repeat(count) only supports integer literal or $integer_variable",
                    Phase::Parse,
                    scene,
                    line,
                    column,
                ));
                None
            }
        }
    }

    fn parse_for_snapshot_header(
        &mut self,
        scene: &str,
        line: usize,
        column: usize,
    ) -> Option<(String, String)> {
        if !self.expect(&Token::LParen) {
            return None;
        }
        if !self.expect(&Token::Dollar) {
            return None;
        }

        let item_name = if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            name
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected loop item variable after '$'",
                Phase::Parse,
                scene,
                line,
                column,
            ));
            return None;
        };

        if !self.expect(&Token::In) {
            return None;
        }
        if !self.expect(&Token::Snapshot) {
            return None;
        }
        if !self.expect(&Token::Dollar) {
            return None;
        }

        let array_name = if let Token::Ident(name) = self.peek().clone() {
            self.advance();
            name
        } else {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::ESyntax,
                "Expected array variable after 'snapshot $'",
                Phase::Parse,
                scene,
                line,
                column,
            ));
            return None;
        };

        if !self.expect(&Token::RParen) {
            return None;
        }

        Some((item_name, array_name))
    }

    fn parse_prep_for_snapshot(&mut self, scene: &str) -> Option<PrepStatement> {
        let (line, column) = self.current_span();
        self.advance(); // for

        let (item_name, array_name) = self.parse_for_snapshot_header(scene, line, column)?;
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_prep_statement(scene) {
                body.push(stmt);
            } else {
                self.advance();
            }
        }
        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(PrepStatement::ForSnapshot(PrepForSnapshot {
            item_name,
            array_name,
            body,
            line,
            column,
        }))
    }

    fn parse_prep_repeat(&mut self, scene: &str) -> Option<PrepStatement> {
        let (line, column) = self.current_span();
        self.advance(); // repeat

        if !self.expect(&Token::LParen) {
            return None;
        }
        let count = self.parse_repeat_count(scene)?;
        if !self.expect(&Token::RParen) {
            return None;
        }
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_prep_statement(scene) {
                body.push(stmt);
            } else {
                self.advance();
            }
        }
        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(PrepStatement::Repeat(PrepRepeat {
            count,
            body,
            line,
            column,
        }))
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
            Token::For => self.parse_story_for_snapshot(scene),
            Token::Repeat => self.parse_story_repeat(scene),
            Token::Break => {
                let (line, column) = self.current_span();
                self.advance();
                self.eat_optional_semicolon();
                Some(StoryStatement::Break { line, column })
            }
            Token::Continue => {
                let (line, column) = self.current_span();
                self.advance();
                self.eat_optional_semicolon();
                Some(StoryStatement::Continue { line, column })
            }
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

        if self.peek() == &Token::As {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::EPhaseTokenForbidden,
                "Variable declaration is forbidden in #STORY",
                Phase::Parse,
                scene,
                line,
                column,
            ));

            self.advance(); // as
            let _ = self.parse_var_type(scene);
            if self.peek() == &Token::Eq {
                self.advance();
                let _ = self.parse_expression();
            }
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
                | Token::For
                | Token::Repeat
                | Token::Break
                | Token::Continue
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

        let entries = self.parse_choice_entries(scene);

        self.expect(&Token::RBrace);

        Some(StoryStatement::Choice(ChoiceBlock {
            entries,
            line,
            column,
        }))
    }

    fn parse_choice_entries(&mut self, scene: &str) -> Vec<ChoiceEntry> {
        let mut entries = Vec::new();

        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            let start_pos = self.pos;
            if let Some(entry) = self.parse_choice_entry(scene) {
                entries.push(entry);
                continue;
            }

            if self.pos == start_pos {
                self.recover_choice_entry_tail();
            }
            if self.pos == start_pos && !matches!(self.peek(), Token::RBrace | Token::Eof) {
                self.advance();
            }
        }

        entries
    }

    fn parse_choice_entry(&mut self, scene: &str) -> Option<ChoiceEntry> {
        match self.peek().clone() {
            Token::StringLit(_) => self
                .parse_choice_option(scene)
                .map(ChoiceEntry::Option),
            Token::If => self.parse_choice_if_entry(scene),
            Token::Repeat => self.parse_choice_repeat_entry(scene),
            Token::For => self.parse_choice_for_snapshot_entry(scene),
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
                None
            }
        }
    }

    fn parse_choice_option(&mut self, scene: &str) -> Option<ChoiceOption> {
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
            line,
            column,
        })
    }

    fn parse_choice_if_entry(&mut self, scene: &str) -> Option<ChoiceEntry> {
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

        let body = self.parse_choice_entries(scene);
        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(ChoiceEntry::If(ChoiceIfEntry {
            condition,
            body,
            line,
            column,
        }))
    }

    fn parse_choice_repeat_entry(&mut self, scene: &str) -> Option<ChoiceEntry> {
        let (line, column) = self.current_span();
        self.advance(); // repeat

        if !self.expect(&Token::LParen) {
            return None;
        }
        let count = self.parse_repeat_count(scene)?;
        if !self.expect(&Token::RParen) {
            return None;
        }
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let body = self.parse_choice_entries(scene);
        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(ChoiceEntry::Repeat(ChoiceRepeatEntry {
            count,
            body,
            line,
            column,
        }))
    }

    fn parse_choice_for_snapshot_entry(&mut self, scene: &str) -> Option<ChoiceEntry> {
        let (line, column) = self.current_span();
        self.advance(); // for

        let (item_name, array_name) = self.parse_for_snapshot_header(scene, line, column)?;
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let body = self.parse_choice_entries(scene);
        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(ChoiceEntry::ForSnapshot(ChoiceForSnapshotEntry {
            item_name,
            array_name,
            body,
            line,
            column,
        }))
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
            if self.peek() == &Token::If {
                let nested_if = self.parse_story_if_else(scene)?;
                Some(vec![nested_if])
            } else {
                if !self.expect(&Token::LBrace) {
                    return None;
                } else {
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
                }
            }
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

    fn parse_story_for_snapshot(&mut self, scene: &str) -> Option<StoryStatement> {
        let (line, column) = self.current_span();
        self.advance(); // for

        let (item_name, array_name) = self.parse_for_snapshot_header(scene, line, column)?;
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_story_statement(scene) {
                body.push(stmt);
            } else {
                if matches!(self.peek(), Token::RBrace | Token::Eof) {
                    break;
                }
                self.advance();
            }
        }

        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(StoryStatement::ForSnapshot(StoryForSnapshot {
            item_name,
            array_name,
            body,
            line,
            column,
        }))
    }

    fn parse_story_repeat(&mut self, scene: &str) -> Option<StoryStatement> {
        let (line, column) = self.current_span();
        self.advance(); // repeat

        if !self.expect(&Token::LParen) {
            return None;
        }
        let count = self.parse_repeat_count(scene)?;
        if !self.expect(&Token::RParen) {
            return None;
        }
        if !self.expect(&Token::LBrace) {
            return None;
        }

        let mut body = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            if let Some(stmt) = self.parse_story_statement(scene) {
                body.push(stmt);
            } else {
                if matches!(self.peek(), Token::RBrace | Token::Eof) {
                    break;
                }
                self.advance();
            }
        }

        if !self.expect(&Token::RBrace) {
            return None;
        }

        Some(StoryStatement::Repeat(StoryRepeat {
            count,
            body,
            line,
            column,
        }))
    }

    fn recover_choice_entry_tail(&mut self) {
        let mut depth = 0usize;

        while self.peek() != &Token::Eof {
            match self.peek() {
                Token::LBrace => {
                    depth += 1;
                    self.advance();
                }
                Token::RBrace => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    self.advance();
                }
                Token::StringLit(_) | Token::If | Token::Repeat | Token::For if depth == 0 => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
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
