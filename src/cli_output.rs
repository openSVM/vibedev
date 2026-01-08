// Agent-friendly CLI output system
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::io::{self, IsTerminal};

/// Output mode for CLI commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Human-friendly output with colors and emojis
    Human,
    /// Machine-readable JSON output
    Json,
    /// Plain text without colors (for pipes/logs)
    Plain,
}

impl OutputMode {
    /// Auto-detect output mode based on environment
    pub fn auto() -> Self {
        if !io::stdout().is_terminal() {
            // Output is piped/redirected, use plain text
            Self::Plain
        } else if std::env::var("VIBEDEV_JSON").is_ok() {
            // JSON mode requested via env var
            Self::Json
        } else {
            // Interactive terminal, use human-friendly output
            Self::Human
        }
    }
}

/// Structured progress update
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub stage: String,
    pub message: String,
    pub current: Option<usize>,
    pub total: Option<usize>,
    pub percentage: Option<f64>,
    pub status: ProgressStatus,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStatus {
    Started,
    Running,
    Completed,
    Failed,
    Warning,
}

/// Structured result for agent consumption
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub command: String,
    pub duration_ms: u64,
    pub output: serde_json::Value,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// CLI output writer with mode awareness
pub struct OutputWriter {
    mode: OutputMode,
}

#[allow(dead_code)]
impl OutputWriter {
    pub fn new(mode: OutputMode) -> Self {
        Self { mode }
    }

    pub fn auto() -> Self {
        Self::new(OutputMode::auto())
    }

    /// Print a section header
    pub fn section(&self, title: &str) {
        match self.mode {
            OutputMode::Human => {
                println!();
                println!("{}", title.cyan().bold());
                println!("{}", "═".repeat(title.len()).cyan());
            }
            OutputMode::Plain => {
                println!();
                println!("{}", title);
                println!("{}", "=".repeat(title.len()));
            }
            OutputMode::Json => {
                // In JSON mode, sections are part of structured output
            }
        }
    }

    /// Print a success message
    pub fn success(&self, message: &str) {
        match self.mode {
            OutputMode::Human => println!("  {} {}", "✓".green(), message),
            OutputMode::Plain => println!("  [OK] {}", message),
            OutputMode::Json => self.emit_progress(ProgressUpdate {
                stage: "info".to_string(),
                message: message.to_string(),
                current: None,
                total: None,
                percentage: None,
                status: ProgressStatus::Completed,
            }),
        }
    }

    /// Print an error message
    pub fn error(&self, message: &str) {
        match self.mode {
            OutputMode::Human => eprintln!("  {} {}", "✗".red(), message),
            OutputMode::Plain => eprintln!("  [ERROR] {}", message),
            OutputMode::Json => self.emit_progress(ProgressUpdate {
                stage: "error".to_string(),
                message: message.to_string(),
                current: None,
                total: None,
                percentage: None,
                status: ProgressStatus::Failed,
            }),
        }
    }

    /// Print a warning message
    pub fn warning(&self, message: &str) {
        match self.mode {
            OutputMode::Human => println!("  {} {}", "⚠".yellow(), message),
            OutputMode::Plain => println!("  [WARN] {}", message),
            OutputMode::Json => self.emit_progress(ProgressUpdate {
                stage: "warning".to_string(),
                message: message.to_string(),
                current: None,
                total: None,
                percentage: None,
                status: ProgressStatus::Warning,
            }),
        }
    }

    /// Print an info message
    pub fn info(&self, message: &str) {
        match self.mode {
            OutputMode::Human => println!("  {}", message),
            OutputMode::Plain => println!("  {}", message),
            OutputMode::Json => self.emit_progress(ProgressUpdate {
                stage: "info".to_string(),
                message: message.to_string(),
                current: None,
                total: None,
                percentage: None,
                status: ProgressStatus::Running,
            }),
        }
    }

    /// Print a metric
    pub fn metric(&self, label: &str, value: &str, unit: Option<&str>) {
        let full_value = if let Some(u) = unit {
            format!("{} {}", value, u)
        } else {
            value.to_string()
        };

        match self.mode {
            OutputMode::Human => println!("    • {}: {}", label, full_value.green()),
            OutputMode::Plain => println!("    - {}: {}", label, full_value),
            OutputMode::Json => {
                // Metrics are part of structured output
            }
        }
    }

    /// Print progress update
    pub fn progress(&self, stage: &str, current: usize, total: usize) {
        let percentage = (current as f64 / total as f64 * 100.0).min(100.0);

        match self.mode {
            OutputMode::Human => {
                print!(
                    "\r  {} [{}/{}] {:.0}%",
                    "⏳".cyan(),
                    current,
                    total,
                    percentage
                );
                if current == total {
                    println!();
                }
                io::Write::flush(&mut io::stdout()).ok();
            }
            OutputMode::Plain => {
                if current == total || current % 10 == 0 {
                    println!("  [{}] {}/{} ({:.0}%)", stage, current, total, percentage);
                }
            }
            OutputMode::Json => self.emit_progress(ProgressUpdate {
                stage: stage.to_string(),
                message: format!("Processing {}/{}", current, total),
                current: Some(current),
                total: Some(total),
                percentage: Some(percentage),
                status: if current == total {
                    ProgressStatus::Completed
                } else {
                    ProgressStatus::Running
                },
            }),
        }
    }

    /// Emit structured progress update (for JSON mode)
    fn emit_progress(&self, update: ProgressUpdate) {
        if matches!(self.mode, OutputMode::Json) {
            if let Ok(json) = serde_json::to_string(&update) {
                println!("{}", json);
            }
        }
    }

    /// Emit final structured result (for JSON mode)
    pub fn emit_result(&self, result: &CommandResult) {
        match self.mode {
            OutputMode::Json => {
                if let Ok(json) = serde_json::to_string_pretty(&result) {
                    println!("{}", json);
                }
            }
            _ => {
                // In human/plain mode, results are printed incrementally
            }
        }
    }

    /// Print a key-value table
    pub fn table(&self, rows: &[(&str, String)]) {
        match self.mode {
            OutputMode::Human => {
                let max_key_len = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
                for (key, value) in rows {
                    println!("  {:width$} │ {}", key.yellow(), value, width = max_key_len);
                }
            }
            OutputMode::Plain => {
                let max_key_len = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
                for (key, value) in rows {
                    println!("  {:width$} : {}", key, value, width = max_key_len);
                }
            }
            OutputMode::Json => {
                // Tables are part of structured output
            }
        }
    }

    /// Print horizontal bar chart
    pub fn bar_chart(&self, label: &str, value: f64, max: f64, width: usize) {
        let filled = ((value / max * width as f64) as usize).min(width);
        let bar = match self.mode {
            OutputMode::Human => {
                format!(
                    "{}{}",
                    "█".repeat(filled).cyan(),
                    "░".repeat(width - filled)
                )
            }
            OutputMode::Plain => {
                format!("{}{}", "#".repeat(filled), "-".repeat(width - filled))
            }
            OutputMode::Json => return,
        };

        println!("  {} │{} {:.1}%", label, bar, value / max * 100.0);
    }

    /// Get the output mode
    pub fn mode(&self) -> OutputMode {
        self.mode
    }

    /// Check if output is human-friendly
    pub fn is_human(&self) -> bool {
        matches!(self.mode, OutputMode::Human)
    }

    /// Check if output is machine-readable
    pub fn is_machine(&self) -> bool {
        matches!(self.mode, OutputMode::Json | OutputMode::Plain)
    }
}

/// Format duration in human-readable form
#[allow(dead_code)]
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

/// Format file size in human-readable form
#[allow(dead_code)]
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_mode() {
        let mode = OutputMode::auto();
        // Will be Plain when running in cargo test (no TTY)
        assert!(matches!(mode, OutputMode::Plain | OutputMode::Human));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3661), "1h 1m");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(2048), "2.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(5 * 1024 * 1024 * 1024), "5.0 GB");
    }
}
