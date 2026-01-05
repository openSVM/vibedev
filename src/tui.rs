// TUI module - Real-time monitoring dashboard (btop-style)
use crate::analyzer::ConversationAnalyzer;
use crate::claude_code_parser::ClaudeCodeParser;
use crate::discovery::LogDiscovery;
use crate::models::DiscoveryFindings;
use crate::viral_insights::{ViralAnalyzer, ViralInsights};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, Paragraph, Row, Sparkline, Table, Tabs,
    },
    Frame, Terminal,
};
use std::collections::{HashMap, VecDeque};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const UPDATE_INTERVAL_MS: u64 = 500; // 500ms updates for smoother feel
const HISTORY_SIZE: usize = 120; // Keep 120 samples (1 minute at 500ms)

/// Real-time metrics tracker with extended history
#[derive(Debug, Clone)]
pub struct MetricsHistory {
    pub timestamps: VecDeque<u64>,
    pub storage: VecDeque<u64>,
    pub conversations: VecDeque<usize>,
    pub messages: VecDeque<usize>,
    pub tokens: VecDeque<u64>,
    pub cost: VecDeque<f64>,
    pub files: VecDeque<usize>,
    pub active_sessions: VecDeque<usize>,
}

impl MetricsHistory {
    fn new() -> Self {
        Self {
            timestamps: VecDeque::with_capacity(HISTORY_SIZE),
            storage: VecDeque::with_capacity(HISTORY_SIZE),
            conversations: VecDeque::with_capacity(HISTORY_SIZE),
            messages: VecDeque::with_capacity(HISTORY_SIZE),
            tokens: VecDeque::with_capacity(HISTORY_SIZE),
            cost: VecDeque::with_capacity(HISTORY_SIZE),
            files: VecDeque::with_capacity(HISTORY_SIZE),
            active_sessions: VecDeque::with_capacity(HISTORY_SIZE),
        }
    }

    fn push(&mut self, snapshot: MetricsSnapshot) {
        if self.timestamps.len() >= HISTORY_SIZE {
            self.timestamps.pop_front();
            self.storage.pop_front();
            self.conversations.pop_front();
            self.messages.pop_front();
            self.tokens.pop_front();
            self.cost.pop_front();
            self.files.pop_front();
            self.active_sessions.pop_front();
        }

        self.timestamps.push_back(snapshot.timestamp);
        self.storage.push_back(snapshot.storage);
        self.conversations.push_back(snapshot.conversations);
        self.messages.push_back(snapshot.messages);
        self.tokens.push_back(snapshot.tokens);
        self.cost.push_back(snapshot.cost);
        self.files.push_back(snapshot.files);
        self.active_sessions.push_back(snapshot.active_sessions);
    }

    fn get_sparkline_data(&self, metric: MetricType, samples: usize) -> Vec<u64> {
        let data = match metric {
            MetricType::Storage => &self.storage,
            MetricType::Conversations => {
                return self.conversations.iter().map(|&x| x as u64).collect()
            }
            MetricType::Messages => return self.messages.iter().map(|&x| x as u64).collect(),
            MetricType::Tokens => &self.tokens,
            MetricType::Cost => return self.cost.iter().map(|&x| (x * 100.0) as u64).collect(),
            MetricType::Files => return self.files.iter().map(|&x| x as u64).collect(),
            MetricType::ActiveSessions => {
                return self.active_sessions.iter().map(|&x| x as u64).collect()
            }
        };

        let len = data.len();
        if len <= samples {
            data.iter().cloned().collect()
        } else {
            data.iter().skip(len - samples).cloned().collect()
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MetricType {
    Storage,
    Conversations,
    Messages,
    Tokens,
    Cost,
    Files,
    ActiveSessions,
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub storage: u64,
    pub conversations: usize,
    pub messages: usize,
    pub tokens: u64,
    pub cost: f64,
    pub files: usize,
    pub active_sessions: usize,
}

/// Real-time monitoring app with enhanced visuals
pub struct App {
    pub findings: Option<DiscoveryFindings>,
    pub insights: Option<ViralInsights>,
    pub base_dir: PathBuf,
    pub status_message: String,
    pub tool_sizes: HashMap<String, u64>,
    pub estimated_tokens: u64,
    pub estimated_cost: f64,
    pub total_conversations: usize,
    pub total_messages: usize,
    pub total_files: usize,
    pub active_sessions: usize,
    pub history: MetricsHistory,
    pub last_update: Instant,
    pub update_count: u64,
    pub paused: bool,
    pub selected_tab: usize,
    pub achievements_unlocked: usize,
    pub hourly_activity: [u64; 24],
    pub start_time: Instant,
    pub uptime: Duration,
    pub peak_storage: u64,
    pub peak_tokens: u64,
    pub peak_active: usize,
}

impl App {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            findings: None,
            insights: None,
            base_dir,
            status_message: "Initializing vibedev monitor...".to_string(),
            tool_sizes: HashMap::new(),
            estimated_tokens: 0,
            estimated_cost: 0.0,
            total_conversations: 0,
            total_messages: 0,
            total_files: 0,
            active_sessions: 0,
            history: MetricsHistory::new(),
            last_update: Instant::now(),
            update_count: 0,
            paused: false,
            selected_tab: 0,
            achievements_unlocked: 0,
            hourly_activity: [0; 24],
            start_time: Instant::now(),
            uptime: Duration::ZERO,
            peak_storage: 0,
            peak_tokens: 0,
            peak_active: 0,
        }
    }

    pub fn update(&mut self) -> Result<()> {
        if self.paused {
            return Ok(());
        }

        self.uptime = self.start_time.elapsed();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Scan for changes
        let discovery = LogDiscovery::new(self.base_dir.clone(), true);
        let findings = discovery.scan()?;

        // Calculate metrics
        self.tool_sizes.clear();
        for loc in &findings.locations {
            *self
                .tool_sizes
                .entry(loc.tool.name().to_string())
                .or_insert(0) += loc.size_bytes;
        }

        self.total_files = findings.total_files;
        self.estimated_tokens = findings.total_size_bytes / 4;
        self.estimated_cost = (self.estimated_tokens as f64 / 1_000_000.0) * 12.0;

        // Load conversation stats
        let analyzer = ConversationAnalyzer::new(self.base_dir.clone());
        if let Ok(stats) = analyzer.analyze() {
            self.total_conversations = stats.total_conversations;
            self.total_messages = stats.total_messages;
            self.estimated_tokens = stats.total_tokens_estimate;
            self.estimated_cost = (self.estimated_tokens as f64 / 1_000_000.0) * 12.0;
        }

        // Detect active sessions (files modified in last 5 minutes)
        self.active_sessions = findings
            .locations
            .iter()
            .filter(|loc| {
                if let Some(newest) = loc.newest_entry {
                    let age = chrono::Utc::now().signed_duration_since(newest);
                    age.num_minutes() < 5
                } else {
                    false
                }
            })
            .count();

        // Track peaks
        self.peak_storage = self.peak_storage.max(findings.total_size_bytes);
        self.peak_tokens = self.peak_tokens.max(self.estimated_tokens);
        self.peak_active = self.peak_active.max(self.active_sessions);

        // Save snapshot to history
        let snapshot = MetricsSnapshot {
            timestamp: now,
            storage: findings.total_size_bytes,
            conversations: self.total_conversations,
            messages: self.total_messages,
            tokens: self.estimated_tokens,
            cost: self.estimated_cost,
            files: self.total_files,
            active_sessions: self.active_sessions,
        };
        self.history.push(snapshot);

        self.findings = Some(findings);
        self.update_count += 1;
        self.last_update = Instant::now();

        // Load insights on first update
        if self.update_count == 1 {
            self.load_insights();
        }

        self.status_message = if self.active_sessions > 0 {
            format!("ACTIVE: {} sessions | {}", self.active_sessions, format_uptime(self.uptime))
        } else {
            format!("IDLE | {}", format_uptime(self.uptime))
        };

        Ok(())
    }

    fn load_insights(&mut self) {
        // Load Claude Code stats
        let parser = ClaudeCodeParser::new(self.base_dir.clone());
        if let Ok(stats) = parser.parse() {
            self.total_conversations = stats.total_conversations;
            self.total_messages = stats.total_messages;
            self.estimated_tokens = stats.estimated_tokens.max(self.estimated_tokens);
        }

        // Load viral insights
        let viral = ViralAnalyzer::new(
            self.base_dir.clone(),
            self.estimated_tokens,
            self.estimated_cost,
        );
        if let Ok(insights) = viral.analyze() {
            // Copy hourly heatmap
            for (hour, count) in &insights.time_analytics.hourly_heatmap {
                if *hour < 24 {
                    self.hourly_activity[*hour] = *count as u64;
                }
            }

            self.achievements_unlocked = insights.achievements.iter().filter(|a| a.unlocked).count();
            self.insights = Some(insights);
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 3;
    }

    pub fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            2
        } else {
            self.selected_tab - 1
        };
    }
}

/// Run the real-time TUI with btop-style interface
pub fn run_tui(base_dir: PathBuf) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(base_dir);

    // Initial update
    app.update()?;

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        // Update every 500ms for smoother feel
        let timeout = UPDATE_INTERVAL_MS.saturating_sub(last_tick.elapsed().as_millis() as u64);

        if event::poll(Duration::from_millis(timeout))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('p') | KeyCode::Char(' ') => app.toggle_pause(),
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::BackTab => app.prev_tab(),
                        KeyCode::Char('1') => app.selected_tab = 0,
                        KeyCode::Char('2') => app.selected_tab = 1,
                        KeyCode::Char('3') => app.selected_tab = 2,
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= Duration::from_millis(UPDATE_INTERVAL_MS) && !app.paused {
            app.update()?;
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with tabs
            Constraint::Min(10),    // Main content
            Constraint::Length(2),  // Status bar
        ])
        .split(f.area());

    // Header with tabs
    render_header(f, app, chunks[0]);

    // Main content based on selected tab
    match app.selected_tab {
        0 => render_dashboard(f, app, chunks[1]),
        1 => render_metrics(f, app, chunks[1]),
        2 => render_tools(f, app, chunks[1]),
        _ => {}
    }

    // Status bar with rich info
    render_status_bar(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] Dashboard", "[2] Metrics", "[3] Tools"];

    let status_indicator = if app.paused {
        Span::styled(" ⏸ PAUSED ", Style::default().fg(Color::Black).bg(Color::Yellow).bold())
    } else if app.active_sessions > 0 {
        Span::styled(" ● ACTIVE ", Style::default().fg(Color::Black).bg(Color::Green).bold())
    } else {
        Span::styled(" ○ IDLE ", Style::default().fg(Color::Black).bg(Color::Blue).bold())
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(vec![
                    Span::styled("╔═ ", Style::default().fg(Color::Cyan).bold()),
                    Span::styled("vibedev", Style::default().fg(Color::Magenta).bold()),
                    Span::styled(" monitor ", Style::default().fg(Color::White)),
                    status_indicator,
                    Span::styled(" ═╗", Style::default().fg(Color::Cyan).bold()),
                ]),
        )
        .select(app.selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        .divider(symbols::line::VERTICAL);

    f.render_widget(tabs, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Left side - status message
    let status_text = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.status_message, Style::default().fg(if app.active_sessions > 0 { Color::Green } else { Color::Cyan })),
        ]),
    ])
    .style(Style::default());

    f.render_widget(status_text, status_chunks[0]);

    // Right side - controls
    let controls = Paragraph::new(Line::from(vec![
        Span::styled("TAB", Style::default().fg(Color::Yellow).bold()),
        Span::styled(":Switch ", Style::default().fg(Color::DarkGray)),
        Span::styled("SPACE", Style::default().fg(Color::Yellow).bold()),
        Span::styled(":Pause ", Style::default().fg(Color::DarkGray)),
        Span::styled("Q", Style::default().fg(Color::Red).bold()),
        Span::styled(":Quit │", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Right);

    f.render_widget(controls, status_chunks[1]);
}

fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),  // Overview cards
            Constraint::Min(10),    // Charts and activity
        ])
        .split(area);

    // Overview cards (4 key metrics)
    render_overview_cards(f, app, main_chunks[0]);

    // Bottom: Charts side by side
    let chart_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    render_primary_chart(f, app, chart_chunks[0]);
    render_activity_panel(f, app, chart_chunks[1]);
}

fn render_overview_cards(f: &mut Frame, app: &App, area: Rect) {
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Card 1: Storage
    let storage_val = app.history.storage.back().cloned().unwrap_or(0);
    let storage_pct = if app.peak_storage > 0 {
        ((storage_val as f64 / app.peak_storage as f64) * 100.0) as u16
    } else {
        0
    };
    let storage_color = metric_color(storage_pct);

    let storage_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(storage_color))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(storage_color)),
                    Span::styled("Storage", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(storage_color)),
                ]),
        )
        .gauge_style(Style::default().fg(storage_color).bg(Color::Black))
        .percent(storage_pct)
        .label(format!("{}", format_bytes(storage_val)));
    f.render_widget(storage_gauge, card_chunks[0]);

    // Card 2: Tokens
    let tokens_val = app.history.tokens.back().cloned().unwrap_or(0);
    let tokens_pct = if app.peak_tokens > 0 {
        ((tokens_val as f64 / app.peak_tokens as f64) * 100.0) as u16
    } else {
        0
    };
    let tokens_color = metric_color(tokens_pct);

    let tokens_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(tokens_color))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(tokens_color)),
                    Span::styled("Tokens", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(tokens_color)),
                ]),
        )
        .gauge_style(Style::default().fg(tokens_color).bg(Color::Black))
        .percent(tokens_pct)
        .label(format!("{}", format_tokens(tokens_val)));
    f.render_widget(tokens_gauge, card_chunks[1]);

    // Card 3: Cost
    let cost_val = app.history.cost.back().cloned().unwrap_or(0.0);
    let cost_pct = ((cost_val / 2000.0) * 100.0).min(100.0) as u16;
    let cost_color = metric_color(cost_pct);

    let cost_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(cost_color))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(cost_color)),
                    Span::styled("Cost", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(cost_color)),
                ]),
        )
        .gauge_style(Style::default().fg(cost_color).bg(Color::Black))
        .percent(cost_pct)
        .label(format!("${:.2}", cost_val));
    f.render_widget(cost_gauge, card_chunks[2]);

    // Card 4: Active Sessions
    let active_val = app.history.active_sessions.back().cloned().unwrap_or(0);
    let active_pct = if app.peak_active > 0 {
        ((active_val as f64 / app.peak_active as f64) * 100.0) as u16
    } else if active_val > 0 {
        100
    } else {
        0
    };
    let active_color = if active_val > 0 {
        Color::Green
    } else {
        Color::DarkGray
    };

    let active_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(active_color))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(active_color)),
                    Span::styled("Active", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(active_color)),
                ]),
        )
        .gauge_style(Style::default().fg(active_color).bg(Color::Black))
        .percent(active_pct)
        .label(format!("{} sessions", active_val));
    f.render_widget(active_gauge, card_chunks[3]);
}

fn render_primary_chart(f: &mut Frame, app: &App, area: Rect) {
    // Main chart showing storage/tokens/cost trends
    let chart_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Min(5),
        ])
        .split(area);

    // Storage sparkline with gradient effect
    let storage_data = app.history.get_sparkline_data(MetricType::Storage, 60);
    let storage_max = *storage_data.iter().max().unwrap_or(&1).max(&1);
    let storage_sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(vec![
                    Span::styled("▶ ", Style::default().fg(Color::Cyan)),
                    Span::styled("Storage Flow ", Style::default().fg(Color::White).bold()),
                    Span::styled(format!("[{}]", format_bytes(storage_data.last().cloned().unwrap_or(0))), Style::default().fg(Color::Cyan)),
                ]),
        )
        .data(&storage_data)
        .max(storage_max)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(storage_sparkline, chart_area[0]);

    // Tokens sparkline
    let tokens_data = app.history.get_sparkline_data(MetricType::Tokens, 60);
    let tokens_max = *tokens_data.iter().max().unwrap_or(&1).max(&1);
    let tokens_sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(vec![
                    Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                    Span::styled("Token Flow ", Style::default().fg(Color::White).bold()),
                    Span::styled(format!("[{}]", format_tokens(tokens_data.last().cloned().unwrap_or(0))), Style::default().fg(Color::Yellow)),
                ]),
        )
        .data(&tokens_data)
        .max(tokens_max)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(tokens_sparkline, chart_area[1]);

    // Stats summary
    let stats_lines = vec![
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Conversations: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.total_conversations), Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Messages:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(format_large(app.total_messages), Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Files:         ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.total_files), Style::default().fg(Color::Blue).bold()),
        ]),
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Tools:         ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.tool_sizes.len()), Style::default().fg(Color::Magenta).bold()),
        ]),
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled("Updates:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.update_count), Style::default().fg(Color::Yellow)),
        ]),
    ];

    let stats_para = Paragraph::new(stats_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(Color::White)),
                    Span::styled("Summary", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(Color::White)),
                ]),
        );
    f.render_widget(stats_para, chart_area[2]);
}

fn render_activity_panel(f: &mut Frame, app: &App, area: Rect) {
    let activity_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(10)])
        .split(area);

    // Active sessions sparkline
    let active_data = app.history.get_sparkline_data(MetricType::ActiveSessions, 60);
    let active_max = *active_data.iter().max().unwrap_or(&1).max(&1);
    let active_color = if active_data.last().cloned().unwrap_or(0) > 0 {
        Color::Green
    } else {
        Color::DarkGray
    };

    let active_sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(active_color))
                .title(vec![
                    Span::styled("▶ ", Style::default().fg(active_color)),
                    Span::styled("Active Sessions ", Style::default().fg(Color::White).bold()),
                    Span::styled(format!("[{}]", active_data.last().cloned().unwrap_or(0)), Style::default().fg(active_color)),
                ]),
        )
        .data(&active_data)
        .max(active_max)
        .style(Style::default().fg(active_color));
    f.render_widget(active_sparkline, activity_chunks[0]);

    // Hourly heatmap with better colors
    let mut heatmap_lines = vec![
        Line::from(Span::styled("  Hour of Day Activity", Style::default().fg(Color::White).bold())),
        Line::from(""),
    ];

    // Hour labels
    let hour_labels = Line::from(
        (0..24)
            .map(|h| {
                Span::styled(
                    format!("{:02} ", h),
                    Style::default().fg(Color::DarkGray),
                )
            })
            .collect::<Vec<_>>(),
    );
    heatmap_lines.push(hour_labels);

    // Activity bars
    let max_hourly = *app.hourly_activity.iter().max().unwrap_or(&1).max(&1);
    let bar_spans: Vec<Span> = app
        .hourly_activity
        .iter()
        .map(|&count| {
            let intensity = if max_hourly > 0 {
                (count as f64 / max_hourly as f64 * 8.0) as usize
            } else {
                0
            };

            let (block, color) = match intensity {
                0 => ("░░ ", Color::DarkGray),
                1 => ("▁▁ ", Color::Blue),
                2 => ("▂▂ ", Color::Blue),
                3 => ("▃▃ ", Color::Cyan),
                4 => ("▄▄ ", Color::Cyan),
                5 => ("▅▅ ", Color::Green),
                6 => ("▆▆ ", Color::Green),
                7 => ("▇▇ ", Color::Yellow),
                _ => ("██ ", Color::Red),
            };

            Span::styled(block, Style::default().fg(color).bold())
        })
        .collect();

    heatmap_lines.push(Line::from(bar_spans));

    // Peak info
    let peak_hour = app
        .hourly_activity
        .iter()
        .enumerate()
        .max_by_key(|(_, &v)| v)
        .map(|(h, _)| h)
        .unwrap_or(0);

    heatmap_lines.push(Line::from(""));
    heatmap_lines.push(Line::from(vec![
        Span::styled("  Peak Activity: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:02}:00-{:02}:00", peak_hour, (peak_hour + 1) % 24),
            Style::default().fg(Color::Green).bold(),
        ),
    ]));

    let heatmap = Paragraph::new(heatmap_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(Color::Magenta)),
                    Span::styled("Activity Heatmap", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(Color::Magenta)),
                ]),
        );
    f.render_widget(heatmap, activity_chunks[1]);
}

fn render_metrics(f: &mut Frame, app: &App, area: Rect) {
    let metrics_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: Detailed metrics
    render_detailed_metrics(f, app, metrics_chunks[0]);

    // Right: Trends and rates
    render_trends(f, app, metrics_chunks[1]);
}

fn render_detailed_metrics(f: &mut Frame, app: &App, area: Rect) {
    let metric_lines = vec![
        Line::from(vec![
            Span::styled("╔═══ ", Style::default().fg(Color::Cyan)),
            Span::styled("Current Values", Style::default().fg(Color::White).bold()),
            Span::styled(" ═══╗", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Storage:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_bytes(app.history.storage.back().cloned().unwrap_or(0)),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Tokens:        ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_tokens(app.history.tokens.back().cloned().unwrap_or(0)),
                Style::default().fg(Color::Yellow).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Cost:          ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.2}", app.history.cost.back().cloned().unwrap_or(0.0)),
                Style::default().fg(Color::Magenta).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Conversations: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", app.history.conversations.back().cloned().unwrap_or(0)),
                Style::default().fg(Color::Green).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Messages:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_large(app.history.messages.back().cloned().unwrap_or(0)),
                Style::default().fg(Color::Blue).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Files:         ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", app.history.files.back().cloned().unwrap_or(0)),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Active:        ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", app.history.active_sessions.back().cloned().unwrap_or(0)),
                Style::default().fg(Color::Green).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("╔═══ ", Style::default().fg(Color::Yellow)),
            Span::styled("Peak Values", Style::default().fg(Color::White).bold()),
            Span::styled(" ═══╗", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Storage:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_bytes(app.peak_storage),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Tokens:        ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_tokens(app.peak_tokens),
                Style::default().fg(Color::Yellow).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Active:        ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", app.peak_active),
                Style::default().fg(Color::Green).bold(),
            ),
        ]),
    ];

    let metrics_para = Paragraph::new(metric_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(" Metrics "),
        );
    f.render_widget(metrics_para, area);
}

fn render_trends(f: &mut Frame, app: &App, area: Rect) {
    let trends_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Calculate rates
    let (storage_rate, conv_rate, msg_rate) = if app.history.storage.len() >= 2 {
        let time_diff = app.history.timestamps.len() as f64;

        let storage_first = app.history.storage.front().cloned().unwrap_or(0);
        let storage_last = app.history.storage.back().cloned().unwrap_or(0);
        let storage_rate = ((storage_last as f64 - storage_first as f64) / time_diff) as i64;

        let conv_first = app.history.conversations.front().cloned().unwrap_or(0);
        let conv_last = app.history.conversations.back().cloned().unwrap_or(0);
        let conv_rate = ((conv_last as f64 - conv_first as f64) / time_diff * 60.0) as i64;

        let msg_first = app.history.messages.front().cloned().unwrap_or(0);
        let msg_last = app.history.messages.back().cloned().unwrap_or(0);
        let msg_rate = ((msg_last as f64 - msg_first as f64) / time_diff * 60.0) as i64;

        (storage_rate, conv_rate, msg_rate)
    } else {
        (0, 0, 0)
    };

    let rate_lines = vec![
        Line::from(vec![
            Span::styled("╔═══ ", Style::default().fg(Color::Green)),
            Span::styled("Rates", Style::default().fg(Color::White).bold()),
            Span::styled(" ═══╗", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Storage:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/s", format_bytes_signed(storage_rate)),
                Style::default().fg(if storage_rate > 0 { Color::Green } else if storage_rate < 0 { Color::Red } else { Color::DarkGray }).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Convos:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:+}/min", conv_rate),
                Style::default().fg(if conv_rate > 0 { Color::Green } else if conv_rate < 0 { Color::Red } else { Color::DarkGray }).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Messages:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:+}/min", msg_rate),
                Style::default().fg(if msg_rate > 0 { Color::Green } else if msg_rate < 0 { Color::Red } else { Color::DarkGray }).bold(),
            ),
        ]),
    ];

    let rates_para = Paragraph::new(rate_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(" Live Rates "),
        );
    f.render_widget(rates_para, trends_chunks[0]);

    // Deltas over history window
    let deltas = if app.history.storage.len() >= 2 {
        let storage_delta = app.history.storage.back().cloned().unwrap_or(0) as i64
            - app.history.storage.front().cloned().unwrap_or(0) as i64;
        let conv_delta = app.history.conversations.back().cloned().unwrap_or(0) as i64
            - app.history.conversations.front().cloned().unwrap_or(0) as i64;
        let msg_delta = app.history.messages.back().cloned().unwrap_or(0) as i64
            - app.history.messages.front().cloned().unwrap_or(0) as i64;
        let cost_delta = app.history.cost.back().cloned().unwrap_or(0.0)
            - app.history.cost.front().cloned().unwrap_or(0.0);

        vec![
            Line::from(vec![
                Span::styled("╔═══ ", Style::default().fg(Color::Magenta)),
                Span::styled(format!("Changes ({}s)", app.history.timestamps.len()), Style::default().fg(Color::White).bold()),
                Span::styled(" ═══╗", Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Storage:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format_bytes_signed(storage_delta),
                    Style::default().fg(if storage_delta > 0 { Color::Green } else if storage_delta < 0 { Color::Red } else { Color::DarkGray }).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Convos:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:+}", conv_delta),
                    Style::default().fg(if conv_delta > 0 { Color::Green } else if conv_delta < 0 { Color::Red } else { Color::DarkGray }).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Messages:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:+}", msg_delta),
                    Style::default().fg(if msg_delta > 0 { Color::Green } else if msg_delta < 0 { Color::Red } else { Color::DarkGray }).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled("  Cost:       ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("${:+.2}", cost_delta),
                    Style::default().fg(if cost_delta > 0.01 { Color::Red } else if cost_delta < -0.01 { Color::Green } else { Color::DarkGray }).bold(),
                ),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled("  Collecting data...", Style::default().fg(Color::DarkGray)))]
    };

    let deltas_para = Paragraph::new(deltas)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Delta Tracker "),
        );
    f.render_widget(deltas_para, trends_chunks[1]);
}

fn render_tools(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref findings) = app.findings else {
        let placeholder = Paragraph::new("  Scanning for AI tools...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Tools Discovery "),
            )
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, area);
        return;
    };

    let mut tool_data: Vec<_> = app.tool_sizes.iter().collect();
    tool_data.sort_by(|a, b| b.1.cmp(a.1));

    let rows: Vec<Row> = tool_data
        .iter()
        .enumerate()
        .map(|(idx, (name, size))| {
            let pct = (**size as f64 / findings.total_size_bytes as f64) * 100.0;
            let bar_width = ((pct / 100.0) * 40.0) as usize;
            let bar = "█".repeat(bar_width);
            let empty = "░".repeat(40 - bar_width);

            let row_color = match idx {
                0 => Color::Cyan,
                1 => Color::Green,
                2 => Color::Yellow,
                _ => Color::White,
            };

            Row::new(vec![
                Span::styled(format!("{:2}", idx + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(name.to_string(), Style::default().fg(row_color).bold()),
                Span::styled(format_bytes(**size), Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:>5.1}%", pct)),
                Span::styled(bar, Style::default().fg(Color::Green).bold()),
                Span::styled(empty, Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let header = Row::new(vec!["#", "Tool", "Size", "%", "Distribution", ""])
        .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(18),
            Constraint::Length(12),
            Constraint::Length(7),
            Constraint::Min(40),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!("╣ Tools Discovery ({}) ╠", tool_data.len()))
            .title_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(table, area);
}

// Helper functions for color-coded metrics
fn metric_color(percent: u16) -> Color {
    match percent {
        0..=30 => Color::Green,
        31..=60 => Color::Yellow,
        61..=80 => Color::LightRed,
        _ => Color::Red,
    }
}

fn format_uptime(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

fn format_bytes_signed(bytes: i64) -> String {
    let abs_bytes = bytes.abs() as u64;
    let formatted = format_bytes(abs_bytes);
    if bytes >= 0 {
        format!("+{}", formatted)
    } else {
        format!("-{}", formatted)
    }
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000_000 {
        format!("{:.1}B", tokens as f64 / 1_000_000_000.0)
    } else if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        format!("{}", tokens)
    }
}

fn format_large(num: usize) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        format!("{}", num)
    }
}

/// Print CLI output (non-interactive mode)
pub fn print_cli_output(base_dir: PathBuf) -> Result<()> {
    use colored::Colorize as ColoredColorize;
    use std::time::Instant;

    println!(
        "{}",
        ColoredColorize::bold(ColoredColorize::cyan("vibedev - Scanning..."))
    );
    println!();

    let start = Instant::now();
    let discovery = crate::discovery::LogDiscovery::new(base_dir.clone(), true);
    let findings = discovery.scan()?;
    let elapsed = start.elapsed();

    // Calculate tool sizes
    let mut tool_sizes: HashMap<String, u64> = HashMap::new();
    for loc in &findings.locations {
        *tool_sizes.entry(loc.tool.name().to_string()).or_insert(0) += loc.size_bytes;
    }

    // Sort by size
    let mut tool_items: Vec<_> = tool_sizes.iter().collect();
    tool_items.sort_by(|a, b| b.1.cmp(a.1));

    // Estimate tokens and cost
    let estimated_tokens = findings.total_size_bytes / 4;
    let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 12.0;

    // Header
    println!(
        "{}",
        ColoredColorize::bright_black(
            "═══════════════════════════════════════════════════════════════════════════════"
        )
    );
    println!(
        "  {}  {:>12}  {:>7}  {}",
        ColoredColorize::bold("Tool"),
        ColoredColorize::bold("Size"),
        ColoredColorize::bold("%"),
        ColoredColorize::bold("Distribution")
    );
    println!(
        "{}",
        ColoredColorize::bright_black(
            "───────────────────────────────────────────────────────────────────────────────"
        )
    );

    // Tool rows with visual bars
    for (name, size) in &tool_items {
        let pct = (**size as f64 / findings.total_size_bytes as f64) * 100.0;
        let bar_width = ((pct / 100.0) * 40.0) as usize;
        let bar = "█".repeat(bar_width);
        let empty = "░".repeat(40 - bar_width);

        println!(
            "  {:<18} {:>10}  {:>5.1}%  {}{}",
            ColoredColorize::cyan(name.as_str()),
            ColoredColorize::yellow(format_bytes(**size).as_str()),
            pct,
            ColoredColorize::green(bar.as_str()),
            ColoredColorize::bright_black(empty.as_str())
        );
    }

    println!(
        "{}",
        ColoredColorize::bright_black(
            "═══════════════════════════════════════════════════════════════════════════════"
        )
    );

    // Summary
    println!();
    println!(
        "{}",
        ColoredColorize::underline(ColoredColorize::bold("Summary"))
    );
    println!(
        "  Total Storage:  {}",
        ColoredColorize::bold(ColoredColorize::yellow(
            format_bytes(findings.total_size_bytes).as_str()
        ))
    );
    println!(
        "  Total Files:    {}",
        ColoredColorize::cyan(findings.total_files.to_string().as_str())
    );
    println!(
        "  Tools Found:    {}",
        ColoredColorize::cyan(findings.tools_found.len().to_string().as_str())
    );
    println!(
        "  Est. Tokens:    {}",
        ColoredColorize::yellow(format_tokens(estimated_tokens).as_str())
    );
    println!(
        "  Est. Cost:      {}",
        ColoredColorize::bold(ColoredColorize::magenta(
            format!("${:.2}", estimated_cost).as_str()
        ))
    );
    println!("  Scan Time:      {:.2}s", elapsed.as_secs_f64());

    // Load more stats
    let parser = ClaudeCodeParser::new(base_dir.clone());
    if let Ok(stats) = parser.parse() {
        println!();
        println!(
            "{}",
            ColoredColorize::underline(ColoredColorize::bold("Conversations"))
        );
        println!(
            "  Total:          {}",
            ColoredColorize::cyan(stats.total_conversations.to_string().as_str())
        );
        println!(
            "  Messages:       {}",
            ColoredColorize::cyan(stats.total_messages.to_string().as_str())
        );
        println!(
            "  User:           {}",
            ColoredColorize::green(stats.user_messages.to_string().as_str())
        );
        println!(
            "  Assistant:      {}",
            ColoredColorize::blue(stats.assistant_messages.to_string().as_str())
        );
    }

    // Top locations
    println!();
    println!(
        "{}",
        ColoredColorize::underline(ColoredColorize::bold("Top Locations by Size"))
    );

    let mut locations: Vec<_> = findings.locations.iter().collect();
    locations.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    for loc in locations.iter().take(10) {
        let pct = (loc.size_bytes as f64 / findings.total_size_bytes as f64) * 100.0;
        let path_str = loc.path.to_string_lossy();
        let truncated = if path_str.len() > 50 {
            format!("...{}", &path_str[path_str.len() - 47..])
        } else {
            path_str.to_string()
        };

        println!(
            "  {:>10} {:>5.1}%  {} {} {}",
            ColoredColorize::yellow(format_bytes(loc.size_bytes).as_str()),
            pct,
            ColoredColorize::cyan(loc.tool.name()),
            ColoredColorize::bright_black(format!("{:?}", loc.log_type).as_str()),
            ColoredColorize::bright_black(truncated.as_str())
        );
    }

    println!();
    println!(
        "{}",
        ColoredColorize::bright_black("Run 'vibedev tui' for real-time btop-style monitoring")
    );

    Ok(())
}
