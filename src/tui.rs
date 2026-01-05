// TUI module - Clean, actionable AI coding insights
use crate::analyzer::ConversationAnalyzer;
use crate::claude_code_parser::ClaudeCodeParser;
use crate::discovery::LogDiscovery;
use crate::models::DiscoveryFindings;
use crate::timeline::{Timeline, TimelineAnalyzer};
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
        Block, Borders, Paragraph, Row, Sparkline, Table, Tabs,
    },
    Frame, Terminal,
};
use std::collections::{HashMap, VecDeque};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const UPDATE_INTERVAL_MS: u64 = 1000;
const HISTORY_SIZE: usize = 60;

#[derive(Debug, Clone)]
pub struct MetricsHistory {
    pub timestamps: VecDeque<u64>,
    pub cost: VecDeque<f64>,
    pub conversations: VecDeque<usize>,
    pub tokens: VecDeque<u64>,
}

impl MetricsHistory {
    fn new() -> Self {
        Self {
            timestamps: VecDeque::with_capacity(HISTORY_SIZE),
            cost: VecDeque::with_capacity(HISTORY_SIZE),
            conversations: VecDeque::with_capacity(HISTORY_SIZE),
            tokens: VecDeque::with_capacity(HISTORY_SIZE),
        }
    }

    fn push(&mut self, timestamp: u64, cost: f64, conversations: usize, tokens: u64) {
        if self.timestamps.len() >= HISTORY_SIZE {
            self.timestamps.pop_front();
            self.cost.pop_front();
            self.conversations.pop_front();
            self.tokens.pop_front();
        }
        self.timestamps.push_back(timestamp);
        self.cost.push_back(cost);
        self.conversations.push_back(conversations);
        self.tokens.push_back(tokens);
    }
}

pub struct App {
    pub findings: Option<DiscoveryFindings>,
    pub insights: Option<ViralInsights>,
    pub timeline: Option<Timeline>,
    pub base_dir: PathBuf,
    pub tool_sizes: HashMap<String, u64>,
    pub estimated_tokens: u64,
    pub estimated_cost: f64,
    pub total_conversations: usize,
    pub total_messages: usize,
    pub history: MetricsHistory,
    pub update_count: u64,
    pub paused: bool,
    pub selected_tab: usize,
    pub hourly_activity: [u64; 24],
    pub start_time: Instant,
    pub current_branch: String,
    pub scroll_offset: usize,
}

impl App {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            findings: None,
            insights: None,
            timeline: None,
            base_dir,
            tool_sizes: HashMap::new(),
            estimated_tokens: 0,
            estimated_cost: 0.0,
            total_conversations: 0,
            total_messages: 0,
            history: MetricsHistory::new(),
            update_count: 0,
            paused: false,
            selected_tab: 0,
            hourly_activity: [0; 24],
            start_time: Instant::now(),
            current_branch: String::new(),
            scroll_offset: 0,
        }
    }

    pub fn update(&mut self) -> Result<()> {
        if self.paused {
            return Ok(());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let discovery = LogDiscovery::new(self.base_dir.clone(), true);
        let findings = discovery.scan()?;

        self.tool_sizes.clear();
        for loc in &findings.locations {
            *self
                .tool_sizes
                .entry(loc.tool.name().to_string())
                .or_insert(0) += loc.size_bytes;
        }

        self.estimated_tokens = findings.total_size_bytes / 4;
        self.estimated_cost = (self.estimated_tokens as f64 / 1_000_000.0) * 12.0;

        let analyzer = ConversationAnalyzer::new(self.base_dir.clone());
        if let Ok(stats) = analyzer.analyze() {
            self.total_conversations = stats.total_conversations;
            self.total_messages = stats.total_messages;
            self.estimated_tokens = stats.total_tokens_estimate;
            self.estimated_cost = (self.estimated_tokens as f64 / 1_000_000.0) * 12.0;
        }

        self.update_git_context();

        self.history.push(
            now,
            self.estimated_cost,
            self.total_conversations,
            self.estimated_tokens,
        );

        self.findings = Some(findings);
        self.update_count += 1;

        if self.update_count == 1 {
            self.load_insights();
        }

        Ok(())
    }

    fn update_git_context(&mut self) {
        if let Ok(output) = std::process::Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(&self.base_dir)
            .output()
        {
            if output.status.success() {
                self.current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
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
            self.insights = Some(insights);
        }

        // Load timeline
        let timeline_analyzer = TimelineAnalyzer::new(self.base_dir.clone());
        if let Ok(timeline) = timeline_analyzer.analyze() {
            self.timeline = Some(timeline);
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 5;
    }

    pub fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            4
        } else {
            self.selected_tab - 1
        };
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    fn get_cost_trend(&self) -> (f64, &'static str) {
        if self.history.cost.len() < 2 {
            return (0.0, "→");
        }
        let recent = self.history.cost.back().cloned().unwrap_or(0.0);
        let previous = self.history.cost.front().cloned().unwrap_or(0.0);
        let change = recent - previous;
        let trend = if change > 0.5 { "↑" } else if change < -0.5 { "↓" } else { "→" };
        (change, trend)
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
                        KeyCode::Char('4') => app.selected_tab = 3,
                        KeyCode::Char('5') => app.selected_tab = 4,
                        KeyCode::Up => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
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
            Constraint::Length(1),
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);

    match app.selected_tab {
        0 => render_overview(f, app, chunks[1]),
        1 => render_analysis(f, app, chunks[1]),
        2 => render_tools(f, app, chunks[1]),
        3 => render_timeline(f, app, chunks[1]),
        4 => render_infographics(f, app, chunks[1]),
        _ => {}
    }

    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["[1] Overview", "[2] Analysis", "[3] Tools", "[4] Timeline", "[5] Git Infographics"];

    let title = if app.current_branch.is_empty() {
        " vibecheck ".to_string()
    } else {
        format!(" vibecheck ({}) ", app.current_branch)
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Style::default().fg(Color::Cyan).bold()),
        )
        .select(app.selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).bold())
        .divider(" | ");

    f.render_widget(tabs, area);
}

fn render_footer(f: &mut Frame, _app: &App, area: Rect) {
    let footer = Paragraph::new(" TAB: Switch  SPACE: Pause  Q: Quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, area);
}

fn render_overview(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),   // Summary
            Constraint::Length(9),   // Cost trend
            Constraint::Min(10),     // Tool breakdown
        ])
        .split(area);

    // Summary box
    let conv_per_msg = if app.total_conversations > 0 {
        app.total_messages as f64 / app.total_conversations as f64
    } else {
        0.0
    };

    let cost_per_conv = if app.total_conversations > 0 {
        app.estimated_cost / app.total_conversations as f64
    } else {
        0.0
    };

    let (cost_change, trend) = app.get_cost_trend();
    let trend_color = if cost_change > 0.5 { Color::Red } else if cost_change < -0.5 { Color::Green } else { Color::Yellow };

    let summary_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Total Cost:      "),
            Span::styled(format!("${:.2}", app.estimated_cost), Style::default().fg(Color::Yellow).bold()),
            Span::raw("  "),
            Span::styled(trend, Style::default().fg(trend_color)),
        ]),
        Line::from(vec![
            Span::raw("  Conversations:   "),
            Span::styled(format!("{}", app.total_conversations), Style::default().fg(Color::Cyan)),
            Span::raw(format!("  (${:.3}/conv)", cost_per_conv)),
        ]),
        Line::from(vec![
            Span::raw("  Messages:        "),
            Span::styled(format!("{}", app.total_messages), Style::default().fg(Color::Green)),
            Span::raw(format!("  ({:.1} msg/conv)", conv_per_msg)),
        ]),
        Line::from(vec![
            Span::raw("  Tokens:          "),
            Span::styled(format_tokens(app.estimated_tokens), Style::default().fg(Color::Magenta)),
        ]),
    ];

    let summary = Paragraph::new(summary_lines)
        .block(Block::default().borders(Borders::ALL).title(" Summary "));
    f.render_widget(summary, chunks[0]);

    // Cost over time
    let cost_data: Vec<u64> = app.history.cost.iter().map(|&x| (x * 100.0) as u64).collect();
    if !cost_data.is_empty() {
        let max_cost = *cost_data.iter().max().unwrap_or(&1).max(&1);
        let cost_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Cost Trend (last {}s) ", cost_data.len())),
            )
            .data(&cost_data)
            .max(max_cost)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(cost_sparkline, chunks[1]);
    }

    // Tool breakdown
    render_tool_breakdown(f, app, chunks[2]);
}

fn render_analysis(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: Efficiency metrics
    render_efficiency(f, app, chunks[0]);

    // Right: Activity heatmap
    render_activity(f, app, chunks[1]);
}

fn render_efficiency(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(5),
        ])
        .split(area);

    let tokens_per_conv = if app.total_conversations > 0 {
        app.estimated_tokens as f64 / app.total_conversations as f64
    } else {
        0.0
    };

    let cost_per_1k_tokens = if app.estimated_tokens > 0 {
        app.estimated_cost / (app.estimated_tokens as f64 / 1000.0)
    } else {
        0.0
    };

    let efficiency_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Tokens/Conv:     "),
            Span::styled(format!("{:.0}", tokens_per_conv), Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(vec![
            Span::raw("  Cost/1K tokens:  "),
            Span::styled(format!("${:.4}", cost_per_1k_tokens), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Insight: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if tokens_per_conv < 30000.0 {
                    "Efficient conversations"
                } else if tokens_per_conv < 100000.0 {
                    "Moderate token usage"
                } else {
                    "High token usage"
                },
                Style::default().fg(
                    if tokens_per_conv < 30000.0 { Color::Green }
                    else if tokens_per_conv < 100000.0 { Color::Yellow }
                    else { Color::Red }
                )
            ),
        ]),
    ];

    let efficiency = Paragraph::new(efficiency_lines)
        .block(Block::default().borders(Borders::ALL).title(" Efficiency "));
    f.render_widget(efficiency, chunks[0]);

    // Token trend
    let token_data: Vec<u64> = app.history.tokens.iter().cloned().collect();
    if !token_data.is_empty() {
        let max_tokens = *token_data.iter().max().unwrap_or(&1).max(&1);
        let token_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Token Flow ({}s) ", token_data.len())),
            )
            .data(&token_data)
            .max(max_tokens)
            .style(Style::default().fg(Color::Magenta));
        f.render_widget(token_sparkline, chunks[1]);
    }
}

fn render_activity(f: &mut Frame, app: &App, area: Rect) {
    let mut heatmap_lines = vec![
        Line::from(""),
        Line::from(Span::styled("  24-Hour Activity", Style::default().fg(Color::White).bold())),
        Line::from(""),
    ];

    let hour_labels = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            (0..24).map(|h| format!("{:02} ", h)).collect::<String>(),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    heatmap_lines.push(hour_labels);

    let max_hourly = *app.hourly_activity.iter().max().unwrap_or(&1).max(&1);
    let mut bar_spans = vec![Span::raw("  ")];

    for &count in &app.hourly_activity {
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

        bar_spans.push(Span::styled(block, Style::default().fg(color)));
    }

    heatmap_lines.push(Line::from(bar_spans));

    let peak_hour = app
        .hourly_activity
        .iter()
        .enumerate()
        .max_by_key(|(_, &v)| v)
        .map(|(h, _)| h)
        .unwrap_or(0);

    heatmap_lines.push(Line::from(""));
    heatmap_lines.push(Line::from(vec![
        Span::raw("  Most active: "),
        Span::styled(
            format!("{:02}:00", peak_hour),
            Style::default().fg(Color::Green).bold(),
        ),
    ]));

    let heatmap = Paragraph::new(heatmap_lines)
        .block(Block::default().borders(Borders::ALL).title(" Activity Pattern "));
    f.render_widget(heatmap, area);
}

fn render_tool_breakdown(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref findings) = app.findings else {
        let placeholder = Paragraph::new("  Scanning...")
            .block(Block::default().borders(Borders::ALL).title(" Tools "));
        f.render_widget(placeholder, area);
        return;
    };

    let mut tool_data: Vec<_> = app.tool_sizes.iter().collect();
    tool_data.sort_by(|a, b| b.1.cmp(a.1));

    let rows: Vec<Row> = tool_data
        .iter()
        .map(|(name, size)| {
            let pct = (**size as f64 / findings.total_size_bytes.max(1) as f64) * 100.0;
            let bar_width = ((pct / 100.0) * 30.0) as usize;
            let bar = "█".repeat(bar_width);

            Row::new(vec![
                Span::styled(name.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled(format_bytes(**size), Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:>5.1}%", pct)),
                Span::styled(bar, Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let header = Row::new(vec!["Tool", "Size", "%", ""])
        .style(Style::default().fg(Color::White).bold())
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Min(30),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" By Tool "));

    f.render_widget(table, area);
}

fn render_tools(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref findings) = app.findings else {
        let placeholder = Paragraph::new("  Scanning...")
            .block(Block::default().borders(Borders::ALL).title(" Tools "));
        f.render_widget(placeholder, area);
        return;
    };

    let mut tool_data: Vec<_> = app.tool_sizes.iter().collect();
    tool_data.sort_by(|a, b| b.1.cmp(a.1));

    let rows: Vec<Row> = tool_data
        .iter()
        .enumerate()
        .map(|(idx, (name, size))| {
            let pct = (**size as f64 / findings.total_size_bytes.max(1) as f64) * 100.0;
            let bar_width = ((pct / 100.0) * 50.0) as usize;
            let bar = "█".repeat(bar_width);
            let empty = "░".repeat(50 - bar_width);

            let name_color = match idx {
                0 => Color::Cyan,
                1 => Color::Green,
                2 => Color::Yellow,
                _ => Color::White,
            };

            Row::new(vec![
                Span::styled(format!("{}", idx + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(name.to_string(), Style::default().fg(name_color).bold()),
                Span::styled(format_bytes(**size), Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:>5.1}%", pct)),
                Span::styled(bar, Style::default().fg(Color::Green)),
                Span::styled(empty, Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let header = Row::new(vec!["#", "Tool", "Size", "%", "Distribution", ""])
        .style(Style::default().fg(Color::White).bold())
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(18),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Min(50),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Tools ({}) ", tool_data.len())),
    );

    f.render_widget(table, area);
}

fn render_timeline(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref timeline) = app.timeline else {
        let placeholder = Paragraph::new("  Analyzing your coding journey...")
            .block(Block::default().borders(Borders::ALL).title(" Timeline "));
        f.render_widget(placeholder, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(10)])
        .split(area);

    // Stats box
    let stats_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Total Sessions:   "),
            Span::styled(format!("{}", timeline.stats.total_sessions), Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::raw("  Completed:        "),
            Span::styled(format!("{}", timeline.stats.completed), Style::default().fg(Color::Green).bold()),
            Span::raw(format!(" ({:.0}%)", timeline.stats.completion_rate)),
        ]),
        Line::from(vec![
            Span::raw("  Abandoned:        "),
            Span::styled(format!("{}", timeline.stats.abandoned), Style::default().fg(Color::Red).bold()),
        ]),
        Line::from(vec![
            Span::raw("  Ongoing:          "),
            Span::styled(format!("{}", timeline.stats.ongoing), Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Avg Session:      "),
            Span::styled(format!("{:.1}h", timeline.stats.avg_session_hours), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::raw("  Context Switches: "),
            Span::styled(format!("{}", timeline.stats.context_switches), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  Most Worked:      "),
            Span::styled(&timeline.stats.most_worked_project, Style::default().fg(Color::Cyan)),
        ]),
    ];

    let stats_box = Paragraph::new(stats_lines)
        .block(Block::default().borders(Borders::ALL).title(" Your Coding Journey Stats "));
    f.render_widget(stats_box, chunks[0]);

    // Timeline visualization
    let mut timeline_lines = vec![
        Line::from(""),
    ];

    let visible_sessions: Vec<_> = timeline.sessions
        .iter()
        .skip(app.scroll_offset)
        .take(20)
        .collect();

    if visible_sessions.is_empty() {
        timeline_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("No sessions found", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for session in &visible_sessions {
            let outcome_color = match session.outcome {
                crate::timeline::SessionOutcome::Completed => Color::Green,
                crate::timeline::SessionOutcome::Abandoned => Color::Red,
                crate::timeline::SessionOutcome::Resumed(_) => Color::Yellow,
                crate::timeline::SessionOutcome::Ongoing => Color::Cyan,
            };

            // Timeline bar length based on hours
            let bar_len = (session.hours * 2.0).min(40.0) as usize;
            let bar = "━".repeat(bar_len.max(1));

            timeline_lines.push(Line::from(vec![
                Span::styled(session.start.format("%Y-%m-%d").to_string(), Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled("●", Style::default().fg(outcome_color)),
                Span::styled(bar, Style::default().fg(outcome_color)),
                Span::styled(session.outcome.symbol().to_string(), Style::default().fg(outcome_color).bold()),
                Span::raw("  "),
                Span::styled(session.description.clone(), Style::default().fg(Color::White)),
            ]));

            timeline_lines.push(Line::from(vec![
                Span::raw("            "),
                Span::styled(
                    format!("{:.1}h | {} convos | {}",
                        session.hours,
                        session.conversations,
                        session.outcome.description()
                    ),
                    Style::default().fg(Color::DarkGray)
                ),
            ]));
            timeline_lines.push(Line::from(""));
        }
    }

    if app.scroll_offset > 0 {
        timeline_lines.insert(1, Line::from(vec![
            Span::raw("  "),
            Span::styled("↑ Scroll up for more ↑", Style::default().fg(Color::Yellow)),
        ]));
    }

    if app.scroll_offset + 20 < timeline.sessions.len() {
        timeline_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("↓ Scroll down for more ↓", Style::default().fg(Color::Yellow)),
        ]));
    }

    let timeline_para = Paragraph::new(timeline_lines)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Timeline ({}/{}) ",
            app.scroll_offset + visible_sessions.len().min(20),
            timeline.sessions.len()
        )));
    f.render_widget(timeline_para, chunks[1]);
}

fn render_infographics(f: &mut Frame, _app: &App, area: Rect) {
    use std::fs;
    use std::path::Path;

    let infographics_dir = Path::new("/tmp/git-infographics");

    let mut info_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Git Infographics Generator  ", Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(""),
    ];

    if infographics_dir.exists() {
        info_lines.push(Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::White)),
            Span::styled("Generated", Style::default().fg(Color::Green).bold()),
        ]));
        info_lines.push(Line::from(""));

        // List generated files
        info_lines.push(Line::from(vec![
            Span::styled("  Generated Infographics:", Style::default().fg(Color::Yellow)),
        ]));

        let infographic_files = vec![
            ("commit_heatmap.png", "Calendar heatmap of daily commits"),
            ("top_contributors.png", "Top 15 contributors by commit count"),
            ("activity_timeline.png", "Commit activity over time (monthly)"),
            ("hourly_activity.png", "Commits by hour of day"),
            ("weekday_distribution.png", "Commits by day of week"),
            ("message_quality.png", "Commit message length distribution"),
            ("code_contribution.png", "Lines added/deleted by top contributors"),
        ];

        for (filename, description) in infographic_files {
            let path = infographics_dir.join(filename);
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path) {
                    let size_kb = metadata.len() / 1024;
                    info_lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled("✓ ", Style::default().fg(Color::Green).bold()),
                        Span::styled(filename, Style::default().fg(Color::Cyan)),
                        Span::raw(format!(" ({} KB)", size_kb)),
                    ]));
                    info_lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(description, Style::default().fg(Color::DarkGray)),
                    ]));
                }
            } else {
                info_lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled("✗ ", Style::default().fg(Color::Red)),
                    Span::styled(filename, Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![
            Span::styled("  Output Directory: ", Style::default().fg(Color::White)),
            Span::styled(infographics_dir.display().to_string(), Style::default().fg(Color::Magenta)),
        ]));
    } else {
        info_lines.push(Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::White)),
            Span::styled("Not Generated", Style::default().fg(Color::Yellow)),
        ]));
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("No infographics found at ", Style::default().fg(Color::DarkGray)),
            Span::styled(infographics_dir.display().to_string(), Style::default().fg(Color::Magenta)),
        ]));
    }

    info_lines.push(Line::from(""));
    info_lines.push(Line::from(""));
    info_lines.push(Line::from(vec![
        Span::styled("  Commands:", Style::default().fg(Color::Yellow)),
    ]));
    info_lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled("vibedev git-infographics", Style::default().fg(Color::Cyan).bold()),
        Span::raw("  - Analyze current directory"),
    ]));
    info_lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled("vibedev git-infographics --scan-all", Style::default().fg(Color::Cyan).bold()),
        Span::raw("  - Scan all repos in $HOME"),
    ]));
    info_lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled("vibedev git-infographics -r /path/to/repo", Style::default().fg(Color::Cyan).bold()),
        Span::raw("  - Analyze specific repo"),
    ]));
    info_lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled("vibedev git-infographics --open", Style::default().fg(Color::Cyan).bold()),
        Span::raw("  - Open in browser"),
    ]));

    let para = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Git Infographics "));

    f.render_widget(para, area);
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

pub fn print_cli_output(base_dir: PathBuf) -> Result<()> {
    use colored::Colorize as ColoredColorize;

    println!("{}", ColoredColorize::bold(ColoredColorize::cyan("vibedev")));
    println!();

    let discovery = crate::discovery::LogDiscovery::new(base_dir.clone(), true);
    let findings = discovery.scan()?;

    let mut tool_sizes: HashMap<String, u64> = HashMap::new();
    for loc in &findings.locations {
        *tool_sizes.entry(loc.tool.name().to_string()).or_insert(0) += loc.size_bytes;
    }

    let mut tool_items: Vec<_> = tool_sizes.iter().collect();
    tool_items.sort_by(|a, b| b.1.cmp(a.1));

    let estimated_tokens = findings.total_size_bytes / 4;
    let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 12.0;

    for (name, size) in &tool_items {
        let pct = (**size as f64 / findings.total_size_bytes as f64) * 100.0;
        let bar_width = ((pct / 100.0) * 30.0) as usize;
        let bar = "█".repeat(bar_width);

        println!(
            "  {:<15} {:>10}  {:>5.1}%  {}",
            ColoredColorize::cyan(name.as_str()),
            ColoredColorize::yellow(format_bytes(**size).as_str()),
            pct,
            ColoredColorize::green(bar.as_str())
        );
    }

    println!();
    println!("  Total: {}", ColoredColorize::yellow(format_bytes(findings.total_size_bytes).as_str()));
    println!("  Tokens: {}", ColoredColorize::magenta(format_tokens(estimated_tokens).as_str()));
    println!("  Cost: {}", ColoredColorize::bold(format!("${:.2}", estimated_cost).as_str()));
    println!();
    println!("{}", ColoredColorize::bright_black("Run 'vibedev tui' for live monitoring"));

    Ok(())
}
