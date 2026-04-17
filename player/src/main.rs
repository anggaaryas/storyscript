mod engine;

use std::env;
use std::fs;
use std::io;
use std::process;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::*,
};

use engine::{Engine, StepResult, Value};

// ===========================================================================
// App state
// ===========================================================================

struct App {
    engine: Engine,
    history: Vec<StepResult>,
    current: Option<StepResult>,
    scroll_offset: u16,
}

impl App {
    fn new(engine: Engine) -> Self {
        let mut app = App {
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
// Entry point
// ===========================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: storycript-player <file.StoryScript>");
        process::exit(1);
    }

    let source = fs::read_to_string(&args[1])?;

    // Lex
    let mut lexer = storycript_parser::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize();

    if lexer.diagnostics.iter().any(|d| d.is_error()) {
        eprintln!("Lexer errors:");
        for d in &lexer.diagnostics {
            eprintln!("  {}", d);
        }
        process::exit(1);
    }

    // Parse
    let mut parser = storycript_parser::parser::Parser::new(tokens);
    let script = match parser.parse() {
        Some(s) => s,
        None => {
            eprintln!("Parse errors:");
            for d in &parser.diagnostics {
                eprintln!("  {}", d);
            }
            process::exit(1);
        }
    };

    if parser.diagnostics.iter().any(|d| d.is_error()) {
        eprintln!("Parse errors:");
        for d in &parser.diagnostics {
            eprintln!("  {}", d);
        }
        process::exit(1);
    }

    // Validate
    let validation_diags = storycript_parser::validator::validate(&script);
    if validation_diags.iter().any(|d| d.is_error()) {
        eprintln!("Validation errors:");
        for d in &validation_diags {
            eprintln!("  {}", d);
        }
        process::exit(1);
    }

    // Boot engine
    let engine = Engine::new(&script);
    let mut app = App::new(engine);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run
    let result = run_app(&mut terminal, &mut app);

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
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                KeyCode::Enter => match &app.current {
                    Some(StepResult::Narration(_)) | Some(StepResult::Dialogue { .. }) => {
                        app.advance();
                    }
                    Some(StepResult::End) => return Ok(()),
                    _ => {}
                },
                KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                    let index = (c as u8 - b'1') as usize;
                    app.select_choice(index);
                }
                KeyCode::Up => {
                    app.scroll_offset = app.scroll_offset.saturating_add(1);
                }
                KeyCode::Down => {
                    app.scroll_offset = app.scroll_offset.saturating_sub(1);
                }
                _ => {}
            }
        }
    }
}

// ===========================================================================
// Rendering
// ===========================================================================

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3), // header
        Constraint::Min(5),   // content
        Constraint::Length(4), // footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mut parts: Vec<String> = vec![format!("Scene: {}", app.engine.current_scene)];
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
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    );

    frame.render_widget(header, area);
}

// ---------------------------------------------------------------------------
// Content (scrolling text log)
// ---------------------------------------------------------------------------

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
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
    let dim = if dimmed { Modifier::DIM } else { Modifier::empty() };

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

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
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
            format!("[1-{}] Choose  [Q] Quit", choices.len())
        }
        Some(StepResult::End) => "[Enter/Q] Quit".to_string(),
        _ => "[Enter] Continue  [↑↓] Scroll  [Q] Quit".to_string(),
    };

    let footer_text = Text::from(vec![
        Line::from(Span::styled(
            var_display,
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            controls,
            Style::default().fg(Color::Green),
        )),
    ]);

    let footer = Paragraph::new(footer_text).block(
        Block::bordered()
            .title(" State ")
            .title_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(footer, area);
}

// ===========================================================================
// Helpers
// ===========================================================================

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
