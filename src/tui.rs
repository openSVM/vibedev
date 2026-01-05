// TUI module - AI Coding Intelligence Dashboard
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

const UPDATE_INTERVAL_MS: u64 = 500;
const HISTORY_SIZE: usize = 120; // 60 seconds at 500ms

/// Smart insights and recommendations
#[derive(Debug, Clone)]
pub struct SmartInsight {
    pub category: InsightCategory,
    pub message: String,
    pub severity: InsightSeverity,
}

#[derive(Debug, Clone)]
pub enum InsightCategory {
    Productivity,
    Cost,
    Efficiency,
    Health,
    Recommendation,
}

#[derive(Debug, Clone, Copy)]
pub enum InsightSeverity {
    Info,
    Warning,
    Critical,
    Positive,
}

/// Productivity and efficiency metrics
#[derive(Debug, Clone)]
pub struct ProductivityMetrics {
    pub score: f64,              // 0-100
    pub efficiency_rating: f64,  // 0-100
    pub cost_health: f64,        // 0-100
    pub session_quality: f64,    // 0-100
    pub daily_burn_rate: f64,
    pub weekly_burn_rate: f64,
    pub monthly_projection: f64,
    pub tokens_per_conversation: f64,
    pub cost_per_conversation: f64,
    pub avg_session_length: Duration,
    pub peak_hour: usize,
    pub current_streak_days: usize,
}

impl ProductivityMetrics {
    fn calculate(app: &App) -> Self {
        let conversations = app.total_conversations.max(1);
        let tokens_per_conv = app.estimated_tokens as f64 / conversations as f64;
        let cost_per_conv = app.estimated_cost / conversations as f64;

        // Productivity score based on activity and efficiency
        let activity_score = (app.active_sessions as f64 / 5.0 * 40.0).min(40.0);
        let efficiency_score = if tokens_per_conv < 50000.0 { 30.0 } else if tokens_per_conv < 100000.0 { 20.0 } else { 10.0 };
        let volume_score = (conversations as f64 / 100.0 * 30.0).min(30.0);
        let score = activity_score + efficiency_score + volume_score;

        // Efficiency rating based on tokens per conversation
        let efficiency_rating = if tokens_per_conv < 30000.0 { 90.0 }
            else if tokens_per_conv < 50000.0 { 75.0 }
            else if tokens_per_conv < 100000.0 { 60.0 }
            else { 40.0 };

        // Cost health - lower is better
        let daily_burn = app.estimated_cost / app.uptime.as_secs_f64().max(1.0) * 86400.0;
        let cost_health = if daily_burn < 5.0 { 90.0 }
            else if daily_burn < 10.0 { 70.0 }
            else if daily_burn < 20.0 { 50.0 }
            else { 30.0 };

        // Session quality based on active vs total
        let session_quality = if app.total_files > 0 {
            (app.active_sessions as f64 / app.total_files.min(10) as f64 * 100.0).min(100.0)
        } else { 50.0 };

        let weekly_burn = daily_burn * 7.0;
        let monthly_projection = daily_burn * 30.0;

        let peak_hour = app.hourly_activity
            .iter()
            .enumerate()
            .max_by_key(|(_, &v)| v)
            .map(|(h, _)| h)
            .unwrap_or(0);

        Self {
            score,
            efficiency_rating,
            cost_health,
            session_quality,
            daily_burn_rate: daily_burn,
            weekly_burn_rate: weekly_burn,
            monthly_projection,
            tokens_per_conversation: tokens_per_conv,
            cost_per_conversation: cost_per_conv,
            avg_session_length: Duration::from_secs(app.uptime.as_secs() / conversations.max(1) as u64),
            peak_hour,
            current_streak_days: 0, // TODO: Calculate from history
        }
    }

    fn generate_insights(&self, app: &App) -> Vec<SmartInsight> {
        let mut insights = Vec::new();

        // Cost insights
        if self.daily_burn_rate > 20.0 {
            insights.push(SmartInsight {
                category: InsightCategory::Cost,
                message: format!("⚠ High burn rate: ${:.2}/day - consider using cheaper models", self.daily_burn_rate),
                severity: InsightSeverity::Warning,
            });
        } else if self.daily_burn_rate < 5.0 {
            insights.push(SmartInsight {
                category: InsightCategory::Cost,
                message: format!("✓ Efficient spending: ${:.2}/day", self.daily_burn_rate),
                severity: InsightSeverity::Positive,
            });
        }

        // Efficiency insights
        if self.tokens_per_conversation > 100000.0 {
            insights.push(SmartInsight {
                category: InsightCategory::Efficiency,
                message: "💡 High token usage - try breaking down complex tasks".to_string(),
                severity: InsightSeverity::Info,
            });
        } else if self.tokens_per_conversation < 30000.0 {
            insights.push(SmartInsight {
                category: InsightCategory::Efficiency,
                message: "✓ Excellent token efficiency!".to_string(),
                severity: InsightSeverity::Positive,
            });
        }

        // Productivity insights
        if self.score > 80.0 {
            insights.push(SmartInsight {
                category: InsightCategory::Productivity,
                message: "🚀 Outstanding productivity today!".to_string(),
                severity: InsightSeverity::Positive,
            });
        } else if self.score < 40.0 {
            insights.push(SmartInsight {
                category: InsightCategory::Productivity,
                message: "💤 Low activity detected - time for a break?".to_string(),
                severity: InsightSeverity::Info,
            });
        }

        // Peak hour recommendation
        insights.push(SmartInsight {
            category: InsightCategory::Recommendation,
            message: format!("📊 Your peak hour: {:02}:00 - schedule deep work then", self.peak_hour),
            severity: InsightSeverity::Info,
        });

        // Active session health
        if app.active_sessions > 5 {
            insights.push(SmartInsight {
                category: InsightCategory::Health,
                message: format!("⚠ {} active sessions - high context switching", app.active_sessions),
                severity: InsightSeverity::Warning,
            });
        } else if app.active_sessions > 0 {
            insights.push(SmartInsight {
                category: InsightCategory::Health,
                message: format!("✓ {} active session(s) - good focus", app.active_sessions),
                severity: InsightSeverity::Positive,
            });
        }

        // Cost projection warning
        if self.monthly_projection > 200.0 {
            insights.push(SmartInsight {
                category: InsightCategory::Cost,
                message: format!("⚠ Monthly projection: ${:.2} - monitor usage", self.monthly_projection),
                severity: InsightSeverity::Critical,
            });
        }

        insights
    }
}

/// Real-time metrics tracker
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
    pub productivity_score: VecDeque<f64>,
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
            productivity_score: VecDeque::with_capacity(HISTORY_SIZE),
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
            self.productivity_score.pop_front();
        }

        self.timestamps.push_back(snapshot.timestamp);
        self.storage.push_back(snapshot.storage);
        self.conversations.push_back(snapshot.conversations);
        self.messages.push_back(snapshot.messages);
        self.tokens.push_back(snapshot.tokens);
        self.cost.push_back(snapshot.cost);
        self.files.push_back(snapshot.files);
        self.active_sessions.push_back(snapshot.active_sessions);
        self.productivity_score.push_back(snapshot.productivity_score);
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
            MetricType::ProductivityScore => {
                return self.productivity_score.iter().map(|&x| x as u64).collect()
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
    ProductivityScore,
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
    pub productivity_score: f64,
}

/// Enhanced monitoring app with intelligence
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
    pub productivity: ProductivityMetrics,
    pub smart_insights: Vec<SmartInsight>,
    pub current_branch: String,
    pub recent_files: Vec<String>,
}

impl App {
    pub fn new(base_dir: PathBuf) -> Self {
        let dummy_metrics = ProductivityMetrics {
            score: 0.0,
            efficiency_rating: 0.0,
            cost_health: 100.0,
            session_quality: 0.0,
            daily_burn_rate: 0.0,
            weekly_burn_rate: 0.0,
            monthly_projection: 0.0,
            tokens_per_conversation: 0.0,
            cost_per_conversation: 0.0,
            avg_session_length: Duration::ZERO,
            peak_hour: 0,
            current_streak_days: 0,
        };

        Self {
            findings: None,
            insights: None,
            base_dir,
            status_message: "Initializing AI Coding Intelligence...".to_string(),
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
            productivity: dummy_metrics,
            smart_insights: Vec::new(),
            current_branch: String::new(),
            recent_files: Vec::new(),
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

        // Detect active sessions
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

        // Get git context
        self.update_git_context();

        // Calculate productivity metrics
        self.productivity = ProductivityMetrics::calculate(self);

        // Generate smart insights
        self.smart_insights = self.productivity.generate_insights(self);

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
            productivity_score: self.productivity.score,
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
            format!("● ACTIVE: {} sessions | Score: {:.0} | {}",
                self.active_sessions, self.productivity.score, format_uptime(self.uptime))
        } else {
            format!("○ IDLE | Score: {:.0} | {}",
                self.productivity.score, format_uptime(self.uptime))
        };

        Ok(())
    }

    fn update_git_context(&mut self) {
        // Try to get current git branch
        if let Ok(output) = std::process::Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(&self.base_dir)
            .output()
        {
            if output.status.success() {
                self.current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }

        // Get recently modified files
        if let Ok(output) = std::process::Command::new("git")
            .args(&["diff", "--name-only", "HEAD"])
            .current_dir(&self.base_dir)
            .output()
        {
            if output.status.success() {
                self.recent_files = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .take(5)
                    .map(String::from)
                    .collect();
            }
        }
    }

    fn load_insights(&mut self) {
        let parser = ClaudeCodeParser::new(self.base_dir.clone());
        if let Ok(stats) = parser.parse() {
            self.total_conversations = stats.total_conversations;
            self.total_messages = stats.total_messages;
            self.estimated_tokens = stats.estimated_tokens.max(self.estimated_tokens);
        }

        let viral = ViralAnalyzer::new(
            self.base_dir.clone(),
            self.estimated_tokens,
            self.estimated_cost,
        );
        if let Ok(insights) = viral.analyze() {
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

pub fn run_tui(base_dir: PathBuf) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(base_dir);
    app.update()?;

    let result = run_app(&mut terminal, &mut app);

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
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);

    match app.selected_tab {
        0 => render_intelligence_dashboard(f, app, chunks[1]),
        1 => render_deep_insights(f, app, chunks[1]),
        2 => render_tools(f, app, chunks[1]),
        _ => {}
    }

    render_status_bar(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] Intelligence", "[2] Deep Insights", "[3] Tools"];

    let status_indicator = if app.paused {
        Span::styled(" ⏸ PAUSED ", Style::default().fg(Color::Black).bg(Color::Yellow).bold())
    } else if app.active_sessions > 0 {
        Span::styled(" ● ACTIVE ", Style::default().fg(Color::Black).bg(Color::Green).bold())
    } else {
        Span::styled(" ○ IDLE ", Style::default().fg(Color::Black).bg(Color::Blue).bold())
    };

    let score_badge = Span::styled(
        format!(" Score: {:.0} ", app.productivity.score),
        Style::default().fg(Color::Black).bg(score_color(app.productivity.score as u16)).bold()
    );

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(vec![
                    Span::styled("╔═ ", Style::default().fg(Color::Cyan).bold()),
                    Span::styled("vibedev", Style::default().fg(Color::Magenta).bold()),
                    Span::styled(" AI Intelligence ", Style::default().fg(Color::White)),
                    status_indicator,
                    Span::raw(" "),
                    score_badge,
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

    let status_text = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.status_message, Style::default().fg(if app.active_sessions > 0 { Color::Green } else { Color::Cyan })),
        ]),
    ]);

    f.render_widget(status_text, status_chunks[0]);

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

fn render_intelligence_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),  // Score cards
            Constraint::Min(10),    // Productivity & Insights
        ])
        .split(area);

    render_score_cards(f, app, main_chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    render_productivity_panel(f, app, bottom_chunks[0]);
    render_smart_insights(f, app, bottom_chunks[1]);
}

fn render_score_cards(f: &mut Frame, app: &App, area: Rect) {
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Productivity Score
    let prod_pct = app.productivity.score as u16;
    let prod_color = score_color(prod_pct);
    let prod_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(prod_color))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(prod_color)),
                    Span::styled("Productivity", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(prod_color)),
                ]),
        )
        .gauge_style(Style::default().fg(prod_color).bg(Color::Black))
        .percent(prod_pct.min(100))
        .label(format!("{:.0}/100", app.productivity.score));
    f.render_widget(prod_gauge, card_chunks[0]);

    // Efficiency Rating
    let eff_pct = app.productivity.efficiency_rating as u16;
    let eff_color = score_color(eff_pct);
    let eff_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(eff_color))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(eff_color)),
                    Span::styled("Efficiency", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(eff_color)),
                ]),
        )
        .gauge_style(Style::default().fg(eff_color).bg(Color::Black))
        .percent(eff_pct.min(100))
        .label(format!("{:.0}/100", app.productivity.efficiency_rating));
    f.render_widget(eff_gauge, card_chunks[1]);

    // Cost Health
    let cost_pct = app.productivity.cost_health as u16;
    let cost_color = score_color(cost_pct);
    let cost_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(cost_color))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(cost_color)),
                    Span::styled("Cost Health", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(cost_color)),
                ]),
        )
        .gauge_style(Style::default().fg(cost_color).bg(Color::Black))
        .percent(cost_pct.min(100))
        .label(format!("${:.2}/day", app.productivity.daily_burn_rate));
    f.render_widget(cost_gauge, card_chunks[2]);

    // Session Quality
    let qual_pct = app.productivity.session_quality as u16;
    let qual_color = score_color(qual_pct);
    let qual_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(qual_color))
                .title(vec![
                    Span::styled("╣ ", Style::default().fg(qual_color)),
                    Span::styled("Session Quality", Style::default().fg(Color::White).bold()),
                    Span::styled(" ╠", Style::default().fg(qual_color)),
                ]),
        )
        .gauge_style(Style::default().fg(qual_color).bg(Color::Black))
        .percent(qual_pct.min(100))
        .label(format!("{} active", app.active_sessions));
    f.render_widget(qual_gauge, card_chunks[3]);
}

fn render_productivity_panel(f: &mut Frame, app: &App, area: Rect) {
    let prod_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(5)])
        .split(area);

    // Productivity sparkline
    let prod_data = app.history.get_sparkline_data(MetricType::ProductivityScore, 60);
    let prod_max = *prod_data.iter().max().unwrap_or(&100).max(&100);
    let prod_sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(vec![
                    Span::styled("▶ ", Style::default().fg(Color::Magenta)),
                    Span::styled("Productivity Flow ", Style::default().fg(Color::White).bold()),
                    Span::styled(format!("[{:.0}]", prod_data.last().cloned().unwrap_or(0)), Style::default().fg(Color::Magenta)),
                ]),
        )
        .data(&prod_data)
        .max(prod_max)
        .style(Style::default().fg(Color::Magenta));
    f.render_widget(prod_sparkline, prod_chunks[0]);

    // Session context
    let context_lines = vec![
        Line::from(vec![
            Span::styled("╔═ ", Style::default().fg(Color::Cyan)),
            Span::styled("Current Session", Style::default().fg(Color::White).bold()),
            Span::styled(" ═╗", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Branch:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if app.current_branch.is_empty() { "N/A" } else { &app.current_branch },
                Style::default().fg(Color::Green).bold()
            ),
        ]),
        Line::from(vec![
            Span::styled("  Files:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} tracked", app.total_files), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Active:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} sessions", app.active_sessions), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Convos:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.total_conversations), Style::default().fg(Color::Magenta)),
        ]),
    ];

    let context_para = Paragraph::new(context_lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::White)));
    f.render_widget(context_para, prod_chunks[1]);
}

fn render_smart_insights(f: &mut Frame, app: &App, area: Rect) {
    let insights_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(10)])
        .split(area);

    // Smart insights panel
    let mut insight_lines = vec![
        Line::from(vec![
            Span::styled("╔═ ", Style::default().fg(Color::Yellow)),
            Span::styled("Smart Insights", Style::default().fg(Color::White).bold()),
            Span::styled(" ═╗", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
    ];

    for insight in app.smart_insights.iter().take(6) {
        let color = match insight.severity {
            InsightSeverity::Positive => Color::Green,
            InsightSeverity::Info => Color::Cyan,
            InsightSeverity::Warning => Color::Yellow,
            InsightSeverity::Critical => Color::Red,
        };

        insight_lines.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(color)),
            Span::styled(&insight.message, Style::default().fg(color)),
        ]));
    }

    if app.smart_insights.is_empty() {
        insight_lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("Analyzing your patterns...", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let insights_para = Paragraph::new(insight_lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));
    f.render_widget(insights_para, insights_chunks[0]);

    // Cost projection panel
    let projection_lines = vec![
        Line::from(vec![
            Span::styled("╔═ ", Style::default().fg(Color::Magenta)),
            Span::styled("Cost Projections", Style::default().fg(Color::White).bold()),
            Span::styled(" ═╗", Style::default().fg(Color::Magenta)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Daily:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("${:.2}", app.productivity.daily_burn_rate), Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Weekly:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("${:.2}", app.productivity.weekly_burn_rate), Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Monthly:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.2}", app.productivity.monthly_projection),
                Style::default().fg(if app.productivity.monthly_projection > 200.0 { Color::Red } else { Color::Green }).bold()
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Per Conv: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("${:.3}", app.productivity.cost_per_conversation), Style::default().fg(Color::Cyan)),
        ]),
    ];

    let projection_para = Paragraph::new(projection_lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)));
    f.render_widget(projection_para, insights_chunks[1]);
}

fn render_deep_insights(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_efficiency_panel(f, app, main_chunks[0]);
    render_activity_intelligence(f, app, main_chunks[1]);
}

fn render_efficiency_panel(f: &mut Frame, app: &App, area: Rect) {
    let eff_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),
            Constraint::Length(9),
            Constraint::Min(5),
        ])
        .split(area);

    // Efficiency metrics
    let eff_lines = vec![
        Line::from(vec![
            Span::styled("╔═══ ", Style::default().fg(Color::Cyan)),
            Span::styled("Efficiency Metrics", Style::default().fg(Color::White).bold()),
            Span::styled(" ═══╗", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tokens/Conv:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format_tokens(app.productivity.tokens_per_conversation as u64), Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Cost/Conv:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("${:.3}", app.productivity.cost_per_conversation), Style::default().fg(Color::Magenta).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Avg Session:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format_uptime(app.productivity.avg_session_length), Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Peak Hour:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:02}:00", app.productivity.peak_hour), Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Total Convos:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.total_conversations), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Total Messages: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format_large(app.total_messages), Style::default().fg(Color::Green)),
        ]),
    ];

    let eff_para = Paragraph::new(eff_lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::White)));
    f.render_widget(eff_para, eff_chunks[0]);

    // Token flow
    let token_data = app.history.get_sparkline_data(MetricType::Tokens, 60);
    let token_max = *token_data.iter().max().unwrap_or(&1).max(&1);
    let token_sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(vec![
                    Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                    Span::styled("Token Flow ", Style::default().fg(Color::White).bold()),
                    Span::styled(format!("[{}]", format_tokens(token_data.last().cloned().unwrap_or(0))), Style::default().fg(Color::Yellow)),
                ]),
        )
        .data(&token_data)
        .max(token_max)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(token_sparkline, eff_chunks[1]);

    // Storage info
    let storage_lines = vec![
        Line::from(vec![
            Span::styled("╔═ ", Style::default().fg(Color::Cyan)),
            Span::styled("Storage", Style::default().fg(Color::White).bold()),
            Span::styled(" ═╗", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Current: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format_bytes(app.history.storage.back().cloned().unwrap_or(0)), Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Peak:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format_bytes(app.peak_storage), Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Files:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.total_files), Style::default().fg(Color::Green)),
        ]),
    ];

    let storage_para = Paragraph::new(storage_lines)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(storage_para, eff_chunks[2]);
}

fn render_activity_intelligence(f: &mut Frame, app: &App, area: Rect) {
    let act_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(10)])
        .split(area);

    // Active sessions
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
    f.render_widget(active_sparkline, act_chunks[0]);

    // Activity heatmap
    let mut heatmap_lines = vec![
        Line::from(Span::styled("  Hour of Day Activity", Style::default().fg(Color::White).bold())),
        Line::from(""),
    ];

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

    heatmap_lines.push(Line::from(""));
    heatmap_lines.push(Line::from(vec![
        Span::styled("  Peak: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:02}:00-{:02}:00", app.productivity.peak_hour, (app.productivity.peak_hour + 1) % 24),
            Style::default().fg(Color::Green).bold(),
        ),
        Span::styled(" | Most productive hour", Style::default().fg(Color::DarkGray)),
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
    f.render_widget(heatmap, act_chunks[1]);
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

fn score_color(score: u16) -> Color {
    match score {
        80..=100 => Color::Green,
        60..=79 => Color::Yellow,
        40..=59 => Color::LightRed,
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

pub fn print_cli_output(base_dir: PathBuf) -> Result<()> {
    use colored::Colorize as ColoredColorize;
    use std::time::Instant;

    println!(
        "{}",
        ColoredColorize::bold(ColoredColorize::cyan("vibedev - AI Coding Intelligence"))
    );
    println!();

    let start = Instant::now();
    let discovery = crate::discovery::LogDiscovery::new(base_dir.clone(), true);
    let findings = discovery.scan()?;
    let elapsed = start.elapsed();

    let mut tool_sizes: HashMap<String, u64> = HashMap::new();
    for loc in &findings.locations {
        *tool_sizes.entry(loc.tool.name().to_string()).or_insert(0) += loc.size_bytes;
    }

    let mut tool_items: Vec<_> = tool_sizes.iter().collect();
    tool_items.sort_by(|a, b| b.1.cmp(a.1));

    let estimated_tokens = findings.total_size_bytes / 4;
    let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 12.0;

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

    println!();
    println!(
        "{}",
        ColoredColorize::bright_black("Run 'vibedev tui' for AI Coding Intelligence Dashboard")
    );

    Ok(())
}
