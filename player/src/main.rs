mod engine;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};

use engine::{Engine, StepResult, Value};

// ===========================================================================
// Player state
// ===========================================================================

struct App {
    script_name: String,
    engine: Engine,
    history: Vec<StepResult>,
    current: Option<StepResult>,
    scroll_offset: u16,
}

impl App {
    fn new(script_name: String, engine: Engine) -> Self {
        let mut app = App {
            script_name,
            engine,
            history: Vec::new(),
            current: None,
            scroll_offset: 0,
        };
        app.advance();
        app
    }

    fn advance(&mut self) {
        if let Some(current) = self.current.take() {
            self.history.push(current);
        }
        match self.engine.step() {
            Some(result) => self.current = Some(result),
            None => self.current = Some(StepResult::End),
        }
        self.scroll_offset = 0;
    }

    fn select_choice(&mut self, index: usize) {
        if let Some(StepResult::Choices(ref choices)) = self.current {
            if index < choices.len() {
                let choice = choices[index].clone();
                // Move current choices display to history
                if let Some(current) = self.current.take() {
                    self.history.push(current);
                }
                // Record the player's selection
                self.history
                    .push(StepResult::Narration(format!("▸ {}", choice.text)));
                // Enter target scene
                self.engine.select_choice(&choice);
                // Advance to next display event
                match self.engine.step() {
                    Some(result) => self.current = Some(result),
                    None => self.current = Some(StepResult::End),
                }
                self.scroll_offset = 0;
            }
        }
    }
}

// ===========================================================================
// File chooser state
// ===========================================================================

struct FileChooser {
    files: Vec<PathBuf>,
    selected: usize,
    message: Option<String>,
}

impl FileChooser {
    fn new(files: Vec<PathBuf>) -> Self {
        Self {
            files,
            selected: 0,
            message: None,
        }
    }

    fn move_up(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
    }

    fn move_down(&mut self) {
        if self.files.is_empty() {
            return;
        }
        let max_index = self.files.len().saturating_sub(1);
        self.selected = (self.selected + 1).min(max_index);
    }

    fn selected_path(&self) -> Option<PathBuf> {
        self.files.get(self.selected).cloned()
    }
}

// ===========================================================================
// Root state
// ===========================================================================

struct RootApp {
    chooser: FileChooser,
    player: Option<App>,
}

impl RootApp {
    fn in_player_mode(&self) -> bool {
        self.player.is_some()
    }
}

// ===========================================================================
// Entry point
// ===========================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let files = discover_story_files(args.get(1).map(String::as_str));
    let mut root_app = RootApp {
        chooser: FileChooser::new(files),
        player: None,
    };

    if root_app.chooser.files.is_empty() {
        root_app.chooser.message = Some(
            "No .StoryScript files found. Put one in the current directory or ../example."
                .to_string(),
        );
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run
    let result = run_app(&mut terminal, &mut root_app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    Ok(())
}

// ===========================================================================
// Main loop
// ===========================================================================

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    root_app: &mut RootApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            if let Some(player) = root_app.player.as_ref() {
                render_player(frame, player);
            } else {
                render_file_chooser(frame, &root_app.chooser);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if root_app.in_player_mode() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        root_app.player = None;
                    }
                    KeyCode::Enter => {
                        if let Some(player) = root_app.player.as_mut() {
                            match &player.current {
                                Some(StepResult::Narration(_))
                                | Some(StepResult::Dialogue { .. }) => player.advance(),
                                Some(StepResult::End) | Some(StepResult::Choices(_)) | None => {}
                            }
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                        if let Some(player) = root_app.player.as_mut() {
                            let index = (c as u8 - b'1') as usize;
                            player.select_choice(index);
                        }
                    }
                    KeyCode::Up => {
                        if let Some(player) = root_app.player.as_mut() {
                            player.scroll_offset = player.scroll_offset.saturating_add(1);
                        }
                    }
                    KeyCode::Down => {
                        if let Some(player) = root_app.player.as_mut() {
                            player.scroll_offset = player.scroll_offset.saturating_sub(1);
                        }
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    KeyCode::Up => root_app.chooser.move_up(),
                    KeyCode::Down => root_app.chooser.move_down(),
                    KeyCode::Enter => {
                        if let Some(path) = root_app.chooser.selected_path() {
                            match load_player_from_file(&path) {
                                Ok(player) => {
                                    root_app.player = Some(player);
                                    root_app.chooser.message = None;
                                }
                                Err(err) => {
                                    root_app.chooser.message = Some(err);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// ===========================================================================
// Rendering
// ===========================================================================

fn render_player(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3), // header
        Constraint::Min(5),    // content
        Constraint::Length(4), // footer
    ])
    .split(area);

    render_player_header(frame, app, chunks[0]);
    render_player_content(frame, app, chunks[1]);
    render_player_footer(frame, app, chunks[2]);
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn render_player_header(frame: &mut Frame, app: &App, area: Rect) {
    let mut parts: Vec<String> = vec![
        format!("File: {}", app.script_name),
        format!("Scene: {}", app.engine.current_scene),
    ];
    if let Some(ref bg) = app.engine.bg {
        parts.push(format!("BG: {}", bg));
    }
    if let Some(ref bgm) = app.engine.bgm {
        parts.push(format!("BGM: {}", bgm));
    }
    let info = parts.join(" │ ");

    let header = Paragraph::new(Line::from(Span::styled(
        info,
        Style::default().fg(Color::Cyan),
    )))
    .block(
        Block::bordered()
            .title(" ▶ StoryScript Player ")
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    frame.render_widget(header, area);
}

// ---------------------------------------------------------------------------
// Content (scrolling text log)
// ---------------------------------------------------------------------------

fn render_player_content(frame: &mut Frame, app: &App, area: Rect) {
    let lines = build_content_lines(app);
    let wrap_width = area.width.saturating_sub(2).max(1) as usize;
    let total_lines = estimate_wrapped_line_count(&lines, wrap_width);
    let visible = area.height.saturating_sub(2); // subtract borders
    let max_scroll = total_lines.saturating_sub(visible);
    let scroll = max_scroll.saturating_sub(app.scroll_offset);

    let content = Paragraph::new(Text::from(lines))
        .block(Block::bordered())
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(content, area);
}

fn estimate_wrapped_line_count(lines: &[Line<'static>], wrap_width: usize) -> u16 {
    lines.iter().fold(0u16, |acc, line| {
        let content_width = line.to_string().chars().count();
        let visual_lines = if content_width == 0 {
            1usize
        } else {
            (content_width + wrap_width - 1) / wrap_width
        };
        acc.saturating_add(visual_lines as u16)
    })
}

fn build_content_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // History (dimmed)
    for entry in &app.history {
        append_step_lines(&mut lines, entry, true);
        lines.push(Line::from(""));
    }

    // Current (bright)
    if let Some(ref current) = app.current {
        append_step_lines(&mut lines, current, false);
    }

    lines
}

fn append_step_lines(lines: &mut Vec<Line<'static>>, result: &StepResult, dimmed: bool) {
    let dim = if dimmed {
        Modifier::DIM
    } else {
        Modifier::empty()
    };

    match result {
        StepResult::Narration(text) => {
            if text.starts_with("───") {
                // Scene header
                lines.push(Line::from(Span::styled(
                    text.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | dim),
                )));
            } else if text.starts_with('▸') {
                // Player's choice marker
                lines.push(Line::from(Span::styled(
                    format!("  {}", text),
                    Style::default().fg(Color::Green).add_modifier(dim),
                )));
            } else {
                // Regular narration
                lines.push(Line::from(Span::styled(
                    format!("  {}", text),
                    Style::default().fg(Color::White).add_modifier(dim),
                )));
            }
        }
        StepResult::Dialogue {
            actor_name,
            actor_id,
            emotion,
            position,
            text,
        } => {
            let color = actor_color(actor_id);

            // Actor header line
            let mut spans: Vec<Span<'static>> = vec![Span::styled(
                format!("  {}", actor_name),
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD | dim),
            )];

            if let Some(em) = emotion {
                spans.push(Span::styled(
                    format!(" ({})", em),
                    Style::default().fg(color).add_modifier(dim),
                ));
            }
            if let Some(pos) = position {
                spans.push(Span::styled(
                    format!(" [{}]", pos),
                    Style::default().fg(Color::DarkGray).add_modifier(dim),
                ));
            }
            spans.push(Span::styled(
                ":".to_string(),
                Style::default().fg(color).add_modifier(dim),
            ));
            lines.push(Line::from(spans));

            // Dialogue text
            lines.push(Line::from(Span::styled(
                format!("  \"{}\"", text),
                Style::default().fg(Color::White).add_modifier(dim),
            )));
        }
        StepResult::Choices(choices) => {
            lines.push(Line::from(Span::styled(
                "  Choose:".to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | dim),
            )));
            lines.push(Line::from(""));
            for (i, choice) in choices.iter().enumerate() {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("    [{}] ", i + 1),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD | dim),
                    ),
                    Span::styled(
                        choice.text.clone(),
                        Style::default().fg(Color::White).add_modifier(dim),
                    ),
                ]));
            }
        }
        StepResult::End => {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ═══════════════════════════════════════".to_string(),
                Style::default().fg(Color::Yellow),
            )));
            lines.push(Line::from(Span::styled(
                "            T H E   E N D".to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "  ═══════════════════════════════════════".to_string(),
                Style::default().fg(Color::Yellow),
            )));
        }
    }
}

// ---------------------------------------------------------------------------
// Footer (variables + controls)
// ---------------------------------------------------------------------------

fn render_player_footer(frame: &mut Frame, app: &App, area: Rect) {
    // Sort variables by name for consistent display
    let mut var_pairs: Vec<(&String, &Value)> = app.engine.variables.iter().collect();
    var_pairs.sort_by(|a, b| a.0.cmp(b.0));

    let var_display: String = var_pairs
        .iter()
        .map(|(k, v)| format!("${}={}", k, v))
        .collect::<Vec<_>>()
        .join("  ");

    let controls = match &app.current {
        Some(StepResult::Choices(choices)) => {
            format!("[1-{}] Choose  [Q] Back To File Chooser", choices.len())
        }
        Some(StepResult::End) => "[Q] Back To File Chooser".to_string(),
        _ => "[Enter] Continue  [↑↓] Scroll  [Q] Back To File Chooser".to_string(),
    };

    let footer_text = Text::from(vec![
        Line::from(Span::styled(
            var_display,
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(controls, Style::default().fg(Color::Green))),
    ]);

    let footer = Paragraph::new(footer_text).block(
        Block::bordered()
            .title(" State ")
            .title_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(footer, area);
}

fn render_file_chooser(frame: &mut Frame, chooser: &FileChooser) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(4),
    ])
    .split(area);

    let header = Paragraph::new(Line::from(Span::styled(
        "Choose a .StoryScript file",
        Style::default().fg(Color::Cyan),
    )))
    .block(
        Block::bordered()
            .title(" StoryScript File Chooser ")
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
    );
    frame.render_widget(header, chunks[0]);

    if chooser.files.is_empty() {
        let empty = Paragraph::new("No .StoryScript files found.")
            .block(Block::bordered())
            .alignment(Alignment::Center);
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = chooser
            .files
            .iter()
            .map(|p| ListItem::new(display_path(p)))
            .collect();

        let list = List::new(items)
            .block(Block::bordered().title(" Files "))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(chooser.selected));
        frame.render_stateful_widget(list, chunks[1], &mut state);
    }

    let selected_text = chooser
        .selected_path()
        .map(|p| format!("Selected: {}", display_path(&p)))
        .unwrap_or_else(|| "Selected: (none)".to_string());
    let message_text = chooser
        .message
        .clone()
        .unwrap_or_else(|| "[Enter] Play  [↑↓] Select  [Q] Quit".to_string());

    let footer = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(
            selected_text,
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            message_text,
            Style::default().fg(Color::Green),
        )),
    ]))
    .block(
        Block::bordered()
            .title(" Controls ")
            .title_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(footer, chunks[2]);
}

// ===========================================================================
// Helpers
// ===========================================================================

fn discover_story_files(arg_path: Option<&str>) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Some(p) = arg_path {
        let path = PathBuf::from(p);
        if path.is_file() {
            if is_storyscript_file(&path) {
                files.push(path);
            }
        } else if path.is_dir() {
            collect_story_files(&path, &mut files);
        }
    }

    if files.is_empty() {
        if let Ok(cwd) = env::current_dir() {
            collect_story_files(&cwd, &mut files);
            collect_story_files(&cwd.join("../example"), &mut files);
        }
    }

    files.sort();
    files.dedup();
    files
}

fn collect_story_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(v) => v,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_story_files(&path, out);
        } else if is_storyscript_file(&path) {
            out.push(path);
        }
    }
}

fn is_storyscript_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "StoryScript")
}

fn display_path(path: &Path) -> String {
    if let Ok(cwd) = env::current_dir() {
        if let Ok(relative) = path.strip_prefix(&cwd) {
            return relative.display().to_string();
        }
    }
    path.display().to_string()
}

fn load_player_from_file(path: &Path) -> Result<App, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", display_path(path), e))?;

    // Lex
    let mut lexer = storycript_parser::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize();
    if lexer.diagnostics.iter().any(|d| d.is_error()) {
        return Err(format_diagnostics(
            &format!("Lexer errors in {}", display_path(path)),
            &lexer
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        ));
    }

    // Parse
    let mut parser = storycript_parser::parser::Parser::new(tokens);
    let script = parser.parse().ok_or_else(|| {
        format_diagnostics(
            &format!("Parse errors in {}", display_path(path)),
            &parser
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        )
    })?;

    if parser.diagnostics.iter().any(|d| d.is_error()) {
        return Err(format_diagnostics(
            &format!("Parse errors in {}", display_path(path)),
            &parser
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        ));
    }

    // Validate
    let validation_diags = storycript_parser::validator::validate(&script);
    if validation_diags.iter().any(|d| d.is_error()) {
        return Err(format_diagnostics(
            &format!("Validation errors in {}", display_path(path)),
            &validation_diags
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        ));
    }

    let engine = Engine::new(&script);
    Ok(App::new(display_path(path), engine))
}

fn format_diagnostics(title: &str, diags: &[String]) -> String {
    if diags.is_empty() {
        return title.to_string();
    }
    let details = diags
        .iter()
        .take(2)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" | ");
    format!("{}: {}", title, details)
}

fn actor_color(actor_id: &str) -> Color {
    let hash: u32 = actor_id
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));

    const COLORS: [Color; 8] = [
        Color::Yellow,
        Color::Green,
        Color::Cyan,
        Color::Magenta,
        Color::LightBlue,
        Color::LightRed,
        Color::LightGreen,
        Color::LightMagenta,
    ];

    COLORS[(hash as usize) % COLORS.len()]
}
