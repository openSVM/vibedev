// TUI module - Interactive terminal UI like dust
use crate::discovery::LogDiscovery;
use crate::models::DiscoveryFindings;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// App state for the TUI
pub struct App {
    pub findings: Option<DiscoveryFindings>,
    pub scan_progress: f64,
    pub scanning: bool,
    pub selected_tab: usize,
    pub selected_row: usize,
    pub base_dir: PathBuf,
    pub status_message: String,
    pub start_time: Instant,
    pub tool_sizes: HashMap<String, u64>,
}

impl App {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            findings: None,
            scan_progress: 0.0,
            scanning: false,
            selected_tab: 0,
            selected_row: 0,
            base_dir,
            status_message: "Press 's' to start scanning, 'q' to quit".to_string(),
            start_time: Instant::now(),
            tool_sizes: HashMap::new(),
        }
    }

    pub fn start_scan(&mut self) {
        self.scanning = true;
        self.scan_progress = 0.0;
        self.status_message = "Scanning...".to_string();
        self.start_time = Instant::now();
    }

    pub fn finish_scan(&mut self, findings: DiscoveryFindings) {
        self.scanning = false;
        self.scan_progress = 100.0;

        // Calculate per-tool sizes
        self.tool_sizes.clear();
        for loc in &findings.locations {
            *self
                .tool_sizes
                .entry(loc.tool.name().to_string())
                .or_insert(0) += loc.size_bytes;
        }

        let elapsed = self.start_time.elapsed();
        self.status_message = format!(
            "Scan complete: {} files, {} in {:.1}s",
            findings.total_files,
            format_bytes(findings.total_size_bytes),
            elapsed.as_secs_f64()
        );
        self.findings = Some(findings);
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 4;
        self.selected_row = 0;
    }

    pub fn prev_tab(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            3
        } else {
            self.selected_tab - 1
        };
        self.selected_row = 0;
    }

    pub fn next_row(&mut self) {
        if let Some(ref findings) = self.findings {
            let max_rows = match self.selected_tab {
                0 => findings.locations.len(),
                1 => findings.tools_found.len(),
                _ => 10,
            };
            if max_rows > 0 {
                self.selected_row = (self.selected_row + 1) % max_rows;
            }
        }
    }

    pub fn prev_row(&mut self) {
        if let Some(ref findings) = self.findings {
            let max_rows = match self.selected_tab {
                0 => findings.locations.len(),
                1 => findings.tools_found.len(),
                _ => 10,
            };
            if max_rows > 0 {
                self.selected_row = if self.selected_row == 0 {
                    max_rows - 1
                } else {
                    self.selected_row - 1
                };
            }
        }
    }
}

/// Run the TUI application
pub fn run_tui(base_dir: PathBuf) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(base_dir.clone());

    // Auto-start scan
    app.start_scan();
    let discovery = LogDiscovery::new(base_dir, true);

    // Run scan in background
    let findings = discovery.scan()?;
    app.finish_scan(findings);

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
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::BackTab => app.prev_tab(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_row(),
                        KeyCode::Up | KeyCode::Char('k') => app.prev_row(),
                        KeyCode::Char('s') if !app.scanning => {
                            app.start_scan();
                            let discovery = LogDiscovery::new(app.base_dir.clone(), true);
                            if let Ok(findings) = discovery.scan() {
                                app.finish_scan(findings);
                            }
                        }
                        KeyCode::Char('1') => app.selected_tab = 0,
                        KeyCode::Char('2') => app.selected_tab = 1,
                        KeyCode::Char('3') => app.selected_tab = 2,
                        KeyCode::Char('4') => app.selected_tab = 3,
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title + tabs
            Constraint::Length(3), // Progress/status
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header with tabs
    let titles = vec!["[1] Locations", "[2] Tools", "[3] Charts", "[4] Details"];
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" vibedev - AI Log Analyzer "),
        )
        .select(app.selected_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // Progress/status bar
    if app.scanning {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Scanning "))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(app.scan_progress as u16)
            .label(format!("{:.0}%", app.scan_progress));
        f.render_widget(gauge, chunks[1]);
    } else {
        let status = Paragraph::new(app.status_message.clone())
            .block(Block::default().borders(Borders::ALL).title(" Status "));
        f.render_widget(status, chunks[1]);
    }

    // Main content area
    match app.selected_tab {
        0 => render_locations(f, app, chunks[2]),
        1 => render_tools(f, app, chunks[2]),
        2 => render_charts(f, app, chunks[2]),
        3 => render_details(f, app, chunks[2]),
        _ => {}
    }

    // Footer
    let footer_text = " q:Quit | Tab:Switch | j/k:Navigate | s:Rescan | 1-4:Jump to tab ";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[3]);
}

fn render_locations(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref findings) = app.findings else {
        let placeholder = Paragraph::new("No data. Press 's' to scan.")
            .block(Block::default().borders(Borders::ALL).title(" Locations "));
        f.render_widget(placeholder, area);
        return;
    };

    // Sort locations by size (largest first)
    let mut locations: Vec<_> = findings.locations.iter().collect();
    locations.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    let rows: Vec<Row> = locations
        .iter()
        .enumerate()
        .map(|(i, loc)| {
            let style = if i == app.selected_row {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Calculate percentage of total
            let pct = (loc.size_bytes as f64 / findings.total_size_bytes as f64) * 100.0;

            // Create bar visualization
            let bar_width = ((pct / 100.0) * 20.0) as usize;
            let bar = "█".repeat(bar_width) + &"░".repeat(20 - bar_width);

            Row::new(vec![
                loc.tool.name().to_string(),
                format!("{:?}", loc.log_type),
                format_bytes(loc.size_bytes),
                format!("{:>5.1}%", pct),
                bar,
                loc.file_count.to_string(),
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec!["Tool", "Type", "Size", "%", "Usage", "Files"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(22),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Locations ({}) ", findings.locations.len())),
    );

    f.render_widget(table, area);
}

fn render_tools(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref findings) = app.findings else {
        let placeholder = Paragraph::new("No data. Press 's' to scan.")
            .block(Block::default().borders(Borders::ALL).title(" Tools "));
        f.render_widget(placeholder, area);
        return;
    };

    let mut tool_items: Vec<(String, u64)> = app
        .tool_sizes
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    tool_items.sort_by(|a, b| b.1.cmp(&a.1));

    let rows: Vec<Row> = tool_items
        .iter()
        .enumerate()
        .map(|(i, (name, size))| {
            let style = if i == app.selected_row {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let pct = (*size as f64 / findings.total_size_bytes as f64) * 100.0;
            let bar_width = ((pct / 100.0) * 30.0) as usize;
            let bar = "█".repeat(bar_width) + &"░".repeat(30 - bar_width);

            Row::new(vec![
                name.clone(),
                format_bytes(*size),
                format!("{:>5.1}%", pct),
                bar,
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec!["Tool", "Size", "%", "Distribution"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Min(32),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Tools ({}) ", findings.tools_found.len())),
    );

    f.render_widget(table, area);
}

fn render_charts(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref findings) = app.findings else {
        let placeholder = Paragraph::new("No data. Press 's' to scan.")
            .block(Block::default().borders(Borders::ALL).title(" Charts "));
        f.render_widget(placeholder, area);
        return;
    };

    // Split area into two charts
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: Tool size bar chart
    let mut tool_data: Vec<_> = app.tool_sizes.iter().collect();
    tool_data.sort_by(|a, b| b.1.cmp(a.1));

    let max_size = tool_data.first().map(|(_, s)| **s).unwrap_or(1);

    let bars: Vec<Bar> = tool_data
        .iter()
        .take(8)
        .map(|(name, size)| {
            let height = ((**size as f64 / max_size as f64) * 100.0) as u64;
            Bar::default()
                .value(height)
                .label(Line::from(truncate(name, 8)))
                .text_value(format_bytes(**size))
                .style(Style::default().fg(Color::Cyan))
        })
        .collect();

    let bar_chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Storage by Tool "),
        )
        .data(BarGroup::default().bars(&bars))
        .bar_width(8)
        .bar_gap(1)
        .max(100);

    f.render_widget(bar_chart, chunks[0]);

    // Right: Summary stats
    let summary_text = format!(
        "Total Storage: {}\n\
         Total Files: {}\n\
         Tools Found: {}\n\
         Locations: {}\n\n\
         Largest Tool: {}\n\
         Smallest Tool: {}",
        format_bytes(findings.total_size_bytes),
        findings.total_files,
        findings.tools_found.len(),
        findings.locations.len(),
        tool_data
            .first()
            .map(|(n, s)| format!("{} ({})", n, format_bytes(**s)))
            .unwrap_or_default(),
        tool_data
            .last()
            .map(|(n, s)| format!("{} ({})", n, format_bytes(**s)))
            .unwrap_or_default(),
    );

    let summary = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title(" Summary "));
    f.render_widget(summary, chunks[1]);
}

fn render_details(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref findings) = app.findings else {
        let placeholder = Paragraph::new("No data. Press 's' to scan.")
            .block(Block::default().borders(Borders::ALL).title(" Details "));
        f.render_widget(placeholder, area);
        return;
    };

    // Show detailed path info for selected location
    let mut locations: Vec<_> = findings.locations.iter().collect();
    locations.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    let selected = locations.get(app.selected_row);

    let content = if let Some(loc) = selected {
        format!(
            "Tool: {}\n\
             Type: {:?}\n\
             Path: {}\n\
             Size: {}\n\
             Files: {}\n\
             Oldest: {:?}\n\
             Newest: {:?}",
            loc.tool.name(),
            loc.log_type,
            loc.path.display(),
            format_bytes(loc.size_bytes),
            loc.file_count,
            loc.oldest_entry
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
            loc.newest_entry
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
        )
    } else {
        "No location selected".to_string()
    };

    let details = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Location Details "),
    );

    f.render_widget(details, area);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

/// Print CLI output (non-interactive mode)
pub fn print_cli_output(base_dir: PathBuf) -> Result<()> {
    use colored::Colorize;
    use std::time::Instant;

    println!("{}", "vibedev - Scanning...".cyan().bold());
    println!();

    let start = Instant::now();
    let discovery = crate::discovery::LogDiscovery::new(base_dir, true);
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

    // Header
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════════════════════"
            .bright_black()
    );
    println!(
        "  {}  {:>12}  {:>7}  {}",
        "Tool".bold(),
        "Size".bold(),
        "%".bold(),
        "Distribution".bold()
    );
    println!(
        "{}",
        "───────────────────────────────────────────────────────────────────────────────"
            .bright_black()
    );

    // Tool rows with visual bars
    for (name, size) in &tool_items {
        let pct = (**size as f64 / findings.total_size_bytes as f64) * 100.0;
        let bar_width = ((pct / 100.0) * 40.0) as usize;
        let bar = "█".repeat(bar_width);
        let empty = "░".repeat(40 - bar_width);

        println!(
            "  {:<18} {:>10}  {:>5.1}%  {}{}",
            name.cyan(),
            format_bytes(**size).yellow(),
            pct,
            bar.green(),
            empty.bright_black()
        );
    }

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════════════════════"
            .bright_black()
    );

    // Summary
    println!();
    println!("{}", "Summary".bold().underline());
    println!(
        "  Total Storage:  {}",
        format_bytes(findings.total_size_bytes).yellow().bold()
    );
    println!(
        "  Total Files:    {}",
        findings.total_files.to_string().cyan()
    );
    println!(
        "  Tools Found:    {}",
        findings.tools_found.len().to_string().cyan()
    );
    println!(
        "  Locations:      {}",
        findings.locations.len().to_string().cyan()
    );
    println!("  Scan Time:      {:.2}s", elapsed.as_secs_f64());

    // Top locations
    println!();
    println!("{}", "Top Locations by Size".bold().underline());

    let mut locations: Vec<_> = findings.locations.iter().collect();
    locations.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    for loc in locations.iter().take(10) {
        let pct = (loc.size_bytes as f64 / findings.total_size_bytes as f64) * 100.0;
        println!(
            "  {:>10} {:>5.1}%  {} {} {}",
            format_bytes(loc.size_bytes).yellow(),
            pct,
            loc.tool.name().cyan(),
            format!("{:?}", loc.log_type).bright_black(),
            truncate(&loc.path.to_string_lossy(), 40).bright_black()
        );
    }

    println!();

    Ok(())
}
