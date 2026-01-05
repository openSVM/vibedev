// Timeline analysis - Your coding journey visualized
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSession {
    pub id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub project: String,
    pub description: String,
    pub conversations: usize,
    pub messages: usize,
    pub outcome: SessionOutcome,
    pub resumed_from: Option<String>,
    pub hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionOutcome {
    Completed,
    Abandoned,
    Resumed(DateTime<Utc>),
    Ongoing,
}

impl SessionOutcome {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Completed => "✓",
            Self::Abandoned => "✗",
            Self::Resumed(_) => "↻",
            Self::Ongoing => "●",
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Completed => "Completed".to_string(),
            Self::Abandoned => "Abandoned".to_string(),
            Self::Resumed(at) => format!("Resumed {}", at.format("%b %d")),
            Self::Ongoing => "In Progress".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub sessions: Vec<WorkSession>,
    pub stats: TimelineStats,
}

#[derive(Debug, Clone)]
pub struct TimelineStats {
    pub total_sessions: usize,
    pub completed: usize,
    pub abandoned: usize,
    pub ongoing: usize,
    pub completion_rate: f64,
    pub avg_session_hours: f64,
    pub avg_abandonment_days: f64,
    pub longest_gap_days: i64,
    pub most_worked_project: String,
    pub context_switches: usize,
}

pub struct TimelineAnalyzer {
    base_dir: PathBuf,
}

impl TimelineAnalyzer {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn analyze(&self) -> Result<Timeline> {
        let mut sessions = Vec::new();

        // Parse Claude Code projects
        let claude_sessions = self.parse_claude_projects()?;
        sessions.extend(claude_sessions);

        // Parse Cline tasks
        let cline_sessions = self.parse_cline_tasks()?;
        sessions.extend(cline_sessions);

        // Sort by start time
        sessions.sort_by(|a, b| a.start.cmp(&b.start));

        // Detect patterns
        self.detect_outcomes(&mut sessions);

        // Calculate stats
        let stats = self.calculate_stats(&sessions);

        Ok(Timeline { sessions, stats })
    }

    fn parse_claude_projects(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let projects_dir = self.base_dir.join(".claude/projects");

        if !projects_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(&projects_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let project_name = entry.file_name().to_string_lossy().to_string();
            let history_file = entry.path().join("history.jsonl");

            if !history_file.exists() {
                continue;
            }

            let content = fs::read_to_string(&history_file)?;
            let lines: Vec<&str> = content.lines().collect();

            if lines.is_empty() {
                continue;
            }

            // Parse first and last messages to get time range
            let mut conversations = 0;
            let mut messages = 0;
            let mut start_time: Option<DateTime<Utc>> = None;
            let mut end_time: Option<DateTime<Utc>> = None;
            let mut description = String::new();

            for line in &lines {
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                    messages += 1;

                    // Get timestamp
                    if let Some(ts_str) = entry.get("timestamp").and_then(|t| t.as_str()) {
                        if let Ok(ts) = DateTime::parse_from_rfc3339(ts_str) {
                            let ts_utc = ts.with_timezone(&Utc);
                            if start_time.is_none() || ts_utc < start_time.unwrap() {
                                start_time = Some(ts_utc);
                            }
                            if end_time.is_none() || ts_utc > end_time.unwrap() {
                                end_time = Some(ts_utc);
                            }
                        }
                    }

                    // Extract description from first user message
                    if description.is_empty() {
                        if let Some(msg) = entry.get("userMessage").or_else(|| entry.get("prompt")) {
                            if let Some(text) = msg.as_str() {
                                description = text.lines().next().unwrap_or("").to_string();
                                if description.len() > 60 {
                                    description = format!("{}...", &description[..60]);
                                }
                            }
                        }
                    }

                    // Count conversation starts
                    if entry.get("userMessage").is_some() || entry.get("prompt").is_some() {
                        conversations += 1;
                    }
                }
            }

            if let (Some(start), Some(end)) = (start_time, end_time) {
                let hours = (end - start).num_seconds() as f64 / 3600.0;

                sessions.push(WorkSession {
                    id: project_name.clone(),
                    start,
                    end,
                    project: project_name,
                    description,
                    conversations,
                    messages,
                    outcome: SessionOutcome::Ongoing, // Will be updated
                    resumed_from: None,
                    hours,
                });
            }
        }

        Ok(sessions)
    }

    fn parse_cline_tasks(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let tasks_dir = self.base_dir.join(".config/Code/User/globalStorage/saoudrizwan.claude-dev/tasks");

        if !tasks_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(&tasks_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let task_id = entry.file_name().to_string_lossy().to_string();
            let history_file = entry.path().join("api_conversation_history.json");

            if !history_file.exists() {
                continue;
            }

            let content = fs::read_to_string(&history_file)?;
            if let Ok(history) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(messages_arr) = history.as_array() {
                    if messages_arr.is_empty() {
                        continue;
                    }

                    let mut start_time: Option<DateTime<Utc>> = None;
                    let mut end_time: Option<DateTime<Utc>> = None;
                    let mut description = String::new();

                    for msg in messages_arr {
                        if let Some(ts) = msg.get("timestamp").and_then(|t| t.as_i64()) {
                            let ts_utc = DateTime::from_timestamp(ts / 1000, 0).unwrap_or(Utc::now());
                            if start_time.is_none() || ts_utc < start_time.unwrap() {
                                start_time = Some(ts_utc);
                            }
                            if end_time.is_none() || ts_utc > end_time.unwrap() {
                                end_time = Some(ts_utc);
                            }
                        }

                        if description.is_empty() {
                            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                                description = content.lines().next().unwrap_or("").to_string();
                                if description.len() > 60 {
                                    description = format!("{}...", &description[..60]);
                                }
                            }
                        }
                    }

                    if let (Some(start), Some(end)) = (start_time, end_time) {
                        let hours = (end - start).num_seconds() as f64 / 3600.0;

                        sessions.push(WorkSession {
                            id: task_id.clone(),
                            start,
                            end,
                            project: "Cline".to_string(),
                            description,
                            conversations: 1,
                            messages: messages_arr.len(),
                            outcome: SessionOutcome::Ongoing,
                            resumed_from: None,
                            hours,
                        });
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn detect_outcomes(&self, sessions: &mut [WorkSession]) {
        let now = Utc::now();

        for i in 0..sessions.len() {
            let session = &sessions[i];

            // Check if abandoned (no activity in 7+ days)
            let days_since_end = (now - session.end).num_days();
            if days_since_end > 7 {
                // Check if resumed later
                let mut resumed_at = None;
                for j in (i + 1)..sessions.len() {
                    let other = &sessions[j];
                    if self.is_related(&session.description, &other.description) {
                        resumed_at = Some(other.start);
                        break;
                    }
                }

                if let Some(resumed) = resumed_at {
                    sessions[i].outcome = SessionOutcome::Resumed(resumed);
                } else if session.hours > 1.0 && days_since_end > 30 {
                    sessions[i].outcome = SessionOutcome::Abandoned;
                } else if session.hours > 2.0 {
                    sessions[i].outcome = SessionOutcome::Completed;
                }
            } else {
                sessions[i].outcome = SessionOutcome::Ongoing;
            }
        }
    }

    fn is_related(&self, desc1: &str, desc2: &str) -> bool {
        let words1: Vec<String> = desc1.to_lowercase().split_whitespace().map(String::from).collect();
        let words2: Vec<String> = desc2.to_lowercase().split_whitespace().map(String::from).collect();

        let common_words: usize = words1
            .iter()
            .filter(|w| words2.contains(w) && w.len() > 3)
            .count();

        common_words >= 2
    }

    fn calculate_stats(&self, sessions: &[WorkSession]) -> TimelineStats {
        let total_sessions = sessions.len();
        let completed = sessions.iter().filter(|s| matches!(s.outcome, SessionOutcome::Completed)).count();
        let abandoned = sessions.iter().filter(|s| matches!(s.outcome, SessionOutcome::Abandoned)).count();
        let ongoing = sessions.iter().filter(|s| matches!(s.outcome, SessionOutcome::Ongoing)).count();

        let completion_rate = if total_sessions > 0 {
            (completed as f64 / total_sessions as f64) * 100.0
        } else {
            0.0
        };

        let avg_session_hours = if total_sessions > 0 {
            sessions.iter().map(|s| s.hours).sum::<f64>() / total_sessions as f64
        } else {
            0.0
        };

        let abandoned_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| matches!(s.outcome, SessionOutcome::Abandoned | SessionOutcome::Resumed(_)))
            .collect();

        let avg_abandonment_days = if !abandoned_sessions.is_empty() {
            let total_days: i64 = abandoned_sessions
                .iter()
                .map(|s| (Utc::now() - s.end).num_days())
                .sum();
            (total_days as f64 / abandoned_sessions.len() as f64)
        } else {
            0.0
        };

        let longest_gap_days = if sessions.len() > 1 {
            sessions
                .windows(2)
                .map(|w| (w[1].start - w[0].end).num_days())
                .max()
                .unwrap_or(0)
        } else {
            0
        };

        let mut project_hours: HashMap<String, f64> = HashMap::new();
        for session in sessions {
            *project_hours.entry(session.project.clone()).or_insert(0.0) += session.hours;
        }

        let most_worked_project = project_hours
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "None".to_string());

        let context_switches = if sessions.len() > 1 {
            sessions
                .windows(2)
                .filter(|w| w[0].project != w[1].project)
                .count()
        } else {
            0
        };

        TimelineStats {
            total_sessions,
            completed,
            abandoned,
            ongoing,
            completion_rate,
            avg_session_hours,
            avg_abandonment_days,
            longest_gap_days,
            most_worked_project,
            context_switches,
        }
    }
}
