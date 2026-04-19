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

use storyscript_player::{StepResult, StoryPlayer, Value};

// ===========================================================================
// Player state
// ===========================================================================

struct App {
    player: StoryPlayer,
    scroll_offset: u16,
}

impl App {
    fn new(player: StoryPlayer) -> Self {
        App {
            player,
            scroll_offset: 0,
        }
    }

    fn current(&self) -> Option<&StepResult> {
        self.player.current()
    }

    fn history(&self) -> &[StepResult] {
        self.player.history()
    }

    fn script_name(&self) -> &str {
        self.player.script_name()
    }

    fn advance(&mut self) {
        self.player.advance();
        self.scroll_offset = 0;
    }

    fn select_choice(&mut self, index: usize) {
        if self.player.select_choice(index) {
            self.scroll_offset = 0;
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
                            match player.current() {
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
        format!("File: {}", app.script_name()),
        format!("Scene: {}", app.player.engine().current_scene),
    ];
    if let Some(ref bg) = app.player.engine().bg {
        parts.push(format!("BG: {}", bg));
    }
    if let Some(ref bgm) = app.player.engine().bgm {
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
    for entry in app.history() {
        append_step_lines(&mut lines, entry, true);
        lines.push(Line::from(""));
    }

    // Current (bright)
    if let Some(current) = app.current() {
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
    let mut var_pairs: Vec<(&String, &Value)> = app.player.engine().variables.iter().collect();
    var_pairs.sort_by(|a, b| a.0.cmp(b.0));

    let var_display: String = var_pairs
        .iter()
        .map(|(k, v)| format!("${}={}", k, v))
        .collect::<Vec<_>>()
        .join("  ");

    let controls = match app.current() {
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
        let body_chunks = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(chunks[1]);

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
        frame.render_stateful_widget(list, body_chunks[0], &mut state);

        let side_title = if chooser.message.is_some() {
            " Compile Errors "
        } else {
            " Details "
        };
        let side_text = chooser.message.clone().unwrap_or_else(|| {
            "Press Enter to load selected file.\n\nIf the selected file has compile errors, full diagnostics will appear here."
                .to_string()
        });
        let side_style = if chooser.message.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let side_panel = Paragraph::new(side_text)
            .style(side_style)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title(side_title));
        frame.render_widget(side_panel, body_chunks[1]);
    }

    let selected_text = chooser
        .selected_path()
        .map(|p| format!("Selected: {}", display_path(&p)))
        .unwrap_or_else(|| "Selected: (none)".to_string());
    let message_text = chooser
        .message
        .clone()
        .map(|_| "[Enter] Retry  [↑↓] Select  [Q] Quit".to_string())
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
    let player = StoryPlayer::from_file(path)?;
    Ok(App::new(player))
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
