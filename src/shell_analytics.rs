// Shell Command Analytics - Productivity patterns from terminal usage
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellAnalytics {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub most_used_commands: Vec<(String, usize)>,
    pub productivity_score: f64,

    // Error analysis
    pub estimated_failures: usize,
    pub failure_rate: f64,
    pub common_errors: Vec<ErrorPattern>,
    pub time_wasted_hours: f64,

    // Workflow patterns
    pub struggle_sessions: Vec<StruggleSession>,
    pub command_chains: Vec<CommandChain>,
    pub average_command_length: f64,

    // Time patterns
    pub commands_by_hour: HashMap<u32, usize>,
    pub most_active_hour: u32,
    pub weekend_vs_weekday_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub error_type: String,
    pub count: usize,
    pub example_commands: Vec<String>,
    pub avg_retries: f64,
    pub estimated_time_wasted_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StruggleSession {
    pub timestamp: DateTime<Utc>,
    pub commands: Vec<String>,
    pub retries: usize,
    pub duration_minutes: f64,
    pub eventually_succeeded: bool,
    pub struggle_type: StruggleType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StruggleType {
    BuildFailures,
    GitConflicts,
    PermissionErrors,
    DependencyIssues,
    TestFailures,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandChain {
    pub pattern: Vec<String>,
    pub frequency: usize,
    pub avg_success_rate: f64,
}

pub struct ShellAnalyzer {
    commands: Vec<ParsedCommand>,
}

#[derive(Debug, Clone)]
struct ParsedCommand {
    text: String,
    timestamp: Option<DateTime<Utc>>,
    base_command: String, // First word (e.g., "git" from "git commit")
    is_likely_error: bool,
}

impl ShellAnalyzer {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Load commands from shell history
    pub fn load_history(&mut self, history_content: &str, shell_type: &str) {
        for line in history_content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let (timestamp, command_text) = self.parse_history_line(line, shell_type);

            if command_text.is_empty() {
                continue;
            }

            let base_command = command_text
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();

            let is_error = self.is_likely_error(&command_text);

            self.commands.push(ParsedCommand {
                text: command_text.to_string(),
                timestamp,
                base_command,
                is_likely_error: is_error,
            });
        }
    }

    fn parse_history_line(&self, line: &str, shell_type: &str) -> (Option<DateTime<Utc>>, String) {
        // Zsh format: : 1234567890:0;command
        // Bash format: usually just "command" (no timestamp)

        if shell_type == "zsh" && line.starts_with(':') {
            // Parse zsh extended history format
            let parts: Vec<&str> = line.splitn(3, ';').collect();
            if parts.len() == 2 {
                let command = parts[1].trim();
                // Extract timestamp from ": 1234567890:0"
                let time_part = parts[0].trim_start_matches(':').trim();
                if let Some(ts_str) = time_part.split(':').next() {
                    if let Ok(timestamp) = ts_str.parse::<i64>() {
                        let dt = DateTime::from_timestamp(timestamp, 0);
                        return (dt, command.to_string());
                    }
                }
                return (None, command.to_string());
            }
        }

        // Default: no timestamp, just command
        (None, line.trim().to_string())
    }

    fn is_likely_error(&self, command: &str) -> bool {
        // Heuristics for detecting failed/error commands
        let error_patterns = [
            "npm ERR",
            "error:",
            "Error:",
            "ERROR:",
            "fatal:",
            "FAILED",
            "failed",
            "cargo build", // Often followed by errors in next command
            "cargo test",
            "npm install",
            "permission denied",
            "command not found",
            "No such file",
            "cannot find",
        ];

        error_patterns
            .iter()
            .any(|pattern| command.contains(pattern))
    }

    pub fn analyze(&self) -> ShellAnalytics {
        let total_commands = self.commands.len();
        let unique_commands = self.count_unique_commands();
        let most_used = self.find_most_used_commands(20);

        // Error analysis
        let estimated_failures = self.commands.iter().filter(|c| c.is_likely_error).count();
        let failure_rate = if total_commands > 0 {
            estimated_failures as f64 / total_commands as f64 * 100.0
        } else {
            0.0
        };

        let common_errors = self.analyze_error_patterns();
        let time_wasted = self.estimate_time_wasted(&common_errors);

        // Workflow patterns
        let struggle_sessions = self.detect_struggle_sessions();
        let command_chains = self.find_common_chains();

        let avg_length: f64 = if !self.commands.is_empty() {
            self.commands.iter().map(|c| c.text.len()).sum::<usize>() as f64
                / self.commands.len() as f64
        } else {
            0.0
        };

        // Time patterns
        let commands_by_hour = self.commands_by_hour_of_day();
        let most_active_hour = commands_by_hour
            .iter()
            .max_by_key(|(_, count)| **count)
            .map(|(hour, _)| *hour)
            .unwrap_or(0);

        let productivity_score =
            self.calculate_productivity_score(failure_rate, struggle_sessions.len());

        ShellAnalytics {
            total_commands,
            unique_commands,
            most_used_commands: most_used,
            productivity_score,
            estimated_failures,
            failure_rate,
            common_errors,
            time_wasted_hours: time_wasted,
            struggle_sessions,
            command_chains,
            average_command_length: avg_length,
            commands_by_hour,
            most_active_hour,
            weekend_vs_weekday_ratio: 1.0, // TODO: calculate from timestamps
        }
    }

    fn count_unique_commands(&self) -> usize {
        let mut unique = std::collections::HashSet::new();
        for cmd in &self.commands {
            unique.insert(&cmd.base_command);
        }
        unique.len()
    }

    fn find_most_used_commands(&self, limit: usize) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for cmd in &self.commands {
            *counts.entry(cmd.base_command.clone()).or_insert(0) += 1;
        }

        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(limit);
        sorted
    }

    fn analyze_error_patterns(&self) -> Vec<ErrorPattern> {
        let mut patterns = Vec::new();

        // Group by error type
        let mut npm_errors = Vec::new();
        let mut cargo_errors = Vec::new();
        let mut git_errors = Vec::new();
        let mut permission_errors = Vec::new();

        for cmd in &self.commands {
            if cmd.text.contains("npm") && (cmd.text.contains("ERR") || cmd.text.contains("error"))
            {
                npm_errors.push(cmd.text.clone());
            } else if cmd.text.contains("cargo") && cmd.is_likely_error {
                cargo_errors.push(cmd.text.clone());
            } else if cmd.text.contains("git") && cmd.is_likely_error {
                git_errors.push(cmd.text.clone());
            } else if cmd.text.to_lowercase().contains("permission denied") {
                permission_errors.push(cmd.text.clone());
            }
        }

        if !npm_errors.is_empty() {
            patterns.push(ErrorPattern {
                error_type: "NPM Errors".to_string(),
                count: npm_errors.len(),
                example_commands: npm_errors.iter().take(3).cloned().collect(),
                avg_retries: 2.8,
                estimated_time_wasted_minutes: npm_errors.len() as f64 * 5.0,
            });
        }

        if !cargo_errors.is_empty() {
            patterns.push(ErrorPattern {
                error_type: "Cargo Build/Test Failures".to_string(),
                count: cargo_errors.len(),
                example_commands: cargo_errors.iter().take(3).cloned().collect(),
                avg_retries: 3.2,
                estimated_time_wasted_minutes: cargo_errors.len() as f64 * 8.0,
            });
        }

        if !git_errors.is_empty() {
            patterns.push(ErrorPattern {
                error_type: "Git Errors".to_string(),
                count: git_errors.len(),
                example_commands: git_errors.iter().take(3).cloned().collect(),
                avg_retries: 2.1,
                estimated_time_wasted_minutes: git_errors.len() as f64 * 3.0,
            });
        }

        if !permission_errors.is_empty() {
            patterns.push(ErrorPattern {
                error_type: "Permission Denied".to_string(),
                count: permission_errors.len(),
                example_commands: permission_errors.iter().take(3).cloned().collect(),
                avg_retries: 1.5,
                estimated_time_wasted_minutes: permission_errors.len() as f64 * 2.0,
            });
        }

        patterns.sort_by(|a, b| b.count.cmp(&a.count));
        patterns
    }

    fn estimate_time_wasted(&self, error_patterns: &[ErrorPattern]) -> f64 {
        error_patterns
            .iter()
            .map(|p| p.estimated_time_wasted_minutes)
            .sum::<f64>()
            / 60.0 // Convert to hours
    }

    fn detect_struggle_sessions(&self) -> Vec<StruggleSession> {
        let mut sessions = Vec::new();
        let mut current_struggle: Vec<&ParsedCommand> = Vec::new();

        for (i, cmd) in self.commands.iter().enumerate() {
            // Check if this looks like part of a struggle
            let is_struggle_command = cmd.is_likely_error
                || cmd.base_command == "npm"
                || cmd.base_command == "cargo"
                || cmd.base_command == "git";

            if is_struggle_command {
                current_struggle.push(cmd);
            } else {
                // End of struggle session
                if current_struggle.len() >= 3 {
                    // This was a struggle (3+ related commands)
                    let struggle_type = self.classify_struggle(&current_struggle);

                    sessions.push(StruggleSession {
                        timestamp: current_struggle
                            .first()
                            .and_then(|c| c.timestamp)
                            .unwrap_or_else(Utc::now),
                        commands: current_struggle.iter().map(|c| c.text.clone()).collect(),
                        retries: current_struggle.len(),
                        duration_minutes: current_struggle.len() as f64 * 2.0, // Estimate
                        eventually_succeeded: i < self.commands.len() - 1,     // Heuristic
                        struggle_type,
                    });
                }
                current_struggle.clear();
            }
        }

        sessions
    }

    fn classify_struggle(&self, commands: &[&ParsedCommand]) -> StruggleType {
        let text = commands
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        if text.contains("cargo build") || text.contains("cargo test") {
            StruggleType::BuildFailures
        } else if text.contains("git merge") || text.contains("git rebase") {
            StruggleType::GitConflicts
        } else if text.contains("permission") || text.contains("sudo") {
            StruggleType::PermissionErrors
        } else if text.contains("npm install") || text.contains("yarn") {
            StruggleType::DependencyIssues
        } else if text.contains("test") || text.contains("spec") {
            StruggleType::TestFailures
        } else {
            StruggleType::Unknown
        }
    }

    fn find_common_chains(&self) -> Vec<CommandChain> {
        // Find common 2-3 command sequences
        let mut chains: HashMap<Vec<String>, usize> = HashMap::new();

        for window in self.commands.windows(2) {
            let pattern: Vec<String> = window.iter().map(|c| c.base_command.clone()).collect();
            *chains.entry(pattern).or_insert(0) += 1;
        }

        let mut result: Vec<_> = chains
            .into_iter()
            .filter(|(_, count)| *count >= 3) // At least 3 occurrences
            .map(|(pattern, frequency)| CommandChain {
                pattern,
                frequency,
                avg_success_rate: 0.75, // Estimate
            })
            .collect();

        result.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        result.truncate(10);
        result
    }

    fn commands_by_hour_of_day(&self) -> HashMap<u32, usize> {
        let mut by_hour: HashMap<u32, usize> = HashMap::new();

        for cmd in &self.commands {
            if let Some(ts) = cmd.timestamp {
                use chrono::Timelike;
                let hour = ts.hour();
                *by_hour.entry(hour).or_insert(0) += 1;
            }
        }

        by_hour
    }

    fn calculate_productivity_score(&self, failure_rate: f64, struggle_count: usize) -> f64 {
        // Score from 0-100
        let base_score = 100.0;

        // Penalize for failures
        let failure_penalty = failure_rate * 0.5; // Up to 50 points

        // Penalize for struggles
        let struggle_penalty = (struggle_count as f64 * 2.0).min(30.0); // Up to 30 points

        (base_score - failure_penalty - struggle_penalty).max(0.0)
    }
}

impl Default for ShellAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
