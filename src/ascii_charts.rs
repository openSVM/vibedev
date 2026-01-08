//! ASCII Chart rendering for beautiful terminal visualizations
//! Inspired by Claude Code's /model output
//!
//! This module provides 10+ beautiful ASCII chart types:
//! - LineChart: Time-series with multi-series support
//! - BarChart: Horizontal bars with percentages
//! - ActivityHeatmap: GitHub-style contribution graph
//! - StatsCard: Key metrics in card format
//! - FunFact: Whimsical token comparisons
//! - StreakCounter: Streak with flames
//! - Histogram: Distribution visualization
//! - ProgressBar: Goal tracking
//! - Leaderboard: Ranked list with medals
//! - CalendarView: Monthly calendar with activity
//! - TimeDistribution: Hour-of-day breakdown
//! - ComparisonChart: Side-by-side comparison

#![allow(dead_code)]

use chrono::{DateTime, Datelike, Utc};
use colored::Colorize;
use std::collections::HashMap;

/// Characters for line chart drawing
const CHART_CHARS: [char; 9] = ['┼', '│', '─', '╭', '╮', '╰', '╯', '┤', '┬'];

/// Braille-style sparkline characters (8 levels)
const SPARK_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Box drawing characters
const BOX_H: char = '─';
const BOX_V: char = '│';
const BOX_TL: char = '╭';
const BOX_TR: char = '╮';
const BOX_BL: char = '╰';
const BOX_BR: char = '╯';

/// Color palette for multi-series charts
const COLORS: [&str; 6] = ["cyan", "magenta", "yellow", "green", "blue", "red"];

/// A data point with timestamp and value
#[derive(Clone, Debug)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

/// A data series for charting
#[derive(Clone, Debug)]
pub struct Series {
    pub name: String,
    pub data: Vec<DataPoint>,
    pub color: String,
}

impl Series {
    pub fn new(name: &str, color: &str) -> Self {
        Self {
            name: name.to_string(),
            data: Vec::new(),
            color: color.to_string(),
        }
    }

    pub fn add(&mut self, timestamp: DateTime<Utc>, value: f64) {
        self.data.push(DataPoint { timestamp, value });
    }

    pub fn max_value(&self) -> f64 {
        self.data.iter().map(|d| d.value).fold(0.0, f64::max)
    }

    pub fn sum(&self) -> f64 {
        self.data.iter().map(|d| d.value).sum()
    }
}

/// Line chart renderer (like the tokens per day chart)
pub struct LineChart {
    pub title: String,
    pub series: Vec<Series>,
    pub width: usize,
    pub height: usize,
    pub y_label_width: usize,
}

impl LineChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            series: Vec::new(),
            width: 56,
            height: 8,
            y_label_width: 6,
        }
    }

    pub fn with_size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn add_series(&mut self, series: Series) {
        self.series.push(series);
    }

    /// Render the chart to a string
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Title
        output.push_str(&format!("  {}\n", self.title.bold()));

        if self.series.is_empty() || self.series.iter().all(|s| s.data.is_empty()) {
            output.push_str("    No data available\n");
            return output;
        }

        // Find global max and date range
        let max_val = self
            .series
            .iter()
            .map(|s| s.max_value())
            .fold(0.0, f64::max);

        let all_points: Vec<&DataPoint> = self.series.iter().flat_map(|s| &s.data).collect();

        if all_points.is_empty() {
            output.push_str("    No data points\n");
            return output;
        }

        let min_time = all_points
            .iter()
            .map(|d| d.timestamp)
            .min()
            .unwrap_or(Utc::now());
        let max_time = all_points
            .iter()
            .map(|d| d.timestamp)
            .max()
            .unwrap_or(Utc::now());

        // Create the chart grid
        let chart_width = self.width - self.y_label_width - 2;
        let chart_height = self.height;

        // Initialize grid with spaces
        let mut grid: Vec<Vec<char>> = vec![vec![' '; chart_width]; chart_height];

        // Plot each series
        for (series_idx, series) in self.series.iter().enumerate() {
            let color_idx = series_idx % COLORS.len();
            let _color = COLORS[color_idx];

            for (i, point) in series.data.iter().enumerate() {
                // Calculate x position based on time
                let time_range = (max_time - min_time).num_seconds().max(1) as f64;
                let time_offset = (point.timestamp - min_time).num_seconds() as f64;
                let x = ((time_offset / time_range) * (chart_width - 1) as f64) as usize;
                let x = x.min(chart_width - 1);

                // Calculate y position based on value
                let y = if max_val > 0.0 {
                    ((1.0 - point.value / max_val) * (chart_height - 1) as f64) as usize
                } else {
                    chart_height - 1
                };
                let y = y.min(chart_height - 1);

                // Determine the character based on neighbors
                let prev_y = if i > 0 {
                    let prev = &series.data[i - 1];
                    let prev_val = prev.value / max_val.max(1.0);
                    Some(((1.0 - prev_val) * (chart_height - 1) as f64) as usize)
                } else {
                    None
                };

                let next_y = if i < series.data.len() - 1 {
                    let next = &series.data[i + 1];
                    let next_val = next.value / max_val.max(1.0);
                    Some(((1.0 - next_val) * (chart_height - 1) as f64) as usize)
                } else {
                    None
                };

                // Choose character based on direction
                let ch = match (prev_y, next_y) {
                    (Some(py), Some(ny)) if py > y && ny > y => '╰', // valley going up both sides
                    (Some(py), Some(ny)) if py < y && ny < y => '╭', // peak
                    (Some(py), Some(ny)) if py > y && ny < y => '╯', // going down then up
                    (Some(py), Some(ny)) if py < y && ny > y => '╮', // going up then down
                    (Some(py), None) if py < y => '╯',
                    (Some(py), None) if py > y => '╮',
                    (None, Some(ny)) if ny < y => '╰',
                    (None, Some(ny)) if ny > y => '╭',
                    _ => '│',
                };

                grid[y][x] = ch;

                // Draw connecting lines
                if let Some(py) = prev_y {
                    if i > 0 {
                        let prev_point = &series.data[i - 1];
                        let prev_time_offset = (prev_point.timestamp - min_time).num_seconds() as f64;
                        let prev_x =
                            ((prev_time_offset / time_range) * (chart_width - 1) as f64) as usize;
                        let prev_x = prev_x.min(chart_width - 1);

                        // Draw horizontal line between points
                        for dx in (prev_x + 1)..x {
                            if grid[y][dx] == ' ' {
                                grid[y][dx] = '─';
                            }
                        }

                        // Draw vertical connection if needed
                        let min_y = y.min(py);
                        let max_y = y.max(py);
                        for dy in (min_y + 1)..max_y {
                            if dy < chart_height {
                                let connect_x = if y < py { x } else { prev_x };
                                if connect_x < chart_width && grid[dy][connect_x] == ' ' {
                                    grid[dy][connect_x] = '│';
                                }
                            }
                        }
                    }
                }
            }
        }

        // Render y-axis labels and grid
        let y_labels = calculate_y_labels(max_val, chart_height);

        for (row_idx, row) in grid.iter().enumerate() {
            let y_label = &y_labels[row_idx];
            output.push_str(&format!("{:>width$} ", y_label, width = self.y_label_width));

            if row_idx == 0 {
                output.push_str("┼");
            } else {
                output.push_str("┤");
            }

            // Color the line based on series
            let row_str: String = row.iter().collect();
            if !self.series.is_empty() {
                let colored_row = match self.series[0].color.as_str() {
                    "cyan" => row_str.cyan(),
                    "magenta" => row_str.magenta(),
                    "yellow" => row_str.yellow(),
                    "green" => row_str.green(),
                    "blue" => row_str.blue(),
                    "red" => row_str.red(),
                    _ => row_str.white(),
                };
                output.push_str(&format!("{}", colored_row));
            } else {
                output.push_str(&row_str);
            }
            output.push('\n');
        }

        // X-axis
        output.push_str(&format!(
            "{:>width$} └",
            "",
            width = self.y_label_width
        ));
        output.push_str(&"─".repeat(chart_width));
        output.push('\n');

        // X-axis labels (dates)
        let date_labels = calculate_x_labels(min_time, max_time, chart_width);
        output.push_str(&format!(
            "{:>width$}  {}",
            "",
            date_labels,
            width = self.y_label_width
        ));
        output.push('\n');

        // Legend
        if self.series.len() > 1 {
            output.push_str("  ");
            for series in &self.series {
                let bullet = match series.color.as_str() {
                    "cyan" => "●".cyan(),
                    "magenta" => "●".magenta(),
                    "yellow" => "●".yellow(),
                    "green" => "●".green(),
                    "blue" => "●".blue(),
                    "red" => "●".red(),
                    _ => "●".white(),
                };
                output.push_str(&format!("{} {} · ", bullet, series.name));
            }
            output.push('\n');
        }

        output
    }
}

/// Format large numbers with K/M/B suffixes
fn format_number(n: f64) -> String {
    if n >= 1_000_000_000.0 {
        format!("{:.1}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.0}k", n / 1_000.0)
    } else if n >= 1.0 {
        format!("{:.0}", n)
    } else {
        format!("{:.1}", n)
    }
}

/// Calculate Y-axis labels
fn calculate_y_labels(max_val: f64, height: usize) -> Vec<String> {
    let mut labels = Vec::with_capacity(height);
    for i in 0..height {
        let val = max_val * (1.0 - i as f64 / (height - 1).max(1) as f64);
        labels.push(format_number(val));
    }
    labels
}

/// Calculate X-axis date labels
fn calculate_x_labels(min_time: DateTime<Utc>, max_time: DateTime<Utc>, width: usize) -> String {
    let mut labels = String::new();

    // Show 3-4 date labels spread across the width
    let num_labels = 4;
    let label_spacing = width / num_labels;

    for i in 0..num_labels {
        let t = min_time
            + chrono::Duration::seconds(
                ((max_time - min_time).num_seconds() as f64 * i as f64 / (num_labels - 1) as f64)
                    as i64,
            );
        let label = format!("{} {}", month_abbrev(t.month()), t.day());

        if i == 0 {
            labels.push_str(&label);
        } else {
            let padding = label_spacing.saturating_sub(label.len() / 2);
            labels.push_str(&" ".repeat(padding));
            labels.push_str(&label);
        }
    }

    labels
}

fn month_abbrev(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

/// Horizontal bar chart for breakdown displays
pub struct BarChart {
    pub title: String,
    pub items: Vec<BarItem>,
    pub width: usize,
    pub show_percentages: bool,
}

#[derive(Clone, Debug)]
pub struct BarItem {
    pub label: String,
    pub value: f64,
    pub sub_label: Option<String>,
    pub color: String,
}

impl BarChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
            width: 40,
            show_percentages: true,
        }
    }

    pub fn add(&mut self, label: &str, value: f64, color: &str) {
        self.items.push(BarItem {
            label: label.to_string(),
            value,
            sub_label: None,
            color: color.to_string(),
        });
    }

    pub fn add_with_detail(&mut self, label: &str, value: f64, detail: &str, color: &str) {
        self.items.push(BarItem {
            label: label.to_string(),
            value,
            sub_label: Some(detail.to_string()),
            color: color.to_string(),
        });
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        if !self.title.is_empty() {
            output.push_str(&format!("  {}\n\n", self.title.bold()));
        }

        if self.items.is_empty() {
            output.push_str("  No data available\n");
            return output;
        }

        let total: f64 = self.items.iter().map(|i| i.value).sum();
        let max_label_len = self.items.iter().map(|i| i.label.len()).max().unwrap_or(10);

        // Two-column layout for many items
        let use_columns = self.items.len() > 4;
        let bar_width = if use_columns { 20 } else { self.width };

        if use_columns {
            // Two-column grid layout
            let items_per_col = (self.items.len() + 1) / 2;

            for row in 0..items_per_col {
                let left_idx = row;
                let right_idx = row + items_per_col;

                // Left column
                if left_idx < self.items.len() {
                    let item = &self.items[left_idx];
                    output.push_str(&format_bar_item(item, total, bar_width, max_label_len));
                }

                // Right column
                if right_idx < self.items.len() {
                    output.push_str("  ");
                    let item = &self.items[right_idx];
                    output.push_str(&format_bar_item(item, total, bar_width, max_label_len));
                }

                output.push('\n');

                // Sub-labels
                if left_idx < self.items.len() {
                    if let Some(ref sub) = self.items[left_idx].sub_label {
                        output.push_str(&format!("    {}\n", sub.dimmed()));
                    }
                }
            }
        } else {
            // Single column layout
            for item in &self.items {
                let pct = if total > 0.0 {
                    item.value / total * 100.0
                } else {
                    0.0
                };

                let bullet = colorize_bullet(&item.color);
                let bar_filled = ((pct / 100.0) * bar_width as f64) as usize;
                let bar = format!(
                    "{}{}",
                    "█".repeat(bar_filled),
                    "░".repeat(bar_width - bar_filled)
                );

                let colored_bar = colorize_text(&bar, &item.color);

                output.push_str(&format!(
                    "  {} {:width$} ({:5.1}%)\n",
                    bullet,
                    item.label,
                    pct,
                    width = max_label_len
                ));
                output.push_str(&format!("    {}\n", colored_bar));

                if let Some(ref sub) = item.sub_label {
                    output.push_str(&format!("    {}\n", sub.dimmed()));
                }
            }
        }

        output
    }
}

fn format_bar_item(item: &BarItem, total: f64, _bar_width: usize, _max_label: usize) -> String {
    let pct = if total > 0.0 {
        item.value / total * 100.0
    } else {
        0.0
    };

    let bullet = colorize_bullet(&item.color);

    format!("  {} {} ({:.1}%)", bullet, item.label, pct)
}

fn colorize_bullet(color: &str) -> colored::ColoredString {
    match color {
        "cyan" => "●".cyan(),
        "magenta" => "●".magenta(),
        "yellow" => "●".yellow(),
        "green" => "●".green(),
        "blue" => "●".blue(),
        "red" => "●".red(),
        "white" => "●".white(),
        _ => "●".white(),
    }
}

fn colorize_text(text: &str, color: &str) -> colored::ColoredString {
    match color {
        "cyan" => text.cyan(),
        "magenta" => text.magenta(),
        "yellow" => text.yellow(),
        "green" => text.green(),
        "blue" => text.blue(),
        "red" => text.red(),
        _ => text.white(),
    }
}

/// Sparkline for compact trend visualization
pub struct Sparkline {
    values: Vec<f64>,
    width: usize,
}

impl Sparkline {
    pub fn new(values: &[f64]) -> Self {
        Self {
            values: values.to_vec(),
            width: values.len(),
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn render(&self) -> String {
        if self.values.is_empty() {
            return "─".repeat(self.width);
        }

        let min = self.values.iter().cloned().fold(f64::MAX, f64::min);
        let max = self.values.iter().cloned().fold(f64::MIN, f64::max);
        let range = (max - min).max(0.001);

        // Resample if needed
        let resampled = if self.values.len() != self.width {
            resample(&self.values, self.width)
        } else {
            self.values.clone()
        };

        resampled
            .iter()
            .map(|&v| {
                let normalized = ((v - min) / range).clamp(0.0, 1.0);
                let idx = (normalized * 7.0) as usize;
                SPARK_CHARS[idx.min(7)]
            })
            .collect()
    }

    pub fn render_colored(&self, color: &str) -> colored::ColoredString {
        let spark = self.render();
        colorize_text(&spark, color)
    }
}

fn resample(values: &[f64], target_len: usize) -> Vec<f64> {
    if values.is_empty() || target_len == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(target_len);
    let ratio = values.len() as f64 / target_len as f64;

    for i in 0..target_len {
        let src_idx = (i as f64 * ratio) as usize;
        result.push(values[src_idx.min(values.len() - 1)]);
    }

    result
}

/// Statistics panel with key metrics
pub struct StatsPanel {
    pub title: String,
    pub metrics: Vec<(String, String, Option<String>)>, // (label, value, trend)
}

impl StatsPanel {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            metrics: Vec::new(),
        }
    }

    pub fn add(&mut self, label: &str, value: &str) {
        self.metrics.push((label.to_string(), value.to_string(), None));
    }

    pub fn add_with_trend(&mut self, label: &str, value: &str, trend: &str) {
        self.metrics.push((
            label.to_string(),
            value.to_string(),
            Some(trend.to_string()),
        ));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        // Box drawing
        let max_label = self
            .metrics
            .iter()
            .map(|(l, _, _)| l.len())
            .max()
            .unwrap_or(10);
        let max_value = self
            .metrics
            .iter()
            .map(|(_, v, _)| v.len())
            .max()
            .unwrap_or(10);
        let inner_width = max_label + max_value + 5;

        // Title
        output.push_str(&format!("  {}\n", self.title.bold()));

        // Top border
        output.push_str(&format!(
            "  {}{}{}",
            BOX_TL,
            BOX_H.to_string().repeat(inner_width),
            BOX_TR
        ));
        output.push('\n');

        // Metrics
        for (label, value, trend) in &self.metrics {
            let trend_str = trend
                .as_ref()
                .map(|t| {
                    if t.starts_with('+') || t.starts_with('↑') {
                        format!(" {}", t.green())
                    } else if t.starts_with('-') || t.starts_with('↓') {
                        format!(" {}", t.red())
                    } else {
                        format!(" {}", t.dimmed())
                    }
                })
                .unwrap_or_default();

            output.push_str(&format!(
                "  {} {:width$} : {}{}\n",
                BOX_V,
                label,
                value.cyan(),
                trend_str,
                width = max_label
            ));
        }

        // Bottom border
        output.push_str(&format!(
            "  {}{}{}",
            BOX_BL,
            BOX_H.to_string().repeat(inner_width),
            BOX_BR
        ));
        output.push('\n');

        output
    }
}

/// Tool usage breakdown like the /model output
pub struct ToolBreakdown {
    pub tools: Vec<ToolUsage>,
}

#[derive(Clone, Debug)]
pub struct ToolUsage {
    pub name: String,
    pub percentage: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub color: String,
}

impl ToolBreakdown {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn add(
        &mut self,
        name: &str,
        percentage: f64,
        input_tokens: u64,
        output_tokens: u64,
        color: &str,
    ) {
        self.tools.push(ToolUsage {
            name: name.to_string(),
            percentage,
            input_tokens,
            output_tokens,
            color: color.to_string(),
        });
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("  {}\n\n", "All tools".bold()));

        if self.tools.is_empty() {
            output.push_str("  No tool data available\n");
            return output;
        }

        // Two-column layout
        let items_per_col = (self.tools.len() + 1) / 2;

        for row in 0..items_per_col {
            let left_idx = row;
            let right_idx = row + items_per_col;

            // Left column
            if left_idx < self.tools.len() {
                let tool = &self.tools[left_idx];
                output.push_str(&self.format_tool_entry(tool));
            }

            // Right column
            if right_idx < self.tools.len() {
                output.push_str("    ");
                let tool = &self.tools[right_idx];
                output.push_str(&self.format_tool_entry(tool));
            }

            output.push('\n');

            // Token details for left column
            if left_idx < self.tools.len() {
                let tool = &self.tools[left_idx];
                output.push_str(&format!(
                    "    In: {} · Out: {}",
                    format_tokens(tool.input_tokens).dimmed(),
                    format_tokens(tool.output_tokens).dimmed()
                ));
            }

            // Token details for right column
            if right_idx < self.tools.len() {
                let tool = &self.tools[right_idx];
                output.push_str(&format!(
                    "    In: {} · Out: {}",
                    format_tokens(tool.input_tokens).dimmed(),
                    format_tokens(tool.output_tokens).dimmed()
                ));
            }

            output.push('\n');
        }

        output
    }

    fn format_tool_entry(&self, tool: &ToolUsage) -> String {
        let bullet = colorize_bullet(&tool.color);
        format!("{} {} ({:.1}%)", bullet, tool.name, tool.percentage)
    }
}

impl Default for ToolBreakdown {
    fn default() -> Self {
        Self::new()
    }
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}m", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}k", tokens as f64 / 1_000.0)
    } else {
        format!("{}", tokens)
    }
}

/// Activity Heatmap - GitHub-style yearly contribution graph
/// Shows month x day-of-week activity matrix
pub struct ActivityHeatmap {
    /// Data indexed by (week_number, day_of_week) -> intensity value
    pub data: HashMap<(u32, u32), f64>,
    /// Number of weeks to show (default 52)
    pub weeks: u32,
}

/// Intensity characters for heatmap (5 levels)
const HEAT_CHARS: [char; 5] = ['·', '░', '▒', '▓', '█'];

impl ActivityHeatmap {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            weeks: 52,
        }
    }

    pub fn with_weeks(mut self, weeks: u32) -> Self {
        self.weeks = weeks;
        self
    }

    pub fn set(&mut self, week: u32, day: u32, value: f64) {
        self.data.insert((week, day), value);
    }

    /// Set from a date and value
    pub fn set_date(&mut self, date: DateTime<Utc>, value: f64) {
        let week = date.iso_week().week() as u32;
        let day = date.weekday().num_days_from_monday();
        self.data.insert((week, day), value);
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        // Month headers
        output.push_str("      ");
        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec", "Jan"];
        let weeks_per_month = self.weeks / 12;
        for month in &months[..13] {
            output.push_str(month);
            if weeks_per_month > 3 {
                output.push_str(&" ".repeat(weeks_per_month as usize - 3));
            } else {
                output.push(' ');
            }
        }
        output.push('\n');

        let days = ["", "Mon", "", "Wed", "", "Fri", ""];
        let max_val = self.data.values().cloned().fold(0.0f64, f64::max);

        // Each day row
        for day_idx in 0..7u32 {
            output.push_str(&format!("  {:3} ", days[day_idx as usize]));

            for week in 0..self.weeks {
                let val = self.data.get(&(week, day_idx)).unwrap_or(&0.0);
                let intensity = if max_val > 0.0 {
                    (val / max_val * 4.0) as usize
                } else {
                    0
                };
                let ch = HEAT_CHARS[intensity.min(4)];

                // Color by intensity
                let colored_ch = if intensity >= 4 {
                    format!("{}", ch).green().bold()
                } else if intensity >= 3 {
                    format!("{}", ch).green()
                } else if intensity >= 2 {
                    format!("{}", ch).yellow()
                } else if intensity >= 1 {
                    format!("{}", ch).cyan()
                } else {
                    format!("{}", ch).dimmed()
                };

                output.push_str(&format!("{}", colored_ch));
            }

            output.push('\n');
        }

        // Legend
        output.push('\n');
        output.push_str("      Less ");
        for ch in HEAT_CHARS {
            output.push(ch);
            output.push(' ');
        }
        output.push_str("More\n");

        output
    }
}

impl Default for ActivityHeatmap {
    fn default() -> Self {
        Self::new()
    }
}

/// Stats Card - Key metrics in a beautiful card format
pub struct StatsCard {
    pub rows: Vec<Vec<(String, String)>>, // Rows of (label, value) pairs
}

impl StatsCard {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Add a row of metric pairs (displayed horizontally)
    pub fn add_row(&mut self, pairs: Vec<(&str, &str)>) {
        self.rows.push(
            pairs
                .into_iter()
                .map(|(l, v)| (l.to_string(), v.to_string()))
                .collect(),
        );
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        for row in &self.rows {
            output.push_str("  ");
            for (idx, (label, value)) in row.iter().enumerate() {
                if idx > 0 {
                    output.push_str("    ");
                }
                output.push_str(&format!("{}: ", label));
                output.push_str(&format!("{}", value.cyan()));
            }
            output.push('\n');
        }

        output
    }
}

impl Default for StatsCard {
    fn default() -> Self {
        Self::new()
    }
}

/// Fun Fact - Whimsical comparison stats
pub struct FunFact {
    pub fact: String,
    pub source_period: String,
}

impl FunFact {
    pub fn new(fact: &str, period: &str) -> Self {
        Self {
            fact: fact.to_string(),
            source_period: period.to_string(),
        }
    }

    /// Generate fun token comparison
    pub fn token_comparison(tokens: u64) -> Self {
        let comparisons = [
            (77_000, "a short novel"),
            (350_000, "Harry Potter and the Philosopher's Stone"),
            (580_000, "The Great Gatsby"),
            (850_000, "Anna Karenina"),
            (1_200_000, "War and Peace"),
            (4_000_000, "the entire Lord of the Rings trilogy"),
            (10_000_000, "all Harry Potter books combined"),
            (50_000_000, "Wikipedia's featured articles"),
        ];

        let mut best_match = ("a tweet", 280u64);
        let mut multiplier = tokens as f64 / 280.0;

        for (book_tokens, name) in comparisons {
            let m = tokens as f64 / book_tokens as f64;
            if m >= 1.0 && m < multiplier {
                multiplier = m;
                best_match = (name, book_tokens);
            }
        }

        let fact = if multiplier >= 100.0 {
            format!("You've used ~{}x more tokens than {}", multiplier as u64, best_match.0)
        } else if multiplier >= 10.0 {
            format!("You've used ~{:.0}x more tokens than {}", multiplier, best_match.0)
        } else if multiplier >= 1.0 {
            format!("You've used ~{:.1}x the tokens of {}", multiplier, best_match.0)
        } else {
            format!("You've used {} tokens (keep going!)", format_tokens(tokens))
        };

        Self {
            fact,
            source_period: String::new(),
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("\n  {}\n", self.fact.italic()));
        if !self.source_period.is_empty() {
            output.push_str(&format!("  {}\n", self.source_period.dimmed()));
        }
        output
    }
}

/// Streak Counter - Visual streak display with flames
pub struct StreakCounter {
    pub current: u32,
    pub longest: u32,
    pub active_days: u32,
    pub total_days: u32,
    pub peak_hour: String,
}

impl StreakCounter {
    pub fn new(current: u32, longest: u32) -> Self {
        Self {
            current,
            longest,
            active_days: 0,
            total_days: 0,
            peak_hour: String::new(),
        }
    }

    pub fn with_activity(mut self, active: u32, total: u32) -> Self {
        self.active_days = active;
        self.total_days = total;
        self
    }

    pub fn with_peak(mut self, peak: &str) -> Self {
        self.peak_hour = peak.to_string();
        self
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        // Streak flames visualization
        let flames = if self.current >= 30 {
            "🔥🔥🔥🔥🔥".to_string()
        } else if self.current >= 14 {
            "🔥🔥🔥🔥".to_string()
        } else if self.current >= 7 {
            "🔥🔥🔥".to_string()
        } else if self.current >= 3 {
            "🔥🔥".to_string()
        } else if self.current >= 1 {
            "🔥".to_string()
        } else {
            "".to_string()
        };

        output.push_str(&format!(
            "  Current streak: {} days {}\n",
            format!("{}", self.current).cyan().bold(),
            flames
        ));
        output.push_str(&format!(
            "  Longest streak: {} days\n",
            format!("{}", self.longest).green()
        ));

        if self.total_days > 0 {
            let pct = (self.active_days as f64 / self.total_days as f64 * 100.0) as u32;
            output.push_str(&format!(
                "  Active days: {}/{} ({}%)\n",
                self.active_days, self.total_days, pct
            ));
        }

        if !self.peak_hour.is_empty() {
            output.push_str(&format!("  Peak hour: {}\n", self.peak_hour.yellow()));
        }

        output
    }
}

/// Histogram - Distribution visualization
pub struct Histogram {
    pub title: String,
    pub buckets: Vec<(String, u64)>,
    pub width: usize,
}

impl Histogram {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            buckets: Vec::new(),
            width: 40,
        }
    }

    pub fn add(&mut self, label: &str, count: u64) {
        self.buckets.push((label.to_string(), count));
    }

    /// Create histogram from values with auto-bucketing
    pub fn from_values(title: &str, values: &[f64], num_buckets: usize) -> Self {
        let mut hist = Self::new(title);

        if values.is_empty() {
            return hist;
        }

        let min = values.iter().cloned().fold(f64::MAX, f64::min);
        let max = values.iter().cloned().fold(f64::MIN, f64::max);
        let range = (max - min).max(0.001);
        let bucket_size = range / num_buckets as f64;

        let mut counts = vec![0u64; num_buckets];

        for &val in values {
            let bucket = ((val - min) / bucket_size) as usize;
            let bucket = bucket.min(num_buckets - 1);
            counts[bucket] += 1;
        }

        for (i, &count) in counts.iter().enumerate() {
            let low = min + i as f64 * bucket_size;
            let high = low + bucket_size;
            let label = if bucket_size >= 1.0 {
                format!("{:.0}-{:.0}", low, high)
            } else {
                format!("{:.1}-{:.1}", low, high)
            };
            hist.add(&label, count);
        }

        hist
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.buckets.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let max_count = self.buckets.iter().map(|(_, c)| *c).max().unwrap_or(1);
        let max_label_len = self.buckets.iter().map(|(l, _)| l.len()).max().unwrap_or(5);

        for (label, count) in &self.buckets {
            let bar_len = if max_count > 0 {
                (*count as f64 / max_count as f64 * self.width as f64) as usize
            } else {
                0
            };

            let bar = "█".repeat(bar_len);
            let colored_bar = if bar_len > self.width * 3 / 4 {
                bar.green()
            } else if bar_len > self.width / 2 {
                bar.cyan()
            } else if bar_len > self.width / 4 {
                bar.yellow()
            } else {
                bar.white()
            };

            output.push_str(&format!(
                "  {:>width$} │{} {}\n",
                label,
                colored_bar,
                count,
                width = max_label_len
            ));
        }

        output
    }
}

/// Progress Bar - Goal tracking visualization
pub struct ProgressBar {
    pub label: String,
    pub current: f64,
    pub target: f64,
    pub width: usize,
    pub show_percentage: bool,
}

impl ProgressBar {
    pub fn new(label: &str, current: f64, target: f64) -> Self {
        Self {
            label: label.to_string(),
            current,
            target,
            width: 30,
            show_percentage: true,
        }
    }

    pub fn render(&self) -> String {
        let pct = (self.current / self.target).min(1.0);
        let filled = (pct * self.width as f64) as usize;
        let empty = self.width - filled;

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        let colored_bar = if pct >= 1.0 {
            bar.green().bold()
        } else if pct >= 0.75 {
            bar.green()
        } else if pct >= 0.5 {
            bar.yellow()
        } else if pct >= 0.25 {
            bar.cyan()
        } else {
            bar.red()
        };

        let status = if pct >= 1.0 { "✓" } else { " " };

        if self.show_percentage {
            format!(
                "  {} {} {} ({:.0}%)\n",
                status,
                self.label,
                colored_bar,
                pct * 100.0
            )
        } else {
            format!(
                "  {} {} {} {}/{}\n",
                status,
                self.label,
                colored_bar,
                self.current as u64,
                self.target as u64
            )
        }
    }
}

/// Leaderboard - Ranked list with comparison bars
pub struct Leaderboard {
    pub title: String,
    pub entries: Vec<(String, f64, Option<String>)>, // (name, value, badge)
}

impl Leaderboard {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &str, value: f64) {
        self.entries.push((name.to_string(), value, None));
    }

    pub fn add_with_badge(&mut self, name: &str, value: f64, badge: &str) {
        self.entries
            .push((name.to_string(), value, Some(badge.to_string())));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.entries.is_empty() {
            output.push_str("  No entries\n");
            return output;
        }

        // Sort by value descending
        let mut sorted: Vec<_> = self.entries.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let max_val = sorted.first().map(|(_, v, _)| *v).unwrap_or(1.0);
        let max_name_len = sorted.iter().map(|(n, _, _)| n.len()).max().unwrap_or(10);

        for (rank, (name, value, badge)) in sorted.iter().enumerate() {
            let rank_str = match rank {
                0 => "🥇".to_string(),
                1 => "🥈".to_string(),
                2 => "🥉".to_string(),
                _ => format!("{:2}.", rank + 1),
            };

            let bar_width = 20;
            let bar_len = (value / max_val * bar_width as f64) as usize;
            let bar = "▓".repeat(bar_len);
            let colored_bar = match rank {
                0 => bar.yellow(),
                1 => bar.white(),
                2 => bar.red(),
                _ => bar.dimmed(),
            };

            let badge_str = badge
                .as_ref()
                .map(|b| format!(" {}", b))
                .unwrap_or_default();

            output.push_str(&format!(
                "  {} {:width$} {} {}{}\n",
                rank_str,
                name,
                colored_bar,
                format_number(*value),
                badge_str,
                width = max_name_len
            ));
        }

        output
    }
}

/// Calendar View - Monthly calendar with activity markers
pub struct CalendarView {
    pub year: i32,
    pub month: u32,
    pub data: HashMap<u32, f64>, // day -> activity level
}

impl CalendarView {
    pub fn new(year: i32, month: u32) -> Self {
        Self {
            year,
            month,
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, day: u32, value: f64) {
        self.data.insert(day, value);
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        let month_names = [
            "", "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December",
        ];

        output.push_str(&format!(
            "  {} {}\n",
            month_names[self.month as usize].bold(),
            self.year
        ));
        output.push_str("  Su Mo Tu We Th Fr Sa\n");

        // Get first day of month and days in month
        let first_day = chrono::NaiveDate::from_ymd_opt(self.year, self.month, 1);
        if first_day.is_none() {
            return output;
        }
        let first_day = first_day.unwrap();
        let weekday = first_day.weekday().num_days_from_sunday();

        let days_in_month = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.year % 4 == 0 && (self.year % 100 != 0 || self.year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        };

        let max_val = self.data.values().cloned().fold(0.0f64, f64::max);

        // Print leading spaces
        output.push_str("  ");
        for _ in 0..weekday {
            output.push_str("   ");
        }

        let mut current_weekday = weekday;
        for day in 1..=days_in_month {
            let val = self.data.get(&day).unwrap_or(&0.0);
            let intensity = if max_val > 0.0 { val / max_val } else { 0.0 };

            let day_str = format!("{:2}", day);
            let colored_day = if intensity > 0.8 {
                day_str.green().bold()
            } else if intensity > 0.5 {
                day_str.green()
            } else if intensity > 0.2 {
                day_str.yellow()
            } else if intensity > 0.0 {
                day_str.cyan()
            } else {
                day_str.dimmed()
            };

            output.push_str(&format!("{} ", colored_day));

            current_weekday += 1;
            if current_weekday >= 7 {
                output.push('\n');
                output.push_str("  ");
                current_weekday = 0;
            }
        }

        if current_weekday != 0 {
            output.push('\n');
        }

        output
    }
}

/// Time Distribution - Radial-style time breakdown
pub struct TimeDistribution {
    pub title: String,
    pub hours: [f64; 24],
}

impl TimeDistribution {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            hours: [0.0; 24],
        }
    }

    pub fn set(&mut self, hour: usize, value: f64) {
        if hour < 24 {
            self.hours[hour] = value;
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("  {}\n", self.title.bold()));

        let max_val = self.hours.iter().cloned().fold(0.0f64, f64::max);

        // Morning row (6-12)
        output.push_str("  Morning  ");
        for hour in 6..12 {
            let intensity = if max_val > 0.0 {
                (self.hours[hour] / max_val * 4.0) as usize
            } else {
                0
            };
            let ch = HEAT_CHARS[intensity.min(4)];
            output.push_str(&format!("{}", colorize_heat(ch, intensity)));
        }
        output.push('\n');

        // Afternoon row (12-18)
        output.push_str("  Afternoon");
        for hour in 12..18 {
            let intensity = if max_val > 0.0 {
                (self.hours[hour] / max_val * 4.0) as usize
            } else {
                0
            };
            let ch = HEAT_CHARS[intensity.min(4)];
            output.push_str(&format!("{}", colorize_heat(ch, intensity)));
        }
        output.push('\n');

        // Evening row (18-24)
        output.push_str("  Evening  ");
        for hour in 18..24 {
            let intensity = if max_val > 0.0 {
                (self.hours[hour] / max_val * 4.0) as usize
            } else {
                0
            };
            let ch = HEAT_CHARS[intensity.min(4)];
            output.push_str(&format!("{}", colorize_heat(ch, intensity)));
        }
        output.push('\n');

        // Night row (0-6)
        output.push_str("  Night    ");
        for hour in 0..6 {
            let intensity = if max_val > 0.0 {
                (self.hours[hour] / max_val * 4.0) as usize
            } else {
                0
            };
            let ch = HEAT_CHARS[intensity.min(4)];
            output.push_str(&format!("{}", colorize_heat(ch, intensity)));
        }
        output.push('\n');

        output
    }
}

fn colorize_heat(ch: char, intensity: usize) -> colored::ColoredString {
    let s = format!("{}", ch);
    match intensity {
        4 => s.green().bold(),
        3 => s.green(),
        2 => s.yellow(),
        1 => s.cyan(),
        _ => s.dimmed(),
    }
}

/// Comparison Chart - Side by side comparison of two values
pub struct ComparisonChart {
    pub left_label: String,
    pub left_value: f64,
    pub right_label: String,
    pub right_value: f64,
    pub unit: String,
}

impl ComparisonChart {
    pub fn new(left: &str, left_val: f64, right: &str, right_val: f64) -> Self {
        Self {
            left_label: left.to_string(),
            left_value: left_val,
            right_label: right.to_string(),
            right_value: right_val,
            unit: String::new(),
        }
    }

    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.to_string();
        self
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        let total = self.left_value + self.right_value;
        let left_pct = if total > 0.0 {
            self.left_value / total
        } else {
            0.5
        };
        let right_pct = 1.0 - left_pct;

        let bar_width = 30;
        let left_bar = (left_pct * bar_width as f64) as usize;
        let right_bar = bar_width - left_bar;

        let left_bar_str = "█".repeat(left_bar);
        let right_bar_str = "█".repeat(right_bar);

        output.push_str(&format!(
            "  {} vs {}\n",
            self.left_label.cyan(),
            self.right_label.magenta()
        ));

        output.push_str(&format!(
            "  {}{}\n",
            left_bar_str.cyan(),
            right_bar_str.magenta()
        ));

        output.push_str(&format!(
            "  {}{} ({:.0}%)    {}{} ({:.0}%)\n",
            format_number(self.left_value),
            if self.unit.is_empty() {
                "".to_string()
            } else {
                format!(" {}", self.unit)
            },
            left_pct * 100.0,
            format_number(self.right_value),
            if self.unit.is_empty() {
                "".to_string()
            } else {
                format!(" {}", self.unit)
            },
            right_pct * 100.0,
        ));

        output
    }
}

/// Trend Arrow - Shows trend direction with arrow
pub fn trend_arrow(current: f64, previous: f64) -> colored::ColoredString {
    let pct_change = if previous != 0.0 {
        (current - previous) / previous * 100.0
    } else if current > 0.0 {
        100.0
    } else {
        0.0
    };

    if pct_change > 10.0 {
        format!("↑ +{:.0}%", pct_change).green().bold()
    } else if pct_change > 0.0 {
        format!("↑ +{:.0}%", pct_change).green()
    } else if pct_change < -10.0 {
        format!("↓ {:.0}%", pct_change).red().bold()
    } else if pct_change < 0.0 {
        format!("↓ {:.0}%", pct_change).red()
    } else {
        "→ 0%".dimmed()
    }
}

/// Format duration in human-friendly format
pub fn format_duration_human(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

// ═══════════════════════════════════════════════════════════════
// ADDITIONAL VISUALIZATIONS - Part 2
// ═══════════════════════════════════════════════════════════════

/// Gauge - Semicircular gauge visualization
pub struct Gauge {
    pub label: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub thresholds: Vec<(f64, &'static str)>, // (threshold, color)
}

impl Gauge {
    pub fn new(label: &str, value: f64, min: f64, max: f64) -> Self {
        Self {
            label: label.to_string(),
            value,
            min,
            max,
            thresholds: vec![
                (0.25, "red"),
                (0.5, "yellow"),
                (0.75, "cyan"),
                (1.0, "green"),
            ],
        }
    }

    pub fn with_thresholds(mut self, thresholds: Vec<(f64, &'static str)>) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        let pct = ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0);

        // Determine color based on thresholds
        let color = self.thresholds
            .iter()
            .find(|(t, _)| pct <= *t)
            .map(|(_, c)| *c)
            .unwrap_or("white");

        // ASCII gauge (20 segments)
        let segments = 20;
        let filled = (pct * segments as f64) as usize;

        // Top arc
        output.push_str(&format!("  {}\n", self.label.bold()));
        output.push_str("       ╭──────────────────────╮\n");

        // Gauge bar
        output.push_str("      │");
        for i in 0..segments {
            let ch = if i < filled { "█" } else { "░" };
            output.push_str(&colorize_text(ch, color).to_string());
        }
        output.push_str("│\n");

        // Bottom with value
        output.push_str("       ╰──────────────────────╯\n");

        let value_str = format!("{:.1}", self.value);
        let pct_str = format!("{:.0}%", pct * 100.0);
        let padding = 11 - value_str.len() / 2;
        output.push_str(&format!(
            "{:>width$}{} ({})\n",
            "",
            colorize_text(&value_str, color),
            pct_str.dimmed(),
            width = padding
        ));

        output
    }
}

/// Donut Chart - Circular percentage visualization
pub struct DonutChart {
    pub title: String,
    pub segments: Vec<(String, f64, String)>, // (label, value, color)
    pub size: usize, // 1=small, 2=medium, 3=large
}

impl DonutChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            segments: Vec::new(),
            size: 2,
        }
    }

    pub fn add(&mut self, label: &str, value: f64, color: &str) {
        self.segments.push((label.to_string(), value, color.to_string()));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.segments.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let total: f64 = self.segments.iter().map(|(_, v, _)| v).sum();

        // ASCII donut representation
        let donut_chars = [
            "    ╭───────╮    ",
            "  ╭─┘       └─╮  ",
            " │             │ ",
            " │      ●      │ ",
            " │             │ ",
            "  ╰─╮       ╭─╯  ",
            "    ╰───────╯    ",
        ];

        for line in &donut_chars {
            output.push_str(&format!("  {}\n", line));
        }

        output.push('\n');

        // Legend with percentages
        for (label, value, color) in &self.segments {
            let pct = if total > 0.0 { value / total * 100.0 } else { 0.0 };
            let bullet = colorize_bullet(color);
            output.push_str(&format!(
                "  {} {} ({:.1}%) - {}\n",
                bullet,
                label,
                pct,
                format_number(*value)
            ));
        }

        output
    }
}

/// Bullet Chart - Progress with target and ranges
pub struct BulletChart {
    pub label: String,
    pub value: f64,
    pub target: f64,
    pub ranges: Vec<(f64, &'static str)>, // (threshold %, color)
    pub width: usize,
}

impl BulletChart {
    pub fn new(label: &str, value: f64, target: f64) -> Self {
        Self {
            label: label.to_string(),
            value,
            target,
            ranges: vec![
                (0.33, "red"),
                (0.66, "yellow"),
                (1.0, "green"),
            ],
            width: 40,
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        let max_val = self.target * 1.2; // 120% of target
        let value_pos = ((self.value / max_val) * self.width as f64) as usize;
        let target_pos = ((self.target / max_val) * self.width as f64) as usize;

        output.push_str(&format!("  {}\n", self.label));
        output.push_str("  ");

        // Draw background ranges
        for i in 0..self.width {
            let pos_pct = i as f64 / self.width as f64;
            let range_color = self.ranges
                .iter()
                .find(|(t, _)| pos_pct <= *t)
                .map(|(_, c)| *c)
                .unwrap_or("white");

            let ch = if i == target_pos {
                "│" // Target marker
            } else if i < value_pos {
                "█" // Value bar
            } else {
                "░" // Background
            };

            if i < value_pos {
                output.push_str(&colorize_text(ch, "white").to_string());
            } else {
                output.push_str(&colorize_text(ch, range_color).to_string());
            }
        }

        output.push_str(&format!(
            " {} (target: {})\n",
            format_number(self.value),
            format_number(self.target)
        ));

        output
    }
}

/// Funnel Chart - Conversion funnel visualization
pub struct FunnelChart {
    pub title: String,
    pub stages: Vec<(String, f64)>,
}

impl FunnelChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            stages: Vec::new(),
        }
    }

    pub fn add(&mut self, label: &str, value: f64) {
        self.stages.push((label.to_string(), value));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.stages.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let max_val = self.stages.first().map(|(_, v)| *v).unwrap_or(1.0);
        let max_width = 40;

        for (i, (label, value)) in self.stages.iter().enumerate() {
            let width = ((value / max_val) * max_width as f64) as usize;
            let padding = (max_width - width) / 2;

            let bar = "█".repeat(width);
            let color = match i {
                0 => "cyan",
                1 => "blue",
                2 => "magenta",
                3 => "yellow",
                _ => "green",
            };

            // Calculate conversion rate from previous stage
            let conv_rate = if i > 0 {
                let prev_val = self.stages[i - 1].1;
                if prev_val > 0.0 {
                    format!(" ({:.1}%)", value / prev_val * 100.0)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            output.push_str(&format!("{:>width$}", "", width = padding + 2));
            output.push_str(&colorize_text(&bar, color).to_string());
            output.push('\n');

            output.push_str(&format!(
                "  {:>12} : {}{}\n",
                label,
                format_number(*value),
                conv_rate.dimmed()
            ));
        }

        output
    }
}

/// Box Plot - Statistical distribution visualization
pub struct BoxPlot {
    pub label: String,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub width: usize,
}

impl BoxPlot {
    pub fn new(label: &str, data: &[f64]) -> Self {
        let mut sorted: Vec<f64> = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = sorted.len();
        let (min, max, median, q1, q3) = if len == 0 {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        } else {
            let min = sorted[0];
            let max = sorted[len - 1];
            let median = if len % 2 == 0 {
                (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
            } else {
                sorted[len / 2]
            };
            let q1 = sorted[len / 4];
            let q3 = sorted[3 * len / 4];
            (min, max, median, q1, q3)
        };

        Self {
            label: label.to_string(),
            min,
            q1,
            median,
            q3,
            max,
            width: 50,
        }
    }

    pub fn from_stats(label: &str, min: f64, q1: f64, median: f64, q3: f64, max: f64) -> Self {
        Self {
            label: label.to_string(),
            min,
            q1,
            median,
            q3,
            max,
            width: 50,
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        let range = self.max - self.min;
        if range <= 0.0 {
            output.push_str(&format!("  {} [no variance]\n", self.label));
            return output;
        }

        let scale = |v: f64| ((v - self.min) / range * self.width as f64) as usize;

        let min_pos = 0;
        let q1_pos = scale(self.q1);
        let med_pos = scale(self.median);
        let q3_pos = scale(self.q3);
        let max_pos = self.width;

        output.push_str(&format!("  {}\n  ", self.label));

        for i in 0..=self.width {
            let ch = if i == min_pos || i == max_pos {
                "│"
            } else if i == med_pos {
                "┃"
            } else if i > min_pos && i < q1_pos {
                "─"
            } else if i >= q1_pos && i <= q3_pos {
                "█"
            } else if i > q3_pos && i < max_pos {
                "─"
            } else {
                " "
            };

            let colored = if i == med_pos {
                ch.yellow().bold().to_string()
            } else if i >= q1_pos && i <= q3_pos {
                ch.cyan().to_string()
            } else {
                ch.dimmed().to_string()
            };
            output.push_str(&colored);
        }

        output.push_str(&format!(
            "\n  {:<6} {:<6} {:<6} {:<6} {:>6}\n",
            format_number(self.min),
            format_number(self.q1),
            format_number(self.median).yellow(),
            format_number(self.q3),
            format_number(self.max)
        ));

        output
    }
}

/// Waterfall Chart - Cumulative increases and decreases
pub struct WaterfallChart {
    pub title: String,
    pub items: Vec<(String, f64, bool)>, // (label, value, is_total)
}

impl WaterfallChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, label: &str, value: f64) {
        self.items.push((label.to_string(), value, false));
    }

    pub fn add_total(&mut self, label: &str) {
        let sum: f64 = self.items.iter().map(|(_, v, _)| v).sum();
        self.items.push((label.to_string(), sum, true));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.items.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let max_abs = self.items
            .iter()
            .map(|(_, v, _)| v.abs())
            .fold(0.0f64, f64::max);

        let bar_width = 30;
        let mut running_total = 0.0f64;

        for (label, value, is_total) in &self.items {
            let display_val = if *is_total { *value } else { *value };
            let bar_len = ((display_val.abs() / max_abs) * bar_width as f64) as usize;

            let (bar, color) = if *is_total {
                ("═".repeat(bar_len), "cyan")
            } else if *value >= 0.0 {
                ("█".repeat(bar_len), "green")
            } else {
                ("█".repeat(bar_len), "red")
            };

            let prefix = if *value >= 0.0 && !*is_total { "+" } else { "" };

            output.push_str(&format!(
                "  {:>12} │{} {}{}\n",
                label,
                colorize_text(&bar, color),
                prefix,
                format_number(display_val)
            ));

            if !*is_total {
                running_total += value;
            }
        }

        output
    }
}

/// Radar Chart - Multi-axis comparison (simplified ASCII)
pub struct RadarChart {
    pub title: String,
    pub axes: Vec<String>,
    pub values: Vec<f64>, // 0.0 to 1.0 normalized
}

impl RadarChart {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            axes: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn add(&mut self, axis: &str, value: f64) {
        self.axes.push(axis.to_string());
        self.values.push(value.clamp(0.0, 1.0));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.axes.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        // Simple horizontal bar representation for each axis
        let max_label_len = self.axes.iter().map(|a| a.len()).max().unwrap_or(10);
        let bar_width = 20;

        for (axis, value) in self.axes.iter().zip(self.values.iter()) {
            let filled = (*value * bar_width as f64) as usize;
            let empty = bar_width - filled;

            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
            let color = if *value >= 0.8 {
                "green"
            } else if *value >= 0.5 {
                "cyan"
            } else if *value >= 0.3 {
                "yellow"
            } else {
                "red"
            };

            output.push_str(&format!(
                "  {:>width$} │{} {:.0}%\n",
                axis,
                colorize_text(&bar, color),
                value * 100.0,
                width = max_label_len
            ));
        }

        output
    }
}

/// Matrix Heatmap - 2D data grid with color intensity
pub struct MatrixHeatmap {
    pub title: String,
    pub row_labels: Vec<String>,
    pub col_labels: Vec<String>,
    pub data: Vec<Vec<f64>>,
}

impl MatrixHeatmap {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            row_labels: Vec::new(),
            col_labels: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn set_labels(&mut self, rows: Vec<&str>, cols: Vec<&str>) {
        self.row_labels = rows.into_iter().map(|s| s.to_string()).collect();
        self.col_labels = cols.into_iter().map(|s| s.to_string()).collect();
    }

    pub fn set_data(&mut self, data: Vec<Vec<f64>>) {
        self.data = data;
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.data.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let max_val = self.data
            .iter()
            .flat_map(|row| row.iter())
            .cloned()
            .fold(0.0f64, f64::max);

        let row_label_width = self.row_labels.iter().map(|l| l.len()).max().unwrap_or(3);

        // Column headers
        output.push_str(&format!("{:>width$} ", "", width = row_label_width + 2));
        for col in &self.col_labels {
            output.push_str(&format!("{:>3} ", &col[..col.len().min(3)]));
        }
        output.push('\n');

        // Data rows
        for (row_idx, row) in self.data.iter().enumerate() {
            let row_label = self.row_labels.get(row_idx).map(|s| s.as_str()).unwrap_or("");
            output.push_str(&format!("  {:>width$} ", row_label, width = row_label_width));

            for val in row {
                let intensity = if max_val > 0.0 { val / max_val } else { 0.0 };
                let ch = if intensity >= 0.8 {
                    "██"
                } else if intensity >= 0.6 {
                    "▓▓"
                } else if intensity >= 0.4 {
                    "▒▒"
                } else if intensity >= 0.2 {
                    "░░"
                } else {
                    "··"
                };

                let colored = if intensity >= 0.8 {
                    ch.green().bold().to_string()
                } else if intensity >= 0.6 {
                    ch.green().to_string()
                } else if intensity >= 0.4 {
                    ch.yellow().to_string()
                } else if intensity >= 0.2 {
                    ch.cyan().to_string()
                } else {
                    ch.dimmed().to_string()
                };
                output.push_str(&format!("{} ", colored));
            }
            output.push('\n');
        }

        output
    }
}

/// Gantt Chart - Timeline/schedule visualization
pub struct GanttChart {
    pub title: String,
    pub tasks: Vec<(String, u32, u32, String)>, // (name, start, duration, color)
    pub total_units: u32,
    pub unit_label: String,
}

impl GanttChart {
    pub fn new(title: &str, total_units: u32) -> Self {
        Self {
            title: title.to_string(),
            tasks: Vec::new(),
            total_units,
            unit_label: "day".to_string(),
        }
    }

    pub fn with_unit_label(mut self, label: &str) -> Self {
        self.unit_label = label.to_string();
        self
    }

    pub fn add(&mut self, name: &str, start: u32, duration: u32, color: &str) {
        self.tasks.push((name.to_string(), start, duration, color.to_string()));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.tasks.is_empty() {
            output.push_str("  No tasks\n");
            return output;
        }

        let max_name_len = self.tasks.iter().map(|(n, _, _, _)| n.len()).max().unwrap_or(10);
        let chart_width = 40;
        let scale = chart_width as f64 / self.total_units as f64;

        // Header with time markers
        let mut header = format!("{:>width$} │", "", width = max_name_len + 2);
        let header_base_len = header.len();
        for i in 0..=4 {
            let mark = (self.total_units as f64 * i as f64 / 4.0) as u32;
            let target_pos = header_base_len + (chart_width as f64 * i as f64 / 4.0) as usize;
            let current_len = header.len();
            if target_pos > current_len {
                header.push_str(&" ".repeat(target_pos - current_len));
            }
            header.push_str(&format!("{}", mark));
        }
        output.push_str(&header);
        output.push('\n');

        // Task bars
        for (name, start, duration, color) in &self.tasks {
            let start_pos = (*start as f64 * scale) as usize;
            let bar_len = (*duration as f64 * scale).max(1.0) as usize;

            output.push_str(&format!("  {:>width$} │", name, width = max_name_len));
            output.push_str(&" ".repeat(start_pos));

            let bar = "█".repeat(bar_len);
            output.push_str(&colorize_text(&bar, color).to_string());

            output.push_str(&format!(" {}{}s\n", duration, self.unit_label));
        }

        output
    }
}

/// Mini Dashboard - Composite widget with multiple metrics
pub struct MiniDashboard {
    pub title: String,
    pub metrics: Vec<(String, String, Option<f64>)>, // (label, value, change%)
    pub sparklines: Vec<(String, Vec<f64>)>,
}

impl MiniDashboard {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            metrics: Vec::new(),
            sparklines: Vec::new(),
        }
    }

    pub fn add_metric(&mut self, label: &str, value: &str, change: Option<f64>) {
        self.metrics.push((label.to_string(), value.to_string(), change));
    }

    pub fn add_sparkline(&mut self, label: &str, data: Vec<f64>) {
        self.sparklines.push((label.to_string(), data));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        // Title bar
        let width = 50;
        output.push_str(&format!("  ╭{}╮\n", "─".repeat(width)));
        let title_padding = (width - self.title.len()) / 2;
        output.push_str(&format!(
            "  │{:>width$}{}{}│\n",
            "",
            self.title.bold(),
            " ".repeat(width - title_padding - self.title.len()),
            width = title_padding
        ));
        output.push_str(&format!("  ├{}┤\n", "─".repeat(width)));

        // Metrics in a grid
        let metrics_per_row = 2;
        for chunk in self.metrics.chunks(metrics_per_row) {
            output.push_str("  │ ");
            for (i, (label, value, change)) in chunk.iter().enumerate() {
                let change_str = match change {
                    Some(c) if *c > 0.0 => format!(" {}", format!("↑{:.0}%", c).green()),
                    Some(c) if *c < 0.0 => format!(" {}", format!("↓{:.0}%", c.abs()).red()),
                    _ => String::new(),
                };

                let cell = format!("{}: {}{}", label, value.cyan(), change_str);
                let cell_width = width / metrics_per_row - 1;

                if i > 0 {
                    output.push_str("│ ");
                }
                output.push_str(&format!("{:width$}", cell, width = cell_width));
            }
            output.push_str("│\n");
        }

        // Sparklines
        if !self.sparklines.is_empty() {
            output.push_str(&format!("  ├{}┤\n", "─".repeat(width)));
            for (label, data) in &self.sparklines {
                let spark = Sparkline::new(data).with_width(width - label.len() - 4);
                output.push_str(&format!(
                    "  │ {}: {}│\n",
                    label,
                    spark.render_colored("cyan")
                ));
            }
        }

        output.push_str(&format!("  ╰{}╯\n", "─".repeat(width)));

        output
    }
}

/// ASCII Banner - Large text display
pub struct AsciiBanner {
    pub text: String,
    pub style: BannerStyle,
}

#[derive(Clone, Copy)]
pub enum BannerStyle {
    Block,
    Slim,
    Shadow,
}

impl AsciiBanner {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_uppercase(),
            style: BannerStyle::Block,
        }
    }

    pub fn with_style(mut self, style: BannerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn render(&self) -> String {
        let mut output = String::new();

        // Simple block letters for common characters
        let chars: HashMap<char, [&str; 5]> = [
            ('A', ["  █  ", " █ █ ", "█████", "█   █", "█   █"]),
            ('B', ["████ ", "█   █", "████ ", "█   █", "████ "]),
            ('C', [" ████", "█    ", "█    ", "█    ", " ████"]),
            ('D', ["████ ", "█   █", "█   █", "█   █", "████ "]),
            ('E', ["█████", "█    ", "████ ", "█    ", "█████"]),
            ('F', ["█████", "█    ", "████ ", "█    ", "█    "]),
            ('G', [" ████", "█    ", "█  ██", "█   █", " ████"]),
            ('H', ["█   █", "█   █", "█████", "█   █", "█   █"]),
            ('I', ["█████", "  █  ", "  █  ", "  █  ", "█████"]),
            ('K', ["█   █", "█  █ ", "███  ", "█  █ ", "█   █"]),
            ('L', ["█    ", "█    ", "█    ", "█    ", "█████"]),
            ('M', ["█   █", "██ ██", "█ █ █", "█   █", "█   █"]),
            ('N', ["█   █", "██  █", "█ █ █", "█  ██", "█   █"]),
            ('O', [" ███ ", "█   █", "█   █", "█   █", " ███ "]),
            ('P', ["████ ", "█   █", "████ ", "█    ", "█    "]),
            ('R', ["████ ", "█   █", "████ ", "█  █ ", "█   █"]),
            ('S', [" ████", "█    ", " ███ ", "    █", "████ "]),
            ('T', ["█████", "  █  ", "  █  ", "  █  ", "  █  "]),
            ('U', ["█   █", "█   █", "█   █", "█   █", " ███ "]),
            ('V', ["█   █", "█   █", "█   █", " █ █ ", "  █  "]),
            ('W', ["█   █", "█   █", "█ █ █", "██ ██", "█   █"]),
            ('X', ["█   █", " █ █ ", "  █  ", " █ █ ", "█   █"]),
            ('Y', ["█   █", " █ █ ", "  █  ", "  █  ", "  █  "]),
            ('Z', ["█████", "   █ ", "  █  ", " █   ", "█████"]),
            ('0', [" ███ ", "█  ██", "█ █ █", "██  █", " ███ "]),
            ('1', ["  █  ", " ██  ", "  █  ", "  █  ", " ███ "]),
            ('2', [" ███ ", "█   █", "  ██ ", " █   ", "█████"]),
            ('3', ["████ ", "    █", " ███ ", "    █", "████ "]),
            ('4', ["█   █", "█   █", "█████", "    █", "    █"]),
            ('5', ["█████", "█    ", "████ ", "    █", "████ "]),
            ('!', ["  █  ", "  █  ", "  █  ", "     ", "  █  "]),
            (' ', ["     ", "     ", "     ", "     ", "     "]),
        ].into_iter().collect();

        for row in 0..5 {
            output.push_str("  ");
            for ch in self.text.chars() {
                if let Some(pattern) = chars.get(&ch) {
                    output.push_str(pattern[row]);
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        output
    }
}

/// Metric Card - Single metric with big number display
pub struct MetricCard {
    pub label: String,
    pub value: String,
    pub subtitle: Option<String>,
    pub trend: Option<f64>,
    pub color: String,
}

impl MetricCard {
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            subtitle: None,
            trend: None,
            color: "cyan".to_string(),
        }
    }

    pub fn with_subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    pub fn with_trend(mut self, trend: f64) -> Self {
        self.trend = Some(trend);
        self
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = color.to_string();
        self
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        let width = 24;

        output.push_str(&format!("  ┌{}┐\n", "─".repeat(width)));
        output.push_str(&format!("  │ {:width$}│\n", self.label.dimmed(), width = width - 1));

        let value_str = colorize_text(&self.value, &self.color).bold().to_string();
        let trend_str = self.trend.map(|t| {
            if t > 0.0 {
                format!(" {}", format!("↑{:.0}%", t).green())
            } else if t < 0.0 {
                format!(" {}", format!("↓{:.0}%", t.abs()).red())
            } else {
                String::new()
            }
        }).unwrap_or_default();

        output.push_str(&format!("  │ {}{:>width$}│\n", value_str, trend_str, width = width - self.value.len() - trend_str.len() - 1));

        if let Some(ref sub) = self.subtitle {
            output.push_str(&format!("  │ {:width$}│\n", sub.dimmed(), width = width - 1));
        }

        output.push_str(&format!("  └{}┘\n", "─".repeat(width)));

        output
    }
}

// ═══════════════════════════════════════════════════════════════
// CREATIVE INSIGHT VISUALIZATIONS
// ═══════════════════════════════════════════════════════════════

/// MoodRing - Circular emotional state visualization based on coding patterns
/// Shows frustration vs flow state based on retry counts, error frequency, etc.
pub struct MoodRing {
    pub title: String,
    pub frustration_score: f64,  // 0-100
    pub flow_score: f64,         // 0-100
    pub energy_score: f64,       // 0-100
    pub focus_score: f64,        // 0-100
}

impl MoodRing {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            frustration_score: 0.0,
            flow_score: 0.0,
            energy_score: 0.0,
            focus_score: 0.0,
        }
    }

    pub fn set_scores(&mut self, frustration: f64, flow: f64, energy: f64, focus: f64) {
        self.frustration_score = frustration.clamp(0.0, 100.0);
        self.flow_score = flow.clamp(0.0, 100.0);
        self.energy_score = energy.clamp(0.0, 100.0);
        self.focus_score = focus.clamp(0.0, 100.0);
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        // Calculate dominant mood
        let moods = [
            (self.flow_score, "Flow", "green", "In the zone"),
            (self.focus_score, "Focus", "cyan", "Deep concentration"),
            (self.energy_score, "Energy", "yellow", "High activity"),
            (self.frustration_score, "Frustration", "red", "Struggling"),
        ];

        let dominant = moods.iter().max_by(|a, b| a.0.partial_cmp(&b.0).unwrap()).unwrap();

        // ASCII mood ring
        let ring_color = match dominant.2 {
            "green" => "🟢",
            "cyan" => "🔵",
            "yellow" => "🟡",
            "red" => "🔴",
            _ => "⚪",
        };

        // Draw concentric ring visualization
        output.push_str("         ╭─────────────╮\n");
        output.push_str("       ╭─┘             └─╮\n");
        output.push_str(&format!("      │    {}  {}      │\n", ring_color, ring_color));
        output.push_str(&format!("     │                   │\n"));
        output.push_str(&format!("     │   {:^13}   │\n", dominant.1.bold()));
        output.push_str(&format!("     │   {:^13}   │\n", format!("{:.0}%", dominant.0)));
        output.push_str(&format!("     │                   │\n"));
        output.push_str(&format!("      │    {}  {}      │\n", ring_color, ring_color));
        output.push_str("       ╰─╮             ╭─╯\n");
        output.push_str("         ╰─────────────╯\n\n");

        // Mood breakdown bars
        output.push_str(&format!("  {} ", "Flow".green()));
        let flow_bar = "█".repeat((self.flow_score / 5.0) as usize);
        let flow_empty = "░".repeat(20 - (self.flow_score / 5.0) as usize);
        output.push_str(&format!("{}{} {:.0}%\n", flow_bar.green(), flow_empty, self.flow_score));

        output.push_str(&format!("  {} ", "Focus".cyan()));
        let focus_bar = "█".repeat((self.focus_score / 5.0) as usize);
        let focus_empty = "░".repeat(20 - (self.focus_score / 5.0) as usize);
        output.push_str(&format!("{}{} {:.0}%\n", focus_bar.cyan(), focus_empty, self.focus_score));

        output.push_str(&format!("  {} ", "Energy".yellow()));
        let energy_bar = "█".repeat((self.energy_score / 5.0) as usize);
        let energy_empty = "░".repeat(20 - (self.energy_score / 5.0) as usize);
        output.push_str(&format!("{}{} {:.0}%\n", energy_bar.yellow(), energy_empty, self.energy_score));

        output.push_str(&format!("  {} ", "Stress".red()));
        let frust_bar = "█".repeat((self.frustration_score / 5.0) as usize);
        let frust_empty = "░".repeat(20 - (self.frustration_score / 5.0) as usize);
        output.push_str(&format!("{}{} {:.0}%\n", frust_bar.red(), frust_empty, self.frustration_score));

        output.push_str(&format!("\n  💭 {}\n", dominant.3));

        output
    }
}

/// CodePulse - ECG/heartbeat style visualization of coding activity rhythm
/// Shows bursts of activity like a heartbeat monitor
pub struct CodePulse {
    pub title: String,
    pub data: Vec<f64>,  // Activity intensity over time
    pub heart_rate: u32, // Calculated "coding heart rate"
}

impl CodePulse {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            data: Vec::new(),
            heart_rate: 0,
        }
    }

    pub fn set_data(&mut self, data: Vec<f64>) {
        // Calculate heart rate from peaks
        let peaks = data.windows(3)
            .filter(|w| w[1] > w[0] && w[1] > w[2])
            .count();
        self.heart_rate = (peaks as f64 * 10.0) as u32;
        self.data = data;
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.data.is_empty() {
            output.push_str("  No pulse data\n");
            return output;
        }

        let max_val = self.data.iter().cloned().fold(0.0_f64, f64::max).max(1.0);
        let height = 6;
        let width = self.data.len().min(60);

        // Resample data if needed
        let data: Vec<f64> = if self.data.len() > width {
            (0..width)
                .map(|i| {
                    let idx = i * self.data.len() / width;
                    self.data[idx]
                })
                .collect()
        } else {
            self.data.clone()
        };

        // ECG-style rendering with proper heartbeat shape
        for row in 0..height {
            let threshold = (height - row) as f64 / height as f64 * max_val;
            output.push_str("  │");
            for (_i, &val) in data.iter().enumerate() {
                let normalized = val / max_val;
                let row_pos = (height - row) as f64 / height as f64;

                // Create ECG spike pattern
                let char = if (normalized - row_pos).abs() < 0.1 {
                    if normalized > 0.7 { "╱" } else if normalized > 0.3 { "─" } else { "╲" }
                } else if val >= threshold {
                    if row == height - 1 { "▄" } else { " " }
                } else {
                    " "
                };

                let colored = if normalized > 0.8 {
                    char.red().to_string()
                } else if normalized > 0.5 {
                    char.yellow().to_string()
                } else {
                    char.green().to_string()
                };
                output.push_str(&colored);
            }
            output.push_str("│\n");
        }

        // Baseline
        output.push_str("  └");
        output.push_str(&"─".repeat(data.len()));
        output.push_str("┘\n");

        // Heart rate display
        let hr_color = if self.heart_rate > 100 {
            format!("{} BPM", self.heart_rate).red()
        } else if self.heart_rate > 60 {
            format!("{} BPM", self.heart_rate).yellow()
        } else {
            format!("{} BPM", self.heart_rate).green()
        };
        output.push_str(&format!("\n  💓 Coding Pulse: {}\n", hr_color));

        let status = if self.heart_rate > 100 {
            "⚡ High intensity coding session!"
        } else if self.heart_rate > 60 {
            "🏃 Active development pace"
        } else {
            "🧘 Calm, focused coding"
        };
        output.push_str(&format!("  {}\n", status));

        output
    }
}

/// TreeMap - Hierarchical rectangles showing proportions
/// Great for showing token usage breakdown by project/file
pub struct TreeMap {
    pub title: String,
    pub items: Vec<(String, f64, String)>, // (name, value, color)
}

impl TreeMap {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &str, value: f64, color: &str) {
        self.items.push((name.to_string(), value, color.to_string()));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.items.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let total: f64 = self.items.iter().map(|(_, v, _)| v).sum();
        let width = 50;
        let height = 12;
        let total_cells = width * height;

        // Sort by value descending
        let mut sorted: Vec<_> = self.items.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Create grid
        let mut grid = vec![vec![(' ', "white".to_string()); width]; height];
        let mut current_x = 0;
        let mut current_y = 0;
        let mut remaining_width = width;
        let mut remaining_height = height;
        let mut horizontal = true;

        for (name, value, color) in &sorted {
            let cells = ((value / total) * total_cells as f64) as usize;
            if cells == 0 { continue; }

            let (rect_w, rect_h) = if horizontal {
                let w = remaining_width;
                let h = (cells / w).max(1).min(remaining_height);
                (w, h)
            } else {
                let h = remaining_height;
                let w = (cells / h).max(1).min(remaining_width);
                (w, h)
            };

            // Fill rectangle with first char of name
            let fill_char = name.chars().next().unwrap_or('█');
            for y in current_y..(current_y + rect_h).min(height) {
                for x in current_x..(current_x + rect_w).min(width) {
                    grid[y][x] = (fill_char, color.clone());
                }
            }

            // Update position
            if horizontal {
                current_y += rect_h;
                remaining_height = remaining_height.saturating_sub(rect_h);
            } else {
                current_x += rect_w;
                remaining_width = remaining_width.saturating_sub(rect_w);
            }
            horizontal = !horizontal;
        }

        // Render grid with box
        output.push_str(&format!("  ┌{}┐\n", "─".repeat(width)));
        for row in &grid {
            output.push_str("  │");
            for (ch, color) in row {
                output.push_str(&colorize_text(&ch.to_string(), color).to_string());
            }
            output.push_str("│\n");
        }
        output.push_str(&format!("  └{}┘\n\n", "─".repeat(width)));

        // Legend
        for (name, value, color) in &sorted {
            let pct = value / total * 100.0;
            let marker = colorize_text("██", color);
            output.push_str(&format!("  {} {} ({:.1}%) - {}\n", marker, name, pct, format_number(*value)));
        }

        output
    }
}

/// SankeyFlow - ASCII flow diagram showing transitions between states
/// Shows flow of tokens/activity between tools, projects, or stages
pub struct SankeyFlow {
    pub title: String,
    pub flows: Vec<(String, String, f64)>, // (from, to, amount)
}

impl SankeyFlow {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            flows: Vec::new(),
        }
    }

    pub fn add_flow(&mut self, from: &str, to: &str, amount: f64) {
        self.flows.push((from.to_string(), to.to_string(), amount));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.flows.is_empty() {
            output.push_str("  No flows\n");
            return output;
        }

        // Get unique sources and targets
        let mut sources: Vec<&str> = self.flows.iter().map(|(s, _, _)| s.as_str()).collect();
        sources.sort();
        sources.dedup();

        let mut targets: Vec<&str> = self.flows.iter().map(|(_, t, _)| t.as_str()).collect();
        targets.sort();
        targets.dedup();

        let max_flow: f64 = self.flows.iter().map(|(_, _, a)| *a).fold(0.0, f64::max);
        let max_name = sources.iter().chain(targets.iter()).map(|s| s.len()).max().unwrap_or(10);

        // Draw flows
        for source in &sources {
            let source_flows: Vec<_> = self.flows.iter()
                .filter(|(s, _, _)| s == *source)
                .collect();

            let total: f64 = source_flows.iter().map(|(_, _, a)| *a).sum();
            let bar_width = ((total / max_flow) * 20.0) as usize;

            output.push_str(&format!("  {:>width$} ", source, width = max_name));
            output.push_str(&"█".repeat(bar_width).cyan().to_string());
            output.push_str(" ─");

            for (i, (_, target, amount)) in source_flows.iter().enumerate() {
                let flow_width = ((*amount / max_flow) * 15.0) as usize;
                let flow_char = if i == 0 { "┬" } else { "├" };
                output.push_str(&format!("{}{}─▶ {} ({})\n",
                    flow_char,
                    "─".repeat(flow_width),
                    target.green(),
                    format_number(*amount)
                ));
                if i < source_flows.len() - 1 {
                    output.push_str(&format!("{:>width$}   │", "", width = max_name + bar_width));
                }
            }
        }

        output
    }
}

/// AsciiWordCloud - ASCII art word cloud of common terms
/// Sizes words based on frequency
pub struct AsciiWordCloud {
    pub title: String,
    pub words: Vec<(String, u32)>, // (word, frequency)
}

impl AsciiWordCloud {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            words: Vec::new(),
        }
    }

    pub fn add_word(&mut self, word: &str, frequency: u32) {
        self.words.push((word.to_string(), frequency));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.words.is_empty() {
            output.push_str("  No words\n");
            return output;
        }

        // Sort by frequency
        let mut sorted = self.words.clone();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        let max_freq = sorted[0].1 as f64;
        let colors = ["red", "yellow", "green", "cyan", "blue", "magenta"];
        let width = 60;

        // Create word cloud layout
        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::from("  ");

        for (i, (word, freq)) in sorted.iter().take(30).enumerate() {
            let scale = (*freq as f64 / max_freq).sqrt();
            let styled_word = if scale > 0.8 {
                colorize_text(&word.to_uppercase(), colors[i % colors.len()]).bold().to_string()
            } else if scale > 0.5 {
                colorize_text(word, colors[i % colors.len()]).to_string()
            } else {
                word.dimmed().to_string()
            };

            let word_len = word.len() + 1;
            if current_line.len() + word_len > width {
                lines.push(current_line);
                current_line = String::from("  ");
            }
            current_line.push_str(&styled_word);
            current_line.push(' ');
        }
        if current_line.len() > 2 {
            lines.push(current_line);
        }

        // Center the cloud
        for line in &lines {
            output.push_str(line);
            output.push('\n');
        }

        output
    }
}

/// BubbleMatrix - Grid of bubbles with varying sizes
/// Shows multi-dimensional data (row, col, size, color)
pub struct BubbleMatrix {
    pub title: String,
    pub row_labels: Vec<String>,
    pub col_labels: Vec<String>,
    pub values: Vec<Vec<f64>>, // [row][col]
}

impl BubbleMatrix {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            row_labels: Vec::new(),
            col_labels: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn set_labels(&mut self, rows: Vec<&str>, cols: Vec<&str>) {
        self.row_labels = rows.iter().map(|s| s.to_string()).collect();
        self.col_labels = cols.iter().map(|s| s.to_string()).collect();
        self.values = vec![vec![0.0; cols.len()]; rows.len()];
    }

    pub fn set_value(&mut self, row: usize, col: usize, value: f64) {
        if row < self.values.len() && col < self.values[row].len() {
            self.values[row][col] = value;
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.values.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let max_val = self.values.iter()
            .flat_map(|row| row.iter())
            .cloned()
            .fold(0.0_f64, f64::max)
            .max(1.0);

        let max_row_label = self.row_labels.iter().map(|s| s.len()).max().unwrap_or(5);
        let bubbles = ["·", "∘", "○", "◎", "●", "◉"];

        // Header
        output.push_str(&format!("{:>width$}  ", "", width = max_row_label));
        for col in &self.col_labels {
            output.push_str(&format!("{:^5}", &col[..col.len().min(4)]));
        }
        output.push('\n');

        // Rows with bubbles
        for (row_idx, row_label) in self.row_labels.iter().enumerate() {
            output.push_str(&format!("{:>width$}  ", row_label, width = max_row_label));
            for col_idx in 0..self.col_labels.len() {
                let val = self.values.get(row_idx).and_then(|r| r.get(col_idx)).unwrap_or(&0.0);
                let normalized = val / max_val;
                let bubble_idx = ((normalized * 5.0) as usize).min(5);
                let bubble = bubbles[bubble_idx];

                let colored = if normalized > 0.8 {
                    bubble.red()
                } else if normalized > 0.6 {
                    bubble.yellow()
                } else if normalized > 0.4 {
                    bubble.green()
                } else if normalized > 0.2 {
                    bubble.cyan()
                } else {
                    bubble.dimmed()
                };
                output.push_str(&format!("  {}  ", colored));
            }
            output.push('\n');
        }

        // Legend
        output.push_str(&format!("\n  Size: {} min  {} max\n", "·".dimmed(), "◉".red()));

        output
    }
}

/// PolarArea - Rose/coxcomb chart for cyclical data
/// Great for showing activity by hour of day or day of week
pub struct PolarArea {
    pub title: String,
    pub segments: Vec<(String, f64, String)>, // (label, value, color)
}

impl PolarArea {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            segments: Vec::new(),
        }
    }

    pub fn add_segment(&mut self, label: &str, value: f64, color: &str) {
        self.segments.push((label.to_string(), value, color.to_string()));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.segments.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let max_val = self.segments.iter().map(|(_, v, _)| *v).fold(0.0_f64, f64::max).max(1.0);
        let radius = 8;

        // ASCII polar plot with concentric circles
        let size = radius * 2 + 3;
        let center = radius + 1;
        let mut grid = vec![vec![' '; size]; size];

        // Draw concentric circles (guidelines)
        for r in [radius / 3, radius * 2 / 3, radius] {
            for angle in 0..360 {
                let rad = (angle as f64).to_radians();
                let x = (center as f64 + rad.cos() * r as f64) as usize;
                let y = (center as f64 + rad.sin() * r as f64 * 0.5) as usize;
                if x < size && y < size && grid[y][x] == ' ' {
                    grid[y][x] = '·';
                }
            }
        }

        // Draw segments as radial bars
        let segment_angle = 360.0 / self.segments.len() as f64;
        for (i, (_, value, _)) in self.segments.iter().enumerate() {
            let angle = (i as f64 * segment_angle + 90.0).to_radians();
            let bar_len = ((value / max_val) * radius as f64) as usize;

            for r in 1..=bar_len {
                let x = (center as f64 + angle.cos() * r as f64) as usize;
                let y = (center as f64 - angle.sin() * r as f64 * 0.5) as usize;
                if x < size && y < size {
                    grid[y][x] = '█';
                }
            }
        }

        // Center marker
        grid[center][center] = '◉';

        // Render grid
        for row in &grid {
            output.push_str("  ");
            for ch in row {
                let s = if *ch == '█' {
                    ch.to_string().cyan().to_string()
                } else if *ch == '◉' {
                    ch.to_string().yellow().to_string()
                } else {
                    ch.to_string().dimmed().to_string()
                };
                output.push_str(&s);
            }
            output.push('\n');
        }

        // Legend
        output.push('\n');
        for (i, (label, value, color)) in self.segments.iter().enumerate() {
            let marker = colorize_text("●", color);
            output.push_str(&format!("  {} {} ({:.0})\n", marker, label, value));
            if i >= 7 {
                output.push_str(&format!("  ... and {} more\n", self.segments.len() - 8));
                break;
            }
        }

        output
    }
}

/// TimelineStory - Narrative timeline with milestones and events
/// Shows a story of your coding journey with key moments
pub struct TimelineStory {
    pub title: String,
    pub events: Vec<(String, String, String, String)>, // (date, title, description, type)
}

impl TimelineStory {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, date: &str, title: &str, description: &str, event_type: &str) {
        self.events.push((
            date.to_string(),
            title.to_string(),
            description.to_string(),
            event_type.to_string(),
        ));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.events.is_empty() {
            output.push_str("  No events\n");
            return output;
        }

        for (i, (date, title, description, event_type)) in self.events.iter().enumerate() {
            let icon = match event_type.as_str() {
                "milestone" => "🏆",
                "achievement" => "⭐",
                "bug" => "🐛",
                "feature" => "✨",
                "refactor" => "🔧",
                "learning" => "📚",
                "breakthrough" => "💡",
                _ => "📌",
            };

            let connector = if i == self.events.len() - 1 { "└" } else { "├" };
            let line = if i == self.events.len() - 1 { " " } else { "│" };

            output.push_str(&format!("  {} {} {}\n", date.dimmed(), icon, title.bold()));
            output.push_str(&format!("  {}──┤\n", connector));

            // Word wrap description
            let words: Vec<&str> = description.split_whitespace().collect();
            let mut current_line = String::new();
            for word in words {
                if current_line.len() + word.len() > 45 {
                    output.push_str(&format!("  {}   {}\n", line, current_line));
                    current_line = word.to_string();
                } else {
                    if !current_line.is_empty() { current_line.push(' '); }
                    current_line.push_str(word);
                }
            }
            if !current_line.is_empty() {
                output.push_str(&format!("  {}   {}\n", line, current_line));
            }
            output.push_str(&format!("  {}\n", line));
        }

        output
    }
}

/// HexGrid - Hexagonal grid heatmap (honeycomb pattern)
/// Unique way to visualize 2D density data
pub struct HexGrid {
    pub title: String,
    pub data: Vec<Vec<f64>>,
    pub width: usize,
    pub height: usize,
}

impl HexGrid {
    pub fn new(title: &str, width: usize, height: usize) -> Self {
        Self {
            title: title.to_string(),
            data: vec![vec![0.0; width]; height],
            width,
            height,
        }
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: f64) {
        if y < self.height && x < self.width {
            self.data[y][x] = value;
        }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        let max_val = self.data.iter()
            .flat_map(|row| row.iter())
            .cloned()
            .fold(0.0_f64, f64::max)
            .max(1.0);

        // Hex characters for different intensities
        let hex_chars = ["⬡", "⬢"];

        for (y, row) in self.data.iter().enumerate() {
            // Offset odd rows for honeycomb effect
            if y % 2 == 1 {
                output.push_str(" ");
            }
            output.push_str("  ");

            for &val in row {
                let intensity = val / max_val;
                let hex = if intensity > 0.1 { hex_chars[1] } else { hex_chars[0] };

                let colored = if intensity > 0.8 {
                    hex.red()
                } else if intensity > 0.6 {
                    hex.yellow()
                } else if intensity > 0.4 {
                    hex.green()
                } else if intensity > 0.2 {
                    hex.cyan()
                } else if intensity > 0.1 {
                    hex.blue()
                } else {
                    hex.dimmed()
                };
                output.push_str(&format!("{} ", colored));
            }
            output.push('\n');
        }

        output.push_str(&format!("\n  Intensity: {} low  {} high\n", "⬡".dimmed(), "⬢".red()));

        output
    }
}

/// FlowState - Visualization of focus/flow vs interruption patterns
/// Shows when you were "in the zone" vs getting interrupted
pub struct FlowState {
    pub title: String,
    pub periods: Vec<(String, f64, bool)>, // (time_label, duration_mins, was_flow)
}

impl FlowState {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            periods: Vec::new(),
        }
    }

    pub fn add_period(&mut self, time_label: &str, duration_mins: f64, was_flow: bool) {
        self.periods.push((time_label.to_string(), duration_mins, was_flow));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.periods.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let total_time: f64 = self.periods.iter().map(|(_, d, _)| d).sum();
        let flow_time: f64 = self.periods.iter()
            .filter(|(_, _, f)| *f)
            .map(|(_, d, _)| d)
            .sum();
        let flow_pct = flow_time / total_time * 100.0;

        // Flow vs Fragmented bar
        output.push_str("  Flow State Timeline\n  ");
        let bar_width = 50;
        for (_, duration, was_flow) in &self.periods {
            let width = ((duration / total_time) * bar_width as f64) as usize;
            let char = if *was_flow { "█" } else { "░" };
            let colored = if *was_flow {
                char.green().to_string()
            } else {
                char.red().to_string()
            };
            output.push_str(&colored.repeat(width.max(1)));
        }
        output.push_str("\n\n");

        // Summary stats
        let flow_score = if flow_pct > 70.0 {
            "Excellent! Deep work champion".green()
        } else if flow_pct > 50.0 {
            "Good flow state achieved".cyan()
        } else if flow_pct > 30.0 {
            "Moderate - try reducing interruptions".yellow()
        } else {
            "Fragmented - consider time blocking".red()
        };

        output.push_str(&format!("  {} Flow time:    {:.0} mins ({:.1}%)\n", "█".green(), flow_time, flow_pct));
        output.push_str(&format!("  {} Interrupted:  {:.0} mins ({:.1}%)\n", "░".red(), total_time - flow_time, 100.0 - flow_pct));
        output.push_str(&format!("\n  🎯 {}\n", flow_score));

        // Flow periods breakdown
        output.push_str("\n  Flow Sessions:\n");
        let flow_sessions: Vec<_> = self.periods.iter()
            .filter(|(_, _, f)| *f)
            .collect();

        let mut longest = 0.0_f64;
        for (time, duration, _) in flow_sessions.iter().take(5) {
            output.push_str(&format!("    {} - {:.0} mins of focus\n", time.cyan(), duration));
            longest = longest.max(*duration);
        }

        if longest > 0.0 {
            output.push_str(&format!("\n  🏆 Longest flow: {:.0} mins\n", longest));
        }

        output
    }
}

// ═══════════════════════════════════════════════════════════════
// GAMIFIED & ADVANCED VISUALIZATIONS
// ═══════════════════════════════════════════════════════════════

/// AchievementBadges - Unlockable achievements display
/// Shows earned badges with progress indicators
pub struct AchievementBadges {
    pub title: String,
    pub badges: Vec<(String, String, bool, f64)>, // (name, icon, unlocked, progress)
}

impl AchievementBadges {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            badges: Vec::new(),
        }
    }

    pub fn add_badge(&mut self, name: &str, icon: &str, unlocked: bool, progress: f64) {
        self.badges.push((name.to_string(), icon.to_string(), unlocked, progress.clamp(0.0, 100.0)));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        let unlocked_count = self.badges.iter().filter(|(_, _, u, _)| *u).count();
        output.push_str(&format!("  🏆 {} / {} Unlocked\n\n", unlocked_count, self.badges.len()));

        for (name, icon, unlocked, progress) in &self.badges {
            if *unlocked {
                output.push_str(&format!("  {} {} {}\n", icon, name.green().bold(), "✓".green()));
            } else {
                let bar_width = 10;
                let filled = (progress / 100.0 * bar_width as f64) as usize;
                let bar = format!("{}{}",
                    "█".repeat(filled),
                    "░".repeat(bar_width - filled)
                );
                output.push_str(&format!("  {} {} {} {:.0}%\n",
                    icon.dimmed(),
                    name.dimmed(),
                    bar.dimmed(),
                    progress
                ));
            }
        }

        output
    }
}

/// SkillTree - RPG-style skill/technology tree
/// Shows progression through different skill branches
pub struct SkillTree {
    pub title: String,
    pub branches: Vec<(String, Vec<(String, u8)>)>, // (branch_name, [(skill, level 0-5)])
}

impl SkillTree {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            branches: Vec::new(),
        }
    }

    pub fn add_branch(&mut self, name: &str, skills: Vec<(&str, u8)>) {
        self.branches.push((
            name.to_string(),
            skills.iter().map(|(s, l)| (s.to_string(), *l)).collect()
        ));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        let level_chars = ["○", "◔", "◑", "◕", "●", "★"];

        for (branch_name, skills) in &self.branches {
            output.push_str(&format!("  ┌─ {} ─────────────────────\n", branch_name.cyan().bold()));

            for (i, (skill, level)) in skills.iter().enumerate() {
                let connector = if i == skills.len() - 1 { "└" } else { "├" };
                let level_idx = (*level as usize).min(5);
                let level_char = level_chars[level_idx];

                let colored_char = match level_idx {
                    5 => level_char.yellow().bold().to_string(),
                    4 => level_char.green().to_string(),
                    3 => level_char.cyan().to_string(),
                    2 => level_char.blue().to_string(),
                    1 => level_char.dimmed().to_string(),
                    _ => level_char.dimmed().to_string(),
                };

                output.push_str(&format!("  {}── {} {} (Lv.{})\n",
                    connector,
                    colored_char,
                    skill,
                    level
                ));
            }
            output.push('\n');
        }

        output.push_str(&format!("  Legend: {} None  {} Beginner  {} Intermediate  {} Advanced  {} Expert  {} Master\n",
            level_chars[0].dimmed(),
            level_chars[1].dimmed(),
            level_chars[2].blue(),
            level_chars[3].cyan(),
            level_chars[4].green(),
            level_chars[5].yellow()
        ));

        output
    }
}

/// CommitGraph - Git-style commit activity visualization
/// Shows commits over time with branch-like structure
pub struct CommitGraph {
    pub title: String,
    pub commits: Vec<(String, String, u32)>, // (date, message, additions)
}

impl CommitGraph {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            commits: Vec::new(),
        }
    }

    pub fn add_commit(&mut self, date: &str, message: &str, additions: u32) {
        self.commits.push((date.to_string(), message.to_string(), additions));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.commits.is_empty() {
            output.push_str("  No commits\n");
            return output;
        }

        let max_adds = self.commits.iter().map(|(_, _, a)| *a).max().unwrap_or(1);

        for (i, (date, message, additions)) in self.commits.iter().enumerate() {
            let is_last = i == self.commits.len() - 1;

            // Commit node
            let node_size = ((*additions as f64 / max_adds as f64) * 3.0) as usize;
            let node = match node_size {
                0 => "○",
                1 => "◎",
                2 => "●",
                _ => "◉",
            };

            let colored_node = if *additions > max_adds / 2 {
                node.green().bold().to_string()
            } else {
                node.cyan().to_string()
            };

            // Branch line
            let branch = if is_last { "└" } else { "├" };
            let line = if is_last { " " } else { "│" };

            output.push_str(&format!("  {} {} {} +{}\n",
                branch,
                colored_node,
                date.dimmed(),
                additions.to_string().green()
            ));

            // Truncate message
            let msg = if message.len() > 45 {
                format!("{}...", &message[..42])
            } else {
                message.clone()
            };
            output.push_str(&format!("  {}   {}\n", line, msg));
            if !is_last {
                output.push_str(&format!("  {}\n", line));
            }
        }

        output
    }
}

/// ProductivityScore - Gamified productivity dashboard
/// Shows score breakdown with level and XP
pub struct ProductivityScore {
    pub title: String,
    pub total_score: u32,
    pub level: u32,
    pub xp_current: u32,
    pub xp_next_level: u32,
    pub multipliers: Vec<(String, f64)>,
}

impl ProductivityScore {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            total_score: 0,
            level: 1,
            xp_current: 0,
            xp_next_level: 1000,
            multipliers: Vec::new(),
        }
    }

    pub fn set_score(&mut self, score: u32, level: u32, xp_current: u32, xp_next: u32) {
        self.total_score = score;
        self.level = level;
        self.xp_current = xp_current;
        self.xp_next_level = xp_next;
    }

    pub fn add_multiplier(&mut self, name: &str, value: f64) {
        self.multipliers.push((name.to_string(), value));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        // Level display
        let level_stars = "★".repeat((self.level as usize).min(5));
        let empty_stars = "☆".repeat(5 - (self.level as usize).min(5));
        output.push_str(&format!("  Level {} {}{}\n\n",
            self.level.to_string().yellow().bold(),
            level_stars.yellow(),
            empty_stars.dimmed()
        ));

        // Score display
        output.push_str(&format!("  ┌─────────────────────────────────┐\n"));
        output.push_str(&format!("  │  PRODUCTIVITY SCORE             │\n"));
        output.push_str(&format!("  │  {:^27}  │\n", format!("{}", self.total_score).green().bold()));
        output.push_str(&format!("  └─────────────────────────────────┘\n\n"));

        // XP bar
        let xp_pct = (self.xp_current as f64 / self.xp_next_level as f64 * 100.0).min(100.0);
        let bar_width = 25;
        let filled = (xp_pct / 100.0 * bar_width as f64) as usize;
        output.push_str(&format!("  XP: {}/{}\n",
            self.xp_current.to_string().cyan(),
            self.xp_next_level
        ));
        output.push_str(&format!("  [{}{}] {:.0}%\n\n",
            "█".repeat(filled).cyan(),
            "░".repeat(bar_width - filled),
            xp_pct
        ));

        // Multipliers
        if !self.multipliers.is_empty() {
            output.push_str("  Active Multipliers:\n");
            for (name, value) in &self.multipliers {
                let colored = if *value >= 1.5 {
                    format!("x{:.1}", value).green().bold().to_string()
                } else if *value >= 1.0 {
                    format!("x{:.1}", value).cyan().to_string()
                } else {
                    format!("x{:.1}", value).red().to_string()
                };
                output.push_str(&format!("    {} {}\n", colored, name));
            }
        }

        output
    }
}

/// EnergyMeter - Battery-style energy level indicator
/// Shows current energy/focus level with depletion tracking
pub struct EnergyMeter {
    pub title: String,
    pub current_energy: f64,  // 0-100
    pub max_energy: f64,
    pub drain_rate: f64,      // per hour
    pub recharge_events: Vec<(String, f64)>,
}

impl EnergyMeter {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            current_energy: 100.0,
            max_energy: 100.0,
            drain_rate: 10.0,
            recharge_events: Vec::new(),
        }
    }

    pub fn set_energy(&mut self, current: f64, max: f64, drain: f64) {
        self.current_energy = current.clamp(0.0, max);
        self.max_energy = max;
        self.drain_rate = drain;
    }

    pub fn add_recharge(&mut self, event: &str, amount: f64) {
        self.recharge_events.push((event.to_string(), amount));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        let pct = self.current_energy / self.max_energy * 100.0;

        // Battery visualization
        let battery_width = 20;
        let filled = (pct / 100.0 * battery_width as f64) as usize;

        let (fill_char, color) = if pct > 60.0 {
            ("█", "green")
        } else if pct > 30.0 {
            ("█", "yellow")
        } else if pct > 10.0 {
            ("█", "red")
        } else {
            ("▒", "red")
        };

        output.push_str("  ┌──────────────────────┬┐\n");
        output.push_str(&format!("  │{}{}│█\n",
            colorize_text(&fill_char.repeat(filled), color),
            " ".repeat(battery_width - filled)
        ));
        output.push_str("  └──────────────────────┴┘\n\n");

        // Stats
        let energy_display = if pct > 60.0 {
            format!("{:.0}%", pct).green()
        } else if pct > 30.0 {
            format!("{:.0}%", pct).yellow()
        } else {
            format!("{:.0}%", pct).red()
        };

        output.push_str(&format!("  Energy: {} ({:.0}/{:.0})\n", energy_display, self.current_energy, self.max_energy));
        output.push_str(&format!("  Drain rate: {:.1}/hour\n", self.drain_rate));

        let hours_left = if self.drain_rate > 0.0 {
            self.current_energy / self.drain_rate
        } else {
            999.0
        };
        output.push_str(&format!("  Time remaining: {:.1}h\n", hours_left));

        // Recharge events
        if !self.recharge_events.is_empty() {
            output.push_str("\n  Recharge boosts:\n");
            for (event, amount) in &self.recharge_events {
                output.push_str(&format!("    {} +{:.0}\n", event, amount));
            }
        }

        // Status
        let status = if pct > 80.0 {
            "⚡ Fully charged! Maximum productivity!"
        } else if pct > 60.0 {
            "🔋 Good energy levels"
        } else if pct > 30.0 {
            "🔋 Moderate - consider a break soon"
        } else if pct > 10.0 {
            "⚠️ Low energy - take a break!"
        } else {
            "🪫 Critical! Rest immediately!"
        };
        output.push_str(&format!("\n  {}\n", status));

        output
    }
}

/// LearningCurve - Shows skill/knowledge progression over time
/// Visualizes improvement trajectory
pub struct LearningCurve {
    pub title: String,
    pub skill_name: String,
    pub data_points: Vec<(String, f64)>, // (date, proficiency 0-100)
}

impl LearningCurve {
    pub fn new(title: &str, skill: &str) -> Self {
        Self {
            title: title.to_string(),
            skill_name: skill.to_string(),
            data_points: Vec::new(),
        }
    }

    pub fn add_point(&mut self, date: &str, proficiency: f64) {
        self.data_points.push((date.to_string(), proficiency.clamp(0.0, 100.0)));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n", self.title.bold()));
        output.push_str(&format!("  Skill: {}\n\n", self.skill_name.cyan()));

        if self.data_points.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        let height = 8;
        let width = self.data_points.len().min(40);

        // Resample if needed
        let data: Vec<f64> = if self.data_points.len() > width {
            (0..width).map(|i| {
                let idx = i * self.data_points.len() / width;
                self.data_points[idx].1
            }).collect()
        } else {
            self.data_points.iter().map(|(_, v)| *v).collect()
        };

        // Draw curve
        for row in 0..height {
            let threshold = 100.0 - (row as f64 / height as f64 * 100.0);
            let label = match row {
                0 => "100%",
                r if r == height - 1 => "  0%",
                _ => "    ",
            };
            output.push_str(&format!("  {} │", label));

            for (i, &val) in data.iter().enumerate() {
                let prev = if i > 0 { data[i - 1] } else { val };

                let char = if val >= threshold && prev < threshold {
                    "╱"
                } else if val < threshold && prev >= threshold {
                    "╲"
                } else if val >= threshold && val >= threshold - (100.0 / height as f64) {
                    "─"
                } else {
                    " "
                };

                let colored = if val > 80.0 {
                    char.green().to_string()
                } else if val > 50.0 {
                    char.cyan().to_string()
                } else if val > 25.0 {
                    char.yellow().to_string()
                } else {
                    char.dimmed().to_string()
                };
                output.push_str(&colored);
            }
            output.push_str("│\n");
        }
        output.push_str(&format!("       └{}┘\n", "─".repeat(data.len())));

        // Summary
        let first = self.data_points.first().map(|(_, v)| *v).unwrap_or(0.0);
        let last = self.data_points.last().map(|(_, v)| *v).unwrap_or(0.0);
        let improvement = last - first;

        output.push_str(&format!("\n  Start: {:.0}%  →  Current: {:.0}%\n", first, last));
        if improvement > 0.0 {
            output.push_str(&format!("  📈 Improvement: {}\n", format!("+{:.0}%", improvement).green()));
        } else if improvement < 0.0 {
            output.push_str(&format!("  📉 Change: {}\n", format!("{:.0}%", improvement).red()));
        }

        output
    }
}

/// SessionStack - Stacked sessions showing depth and duration
/// Visualizes coding session layers
pub struct SessionStack {
    pub title: String,
    pub sessions: Vec<(String, f64, String)>, // (project, hours, status)
}

impl SessionStack {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            sessions: Vec::new(),
        }
    }

    pub fn add_session(&mut self, project: &str, hours: f64, status: &str) {
        self.sessions.push((project.to_string(), hours, status.to_string()));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.sessions.is_empty() {
            output.push_str("  No sessions\n");
            return output;
        }

        let max_hours = self.sessions.iter().map(|(_, h, _)| *h).fold(0.0_f64, f64::max).max(1.0);
        let total_hours: f64 = self.sessions.iter().map(|(_, h, _)| *h).sum();

        // Draw stacked blocks
        let width = 40;
        output.push_str(&format!("  ┌{}┐\n", "─".repeat(width)));

        for (project, hours, status) in self.sessions.iter().rev() {
            let block_width = ((hours / max_hours) * (width - 4) as f64) as usize;
            let status_icon = match status.as_str() {
                "completed" => "✓".green(),
                "active" => "●".cyan(),
                "paused" => "◐".yellow(),
                _ => "○".dimmed(),
            };

            let bar = "█".repeat(block_width.max(1));
            let padding = width - 4 - block_width.max(1);

            output.push_str(&format!("  │ {}{}{} │\n",
                bar.cyan(),
                " ".repeat(padding),
                status_icon
            ));
            output.push_str(&format!("  │ {:width$} │\n",
                format!("{} ({:.1}h)", project, hours),
                width = width - 4
            ));
        }

        output.push_str(&format!("  └{}┘\n", "─".repeat(width)));
        output.push_str(&format!("\n  Total: {:.1}h across {} sessions\n", total_hours, self.sessions.len()));

        output
    }
}

/// MilestoneRoad - Road/path visualization to goals
/// Shows progress along a journey with checkpoints
pub struct MilestoneRoad {
    pub title: String,
    pub milestones: Vec<(String, bool, Option<String>)>, // (name, reached, date)
    pub current_progress: f64, // 0-100 between milestones
}

impl MilestoneRoad {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            milestones: Vec::new(),
            current_progress: 0.0,
        }
    }

    pub fn add_milestone(&mut self, name: &str, reached: bool, date: Option<&str>) {
        self.milestones.push((name.to_string(), reached, date.map(|s| s.to_string())));
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.current_progress = progress.clamp(0.0, 100.0);
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.milestones.is_empty() {
            output.push_str("  No milestones\n");
            return output;
        }

        let reached_count = self.milestones.iter().filter(|(_, r, _)| *r).count();
        output.push_str(&format!("  Progress: {} / {} milestones\n\n", reached_count, self.milestones.len()));

        // Find current position
        let current_idx = self.milestones.iter().position(|(_, r, _)| !*r).unwrap_or(self.milestones.len());

        for (i, (name, reached, date)) in self.milestones.iter().enumerate() {
            let is_current = i == current_idx;
            let is_last = i == self.milestones.len() - 1;

            // Milestone marker
            let marker = if *reached {
                "◉".green()
            } else if is_current {
                "◎".yellow()
            } else {
                "○".dimmed()
            };

            // Road segment
            let road = if is_last {
                "   ".to_string()
            } else if *reached {
                " ║ ".green().to_string()
            } else if is_current {
                format!(" {} ", "┊".yellow())
            } else {
                " ┊ ".dimmed().to_string()
            };

            let name_str = if *reached {
                name.green().to_string()
            } else if is_current {
                name.yellow().bold().to_string()
            } else {
                name.dimmed().to_string()
            };

            let date_str = date.as_ref().map(|d| format!(" ({})", d.dimmed())).unwrap_or_default();

            output.push_str(&format!("  {} {}{}\n", marker, name_str, date_str));

            if !is_last {
                output.push_str(&format!("  {}\n", road));
            }

            // Show current position indicator
            if is_current && self.current_progress > 0.0 && !is_last {
                let position = (self.current_progress / 100.0 * 3.0) as usize;
                for p in 0..3 {
                    if p == position {
                        output.push_str(&format!("  {} 🚀\n", "┊".yellow()));
                    }
                }
            }
        }

        output
    }
}

/// ComparisonRadar - Multi-axis spider/radar chart for comparisons
/// Compare multiple items across different dimensions
pub struct ComparisonRadar {
    pub title: String,
    pub axes: Vec<String>,
    pub items: Vec<(String, Vec<f64>, String)>, // (name, values, color)
}

impl ComparisonRadar {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            axes: Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn set_axes(&mut self, axes: Vec<&str>) {
        self.axes = axes.iter().map(|s| s.to_string()).collect();
    }

    pub fn add_item(&mut self, name: &str, values: Vec<f64>, color: &str) {
        self.items.push((name.to_string(), values, color.to_string()));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        if self.axes.is_empty() || self.items.is_empty() {
            output.push_str("  No data\n");
            return output;
        }

        // Render as horizontal bars per axis for ASCII compatibility
        let max_name_len = self.axes.iter().map(|s| s.len()).max().unwrap_or(10);

        for (axis_idx, axis) in self.axes.iter().enumerate() {
            output.push_str(&format!("  {:>width$} │", axis, width = max_name_len));

            for (_item_name, values, color) in &self.items {
                let val = values.get(axis_idx).unwrap_or(&0.0);
                let bar_len = (val / 100.0 * 10.0) as usize;
                let bar = colorize_text(&"█".repeat(bar_len), color);
                output.push_str(&format!(" {}", bar));
            }
            output.push('\n');
        }

        // Legend
        output.push('\n');
        for (name, _, color) in &self.items {
            output.push_str(&format!("  {} {}\n", colorize_text("██", color), name));
        }

        output
    }
}

/// StreakFlame - Animated fire-style streak display
/// Shows current streak with intensity visualization
pub struct StreakFlame {
    pub title: String,
    pub current_streak: u32,
    pub best_streak: u32,
    pub streak_history: Vec<u32>, // Last N days
}

impl StreakFlame {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            current_streak: 0,
            best_streak: 0,
            streak_history: Vec::new(),
        }
    }

    pub fn set_streak(&mut self, current: u32, best: u32) {
        self.current_streak = current;
        self.best_streak = best;
    }

    pub fn set_history(&mut self, history: Vec<u32>) {
        self.streak_history = history;
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("  {}\n\n", self.title.bold()));

        // Flame intensity based on streak
        let intensity = (self.current_streak as f64 / 30.0).min(1.0);

        // ASCII flame art
        if self.current_streak > 0 {
            let flame_height = ((intensity * 5.0) as usize).max(1);

            let flames = [
                ("      ", "      "),
                ("  )   ", "   (  "),
                (" ) (  ", "  ) ( "),
                ("( ) ) ", " ( ( )"),
                (") ( ( ", "( ) ) "),
            ];

            for i in (0..flame_height).rev() {
                let (left, right) = flames[i.min(4)];
                let color = if i > 3 {
                    "red"
                } else if i > 1 {
                    "yellow"
                } else {
                    "white"
                };
                output.push_str(&format!("  {}{}\n",
                    colorize_text(left, color),
                    colorize_text(right, color)
                ));
            }

            output.push_str(&format!("  {}🔥{}\n", " ".repeat(4), " ".repeat(4)));
        }

        // Streak count
        let streak_display = if self.current_streak >= self.best_streak && self.current_streak > 0 {
            format!("{} 🏆 NEW RECORD!", self.current_streak).yellow().bold().to_string()
        } else if self.current_streak > 20 {
            format!("{} days", self.current_streak).red().bold().to_string()
        } else if self.current_streak > 7 {
            format!("{} days", self.current_streak).yellow().bold().to_string()
        } else if self.current_streak > 0 {
            format!("{} days", self.current_streak).green().to_string()
        } else {
            "0 days".dimmed().to_string()
        };

        output.push_str(&format!("\n  Current: {}\n", streak_display));
        output.push_str(&format!("  Best:    {} days\n", self.best_streak));

        // Streak history sparkline
        if !self.streak_history.is_empty() {
            output.push_str("\n  History: ");
            for &s in self.streak_history.iter().take(14) {
                let char = if s > 0 { "█" } else { "░" };
                let colored = if s > 7 {
                    char.red().to_string()
                } else if s > 3 {
                    char.yellow().to_string()
                } else if s > 0 {
                    char.green().to_string()
                } else {
                    char.dimmed().to_string()
                };
                output.push_str(&colored);
            }
            output.push('\n');
        }

        // Motivation message
        let msg = if self.current_streak == 0 {
            "Start a new streak today!"
        } else if self.current_streak < 3 {
            "Keep going! Building momentum..."
        } else if self.current_streak < 7 {
            "Nice streak! Almost a week!"
        } else if self.current_streak < 14 {
            "Impressive! Two weeks in sight!"
        } else if self.current_streak < 30 {
            "On fire! A month is within reach!"
        } else {
            "LEGENDARY! You're unstoppable!"
        };
        output.push_str(&format!("\n  💪 {}\n", msg));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500.0), "500");
        assert_eq!(format_number(1500.0), "2k");
        assert_eq!(format_number(1_500_000.0), "1.5M");
        assert_eq!(format_number(1_500_000_000.0), "1.5B");
    }

    #[test]
    fn test_sparkline() {
        let spark = Sparkline::new(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let rendered = spark.render();
        assert_eq!(rendered.chars().count(), 5);
    }

    #[test]
    fn test_bar_chart() {
        let mut chart = BarChart::new("Test Chart");
        chart.add("Item 1", 50.0, "cyan");
        chart.add("Item 2", 30.0, "magenta");
        chart.add("Item 3", 20.0, "yellow");
        let rendered = chart.render();
        assert!(rendered.contains("Item 1"));
        assert!(rendered.contains("50.0%"));
    }
}
