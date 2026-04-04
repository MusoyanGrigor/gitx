use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io, time::Duration};
use crate::core::GitRepo;
use crate::models::{CommitInfo, LabelInfo};

// --- [ Style Constants ] ---
const COLOR_HEAD: Color = Color::Cyan;
const COLOR_LOCAL_BRANCH: Color = Color::Green;
const COLOR_REMOTE_BRANCH: Color = Color::Red;
const COLOR_TAG: Color = Color::Yellow;
const COLOR_HASH: Color = Color::DarkGray;
const COLOR_ERROR: Color = Color::LightRed;

pub fn run_tree_explorer(repo: GitRepo, filter: Option<String>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(repo, filter)?;
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Fatal Error: {:?}", err);
    }

    Ok(())
}

enum InputMode {
    Normal,
    Search,
    Jump,
}

struct App {
    repo: GitRepo,
    commits: Vec<CommitInfo>,
    state: ListState,
    detail_visible: bool,
    input_mode: InputMode,
    input_buffer: String,
    status_message: Option<(String, Color)>,
    active_filter: Option<String>,
}

impl App {
    fn new(repo: GitRepo, filter: Option<String>) -> Result<Self> {
        let commits = if let Some(ref q) = filter {
            repo.filter_commits(q)?
        } else {
            repo.get_commits(100)?
        };
        let mut state = ListState::default();
        state.select(Some(0));
        Ok(Self {
            repo,
            commits,
            state,
            detail_visible: true,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: None,
            active_filter: filter,
        })
    }

    fn refresh_commits(&mut self) -> Result<()> {
        self.commits = if let Some(ref q) = self.active_filter {
            self.repo.filter_commits(q)?
        } else {
            self.repo.get_commits(100)?
        };
        self.state.select(Some(0));
        Ok(())
    }

    fn next(&mut self) {
        if self.commits.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i >= self.commits.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.commits.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i == 0 { self.commits.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn toggle_detail(&mut self) {
        self.detail_visible = !self.detail_visible;
    }

    fn jump_to_ref(&mut self, reference: &str) {
        match self.repo.resolve_ref(reference) {
            Ok(hash) => {
                if let Some(index) = self.commits.iter().position(|c| c.hash == hash || c.hash.starts_with(&hash)) {
                    self.state.select(Some(index));
                    self.status_message = Some((format!("Jumped to {}", &hash[..7]), Color::Gray));
                } else {
                    self.status_message = Some((format!("Ref resolved to {} but not in current list", &hash[..7]), COLOR_ERROR));
                }
            }
            Err(_) => {
                self.status_message = Some((format!("Reference '{}' not found", reference), COLOR_ERROR));
            }
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char('d') => app.toggle_detail(),
                        KeyCode::Char('/') | KeyCode::Char('f') => {
                            app.input_mode = InputMode::Search;
                            app.input_buffer.clear();
                        }
                        KeyCode::Char('J') => {
                            app.input_mode = InputMode::Jump;
                            app.input_buffer.clear();
                        }
                        KeyCode::Esc => {
                            app.active_filter = None;
                            let _ = app.refresh_commits();
                        }
                        _ => {}
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Enter => {
                            app.active_filter = if app.input_buffer.is_empty() { None } else { Some(app.input_buffer.clone()) };
                            let _ = app.refresh_commits();
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => app.input_buffer.push(c),
                        KeyCode::Backspace => { app.input_buffer.pop(); }
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Jump => match key.code {
                        KeyCode::Enter => {
                            let target = app.input_buffer.clone();
                            app.jump_to_ref(&target);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => app.input_buffer.push(c),
                        KeyCode::Backspace => { app.input_buffer.pop(); }
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.size());

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            if app.detail_visible {
                [Constraint::Percentage(50), Constraint::Percentage(50)]
            } else {
                [Constraint::Percentage(100), Constraint::Percentage(0)]
            }
        )
        .split(main_chunks[0]);

    render_commit_list(f, app, body_chunks[0]);

    if app.detail_visible {
        render_commit_detail(f, app, body_chunks[1]);
    }

    render_status_bar(f, app, main_chunks[1]);
}

fn render_commit_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.commits.iter().map(|c| {
        let hash_span = Span::styled(
            format!("[{}] ", if c.hash.len() > 7 { &c.hash[..7] } else { &c.hash }),
            Style::default().fg(COLOR_HASH)
        );
        
        let mut spans = vec![hash_span];
        
        for label in &c.labels {
            let (text, color) = match label {
                LabelInfo::Head(n) => (n, COLOR_HEAD),
                LabelInfo::LocalBranch(n) => (n, COLOR_LOCAL_BRANCH),
                LabelInfo::RemoteBranch(n) => (n, COLOR_REMOTE_BRANCH),
                LabelInfo::Tag(n) => (n, COLOR_TAG),
            };
            spans.push(Span::styled(format!("({}) ", text), Style::default().fg(color).add_modifier(Modifier::BOLD)));
        }

        spans.push(Span::raw(&c.subject));
        
        ListItem::new(Line::from(spans))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Commit Tree "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.state);
}

fn render_commit_detail(f: &mut Frame, app: &mut App, area: Rect) {
    let i = app.state.selected().unwrap_or(0);
    if let Some(commit) = app.commits.get(i) {
        let text = vec![
            Line::from(vec![Span::styled("Hash:   ", Style::default().fg(Color::Gray)), Span::raw(&commit.hash)]),
            Line::from(vec![Span::styled("Author: ", Style::default().fg(Color::Gray)), Span::raw(&commit.author)]),
            Line::from(vec![Span::styled("TS:     ", Style::default().fg(Color::Gray)), Span::raw(commit.date.to_string())]),
            Line::from(""),
            Line::from(vec![Span::styled("Subject: ", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(commit.subject.as_str()),
            Line::from(""),
            Line::from(vec![Span::styled("Body:", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(commit.body.as_deref().unwrap_or("[No body text]")),
        ];

        let p = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Commit Detail "))
            .wrap(Wrap { trim: true });
        f.render_widget(p, area);
    }
}

fn render_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    let (prompt, color) = match app.input_mode {
        InputMode::Normal => {
            if let Some((msg, col)) = &app.status_message {
                (msg.clone(), *col)
            } else if let Some(filter) = &app.active_filter {
                (format!("Filter: {}", filter), Color::Cyan)
            } else {
                ("q:quit | /:filter | J:jump | d:detail | Esc:clear filter".to_string(), Color::DarkGray)
            }
        }
        InputMode::Search => (format!("Search: {}_", app.input_buffer), Color::Yellow),
        InputMode::Jump => (format!("Jump to (branch/tag/hash): {}_", app.input_buffer), Color::Magenta),
    };

    let p = Paragraph::new(prompt)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(color));
    f.render_widget(p, area);
}
