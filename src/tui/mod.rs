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
use crate::utils::tui_graph::{TuiGraphRenderer, GraphRow};

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

enum InputMode { Normal, Search, Jump }

struct App {
    repo: GitRepo,
    graph_rows: Vec<GraphRow>,
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
        
        let mut renderer = TuiGraphRenderer::new();
        let graph_rows = renderer.compute_rows(&commits);
        
        let mut state = ListState::default();
        state.select(Some(0));
        
        Ok(Self {
            repo,
            graph_rows,
            state,
            detail_visible: true,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: None,
            active_filter: filter,
        })
    }

    fn refresh_commits(&mut self) -> Result<()> {
        let commits = if let Some(ref q) = self.active_filter {
            self.repo.filter_commits(q)?
        } else {
            self.repo.get_commits(100)?
        };
        let mut renderer = TuiGraphRenderer::new();
        self.graph_rows = renderer.compute_rows(&commits);
        self.state.select(Some(0));
        Ok(())
    }

    fn next(&mut self) {
        if self.graph_rows.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i >= self.graph_rows.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.graph_rows.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i == 0 { self.graph_rows.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn page_down(&mut self) {
        if self.graph_rows.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => (i + 15).min(self.graph_rows.len() - 1),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn page_up(&mut self) {
        if self.graph_rows.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(15),
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn jump_to_ref(&mut self, reference: &str) {
        match self.repo.resolve_ref(reference) {
            Ok(hash) => {
                if let Some(index) = self.graph_rows.iter().position(|r| r.commit.hash == hash || r.commit.hash.starts_with(&hash)) {
                    self.state.select(Some(index));
                    self.status_message = Some((format!("Jumped to {}", &hash[..7]), Color::Gray));
                } else {
                    self.status_message = Some((format!("Ref {} not found in current view", &hash[..7]), Color::Red));
                }
            }
            Err(_) => {
                self.status_message = Some((format!("Reference '{}' not found", reference), Color::Red));
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
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::PageDown => app.page_down(),
                        KeyCode::PageUp => app.page_up(),
                        KeyCode::Home => app.state.select(Some(0)),
                        KeyCode::End => app.state.select(Some(app.graph_rows.len().saturating_sub(1))),
                        KeyCode::Char('d') => app.detail_visible = !app.detail_visible,
                        KeyCode::Char('/') | KeyCode::Char('f') => {
                            app.input_mode = InputMode::Search;
                            app.input_buffer.clear();
                        }
                        KeyCode::Char('J') => {
                            app.input_mode = InputMode::Jump;
                            app.input_buffer.clear();
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
        .constraints(if app.detail_visible { [Constraint::Percentage(60), Constraint::Percentage(40)] } else { [Constraint::Percentage(100), Constraint::Percentage(0)] })
        .split(main_chunks[0]);

    render_commit_list(f, app, body_chunks[0]);
    if app.detail_visible {
        render_commit_detail(f, app, body_chunks[1]);
    }
    render_status_bar(f, app, main_chunks[1]);
}

fn render_commit_list(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_idx = app.state.selected().unwrap_or(0);
    let items: Vec<ListItem> = app.graph_rows.iter().enumerate().map(|(idx, r)| {
        ListItem::new(r.render(idx == selected_idx))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Commit Graph "))
        .highlight_style(Style::default().bg(Color::Indexed(237)).add_modifier(Modifier::BOLD)) // Highlight background like IDE
        .highlight_symbol("→ ");

    f.render_stateful_widget(list, area, &mut app.state);
}

fn render_commit_detail(f: &mut Frame, app: &mut App, area: Rect) {
    let i = app.state.selected().unwrap_or(0);
    if let Some(row) = app.graph_rows.get(i) {
        let commit = &row.commit;
        let text = vec![
            Line::from(vec![Span::styled("Hash:   ", Style::default().fg(Color::DarkGray)), Span::raw(&commit.hash)]),
            Line::from(vec![Span::styled("Author: ", Style::default().fg(Color::DarkGray)), Span::raw(&commit.author)]),
            Line::from(vec![Span::styled("Date:   ", Style::default().fg(Color::DarkGray)), Span::raw(commit.date.to_string())]),
            Line::from(""),
            Line::from(vec![Span::styled("Subject: ", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(commit.subject.as_str()),
            Line::from(""),
            Line::from(vec![Span::styled("Body:", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(commit.body.as_deref().unwrap_or("[No body text]")),
        ];

        let p = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Commit Info "))
            .wrap(Wrap { trim: true });
        f.render_widget(p, area);
    }
}

fn render_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    let (prompt, color) = match app.input_mode {
        InputMode::Normal => {
            if let Some((msg, col)) = &app.status_message { (msg.clone(), *col) }
            else if let Some(filter) = &app.active_filter { (format!("Filter: {}", filter), Color::Cyan) }
            else { ("q:quit | arrows/jk:scroll | PgUp/PgDn:scroll | Home/End | d:detail | /:filter | J:jump".to_string(), Color::DarkGray) }
        }
        InputMode::Search => (format!("Search index: {}_", app.input_buffer), Color::Yellow),
        InputMode::Jump => (format!("Jump to ref: {}_", app.input_buffer), Color::Magenta),
    };

    let p = Paragraph::new(prompt)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(color));
    f.render_widget(p, area);
}
