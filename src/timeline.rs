// Timeline analysis - Your coding journey visualized
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
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
    pub resumed: usize,
    pub ongoing: usize,
    pub completion_rate: f64,
    pub avg_session_hours: f64,
    #[allow(dead_code)]
    pub avg_abandonment_days: f64,
    #[allow(dead_code)]
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
        self.analyze_with_options(None, false, false)
    }

    pub fn analyze_with_options(
        &self,
        months_back: Option<i64>,
        cluster: bool,
        skip_noise: bool,
    ) -> Result<Timeline> {
        let mut sessions = Vec::new();

        // AI Coding Assistants
        let claude_sessions = self.parse_claude_projects()?;
        sessions.extend(claude_sessions);

        let cline_sessions = self.parse_cline_tasks()?;
        sessions.extend(cline_sessions);

        let kilo_sessions = self.parse_kilo_tasks()?;
        sessions.extend(kilo_sessions);

        let roo_sessions = self.parse_roo_tasks()?;
        sessions.extend(roo_sessions);

        let cursor_sessions = self.parse_cursor_history()?;
        sessions.extend(cursor_sessions);

        let continue_sessions = self.parse_continue_sessions()?;
        sessions.extend(continue_sessions);

        let aider_sessions = self.parse_aider_logs()?;
        sessions.extend(aider_sessions);

        // More AI tools
        let windsurf_sessions = self.parse_windsurf_logs()?;
        sessions.extend(windsurf_sessions);

        let cody_sessions = self.parse_cody_logs()?;
        sessions.extend(cody_sessions);

        let tabnine_sessions = self.parse_tabnine_logs()?;
        sessions.extend(tabnine_sessions);

        let copilot_sessions = self.parse_copilot_logs()?;
        sessions.extend(copilot_sessions);

        let codegpt_sessions = self.parse_codegpt_logs()?;
        sessions.extend(codegpt_sessions);

        let bito_sessions = self.parse_bito_logs()?;
        sessions.extend(bito_sessions);

        let amazonq_sessions = self.parse_amazonq_logs()?;
        sessions.extend(amazonq_sessions);

        let supermaven_sessions = self.parse_supermaven_logs()?;
        sessions.extend(supermaven_sessions);

        // GIT ANALYSIS - THE BIG ONE!
        let git_sessions = self.parse_git_commits()?;
        sessions.extend(git_sessions);

        // Shell history
        let shell_sessions = self.parse_shell_history()?;
        sessions.extend(shell_sessions);

        // Editor sessions
        let vim_sessions = self.parse_vim_sessions()?;
        sessions.extend(vim_sessions);

        let vscode_sessions = self.parse_vscode_sessions()?;
        sessions.extend(vscode_sessions);

        // Terminal multiplexers
        let tmux_sessions = self.parse_tmux_sessions()?;
        sessions.extend(tmux_sessions);

        // Filter by time range if specified
        if let Some(months) = months_back {
            let cutoff = Utc::now() - Duration::days(months * 30);
            sessions.retain(|s| s.start > cutoff);
        }

        // Filter out noise if requested
        if skip_noise {
            sessions.retain(|s| {
                !s.project.starts_with("Shell")
                    && !s.project.starts_with("Vim")
                    && !s.project.starts_with("VSCode")
                    && !s.project.starts_with("Tmux")
            });
        }

        // Sort by start time
        sessions.sort_by(|a, b| a.start.cmp(&b.start));

        // Cluster sessions if requested
        if cluster {
            sessions = self.cluster_sessions(sessions);
        }

        // Detect patterns
        self.detect_outcomes(&mut sessions);

        // Calculate stats
        let stats = self.calculate_stats(&sessions);

        Ok(Timeline { sessions, stats })
    }

    fn cluster_sessions(&self, sessions: Vec<WorkSession>) -> Vec<WorkSession> {
        if sessions.is_empty() {
            return sessions;
        }

        let mut clustered = Vec::new();
        let mut current_cluster: Vec<WorkSession> = vec![sessions[0].clone()];

        let cluster_window = Duration::hours(2); // Sessions within 2 hours = same cluster

        for session in sessions.iter().skip(1) {
            let last = current_cluster.last().unwrap();

            // Same project and within time window?
            let same_project = session.project == last.project;
            let time_diff = session.start - last.end;
            let within_window = time_diff < cluster_window && time_diff > Duration::hours(-1);

            if same_project && within_window {
                // Add to current cluster
                current_cluster.push(session.clone());
            } else {
                // Finish current cluster and start new one
                if current_cluster.len() > 1 {
                    // Merge cluster into single session
                    clustered.push(self.merge_cluster(&current_cluster));
                } else {
                    clustered.push(current_cluster[0].clone());
                }
                current_cluster = vec![session.clone()];
            }
        }

        // Don't forget the last cluster
        if current_cluster.len() > 1 {
            clustered.push(self.merge_cluster(&current_cluster));
        } else if !current_cluster.is_empty() {
            clustered.push(current_cluster[0].clone());
        }

        clustered
    }

    fn merge_cluster(&self, cluster: &[WorkSession]) -> WorkSession {
        let first = &cluster[0];
        let last = &cluster[cluster.len() - 1];

        let total_hours: f64 = cluster.iter().map(|s| s.hours).sum();
        let total_messages: usize = cluster.iter().map(|s| s.messages).sum();
        let total_convos: usize = cluster.iter().map(|s| s.conversations).sum();

        // Create description from first few sessions
        let descriptions: Vec<String> = cluster
            .iter()
            .take(3)
            .filter(|s| !s.description.is_empty())
            .map(|s| s.description.clone())
            .collect();

        let description = if descriptions.is_empty() {
            format!("{} sessions merged", cluster.len())
        } else if descriptions.len() == 1 {
            descriptions[0].clone()
        } else {
            format!("{} +{} more", descriptions[0], cluster.len() - 1)
        };

        WorkSession {
            id: format!("cluster-{}", first.id),
            start: first.start,
            end: last.end,
            project: first.project.clone(),
            description,
            conversations: total_convos,
            messages: total_messages,
            outcome: first.outcome.clone(),
            resumed_from: first.resumed_from.clone(),
            hours: total_hours,
        }
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
                        if let Some(msg) = entry.get("userMessage").or_else(|| entry.get("prompt"))
                        {
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
        let tasks_dir = self
            .base_dir
            .join(".config/Code/User/globalStorage/saoudrizwan.claude-dev/tasks");

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
                            let ts_utc =
                                DateTime::from_timestamp(ts / 1000, 0).unwrap_or(Utc::now());
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

    fn parse_kilo_tasks(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let tasks_dir = self
            .base_dir
            .join(".config/Code/User/globalStorage/kilocode.kilo-code/tasks");

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
                            let ts_utc =
                                DateTime::from_timestamp(ts / 1000, 0).unwrap_or(Utc::now());
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
                            project: "Kilo".to_string(),
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

    fn parse_roo_tasks(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let tasks_dir = self
            .base_dir
            .join(".config/Code/User/globalStorage/rooveterinaryinc.roo-cline/tasks");

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
                            let ts_utc =
                                DateTime::from_timestamp(ts / 1000, 0).unwrap_or(Utc::now());
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
                            project: "Roo-Cline".to_string(),
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

    fn parse_cursor_history(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();

        // Try multiple Cursor locations
        let cursor_dirs = vec![
            self.base_dir.join(".cursor"),
            self.base_dir.join(".config/Cursor/User/globalStorage"),
        ];

        for cursor_dir in cursor_dirs {
            if !cursor_dir.exists() {
                continue;
            }

            // Look for chat.log, main.log, or history files
            if let Ok(entries) = fs::read_dir(&cursor_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    if filename.contains("chat") || filename.contains("history") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let lines: Vec<&str> = content.lines().collect();
                            if lines.is_empty() {
                                continue;
                            }

                            let mut start_time: Option<DateTime<Utc>> = None;
                            let mut end_time: Option<DateTime<Utc>> = None;
                            let mut description = String::new();
                            let mut message_count = 0;

                            for line in &lines {
                                message_count += 1;

                                // Try parsing as JSON first
                                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                                    if let Some(ts_str) =
                                        entry.get("timestamp").and_then(|t| t.as_str())
                                    {
                                        if let Ok(ts) = DateTime::parse_from_rfc3339(ts_str) {
                                            let ts_utc = ts.with_timezone(&Utc);
                                            if start_time.is_none() || ts_utc < start_time.unwrap()
                                            {
                                                start_time = Some(ts_utc);
                                            }
                                            if end_time.is_none() || ts_utc > end_time.unwrap() {
                                                end_time = Some(ts_utc);
                                            }
                                        }
                                    }

                                    if description.is_empty() {
                                        if let Some(msg) =
                                            entry.get("message").and_then(|m| m.as_str())
                                        {
                                            description =
                                                msg.lines().next().unwrap_or("").to_string();
                                            if description.len() > 60 {
                                                description = format!("{}...", &description[..60]);
                                            }
                                        }
                                    }
                                }
                            }

                            if let (Some(start), Some(end)) = (start_time, end_time) {
                                let hours = (end - start).num_seconds() as f64 / 3600.0;

                                sessions.push(WorkSession {
                                    id: filename.to_string(),
                                    start,
                                    end,
                                    project: "Cursor".to_string(),
                                    description: if description.is_empty() {
                                        "Cursor session".to_string()
                                    } else {
                                        description
                                    },
                                    conversations: 1,
                                    messages: message_count,
                                    outcome: SessionOutcome::Ongoing,
                                    resumed_from: None,
                                    hours,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_continue_sessions(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();

        let continue_dirs = vec![
            self.base_dir.join(".continue"),
            self.base_dir
                .join(".config/Code/User/globalStorage/continue.continue"),
        ];

        for continue_dir in continue_dirs {
            if !continue_dir.exists() {
                continue;
            }

            // Look for sessions directory
            let sessions_dir = continue_dir.join("sessions");
            if sessions_dir.exists() {
                if let Ok(entries) = fs::read_dir(&sessions_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("json") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(session_data) =
                                    serde_json::from_str::<serde_json::Value>(&content)
                                {
                                    let mut start_time: Option<DateTime<Utc>> = None;
                                    let mut end_time: Option<DateTime<Utc>> = None;
                                    let mut description = String::new();
                                    let mut message_count = 0;

                                    if let Some(messages) =
                                        session_data.get("messages").and_then(|m| m.as_array())
                                    {
                                        message_count = messages.len();

                                        for msg in messages {
                                            if let Some(ts) =
                                                msg.get("timestamp").and_then(|t| t.as_i64())
                                            {
                                                let ts_utc = DateTime::from_timestamp(ts / 1000, 0)
                                                    .unwrap_or(Utc::now());
                                                if start_time.is_none()
                                                    || ts_utc < start_time.unwrap()
                                                {
                                                    start_time = Some(ts_utc);
                                                }
                                                if end_time.is_none() || ts_utc > end_time.unwrap()
                                                {
                                                    end_time = Some(ts_utc);
                                                }
                                            }

                                            if description.is_empty() {
                                                if let Some(content) =
                                                    msg.get("content").and_then(|c| c.as_str())
                                                {
                                                    description = content
                                                        .lines()
                                                        .next()
                                                        .unwrap_or("")
                                                        .to_string();
                                                    if description.len() > 60 {
                                                        description =
                                                            format!("{}...", &description[..60]);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if let (Some(start), Some(end)) = (start_time, end_time) {
                                        let hours = (end - start).num_seconds() as f64 / 3600.0;

                                        sessions.push(WorkSession {
                                            id: path
                                                .file_stem()
                                                .and_then(|s| s.to_str())
                                                .unwrap_or("unknown")
                                                .to_string(),
                                            start,
                                            end,
                                            project: "Continue".to_string(),
                                            description: if description.is_empty() {
                                                "Continue session".to_string()
                                            } else {
                                                description
                                            },
                                            conversations: 1,
                                            messages: message_count,
                                            outcome: SessionOutcome::Ongoing,
                                            resumed_from: None,
                                            hours,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_aider_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let aider_dir = self.base_dir.join(".aider");

        if !aider_dir.exists() {
            return Ok(sessions);
        }

        // Look for .aider.chat.history.md files
        if let Ok(entries) = fs::read_dir(&aider_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if filename.contains("history") || filename.ends_with(".md") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let lines: Vec<&str> = content.lines().collect();
                        if lines.is_empty() {
                            continue;
                        }

                        // Aider logs are markdown format with timestamps in headers
                        let mut start_time: Option<DateTime<Utc>> = None;
                        let mut end_time: Option<DateTime<Utc>> = None;
                        let mut description = String::new();
                        let mut message_count = 0;

                        for line in &lines {
                            // Look for user messages
                            if line.starts_with(">") || line.starts_with("####") {
                                message_count += 1;

                                if description.is_empty() && line.len() > 2 {
                                    let msg =
                                        line.trim_start_matches('>').trim_start_matches('#').trim();
                                    description = msg.to_string();
                                    if description.len() > 60 {
                                        description = format!("{}...", &description[..60]);
                                    }
                                }
                            }
                        }

                        // Use file metadata for timestamps
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(created) = metadata.created() {
                                start_time = Some(created.into());
                            }
                            if let Ok(modified) = metadata.modified() {
                                end_time = Some(modified.into());
                            }
                        }

                        if let (Some(start), Some(end)) = (start_time, end_time) {
                            let hours = (end - start).num_seconds() as f64 / 3600.0;

                            sessions.push(WorkSession {
                                id: filename.to_string(),
                                start,
                                end,
                                project: "Aider".to_string(),
                                description: if description.is_empty() {
                                    "Aider session".to_string()
                                } else {
                                    description
                                },
                                conversations: 1,
                                messages: message_count,
                                outcome: SessionOutcome::Ongoing,
                                resumed_from: None,
                                hours,
                            });
                        }
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

            // CRITICAL: Don't override sessions that are already marked Completed!
            // Git commits are Completed by definition - they're done when committed
            if matches!(session.outcome, SessionOutcome::Completed) {
                continue;
            }

            let days_since_end = (now - session.end).num_days();

            // Check if ongoing (recent activity)
            if days_since_end <= 1 {
                sessions[i].outcome = SessionOutcome::Ongoing;
                continue;
            }

            // Only check for "resumed" pattern on AI assistant sessions, not git commits
            let is_ai_session = !session.project.starts_with("Git:")
                && !session.project.starts_with("Shell")
                && !session.project.starts_with("Vim")
                && !session.project.starts_with("VSCode")
                && !session.project.starts_with("Tmux");

            if is_ai_session {
                // Check if resumed in a later session
                let mut resumed_at = None;
                for other in sessions.iter().skip(i + 1) {
                    let gap_days = (other.start - session.end).num_days();

                    // Only consider it a resume if gap > 7 days and related
                    if gap_days > 7 && self.is_related(&session.project, &other.project) {
                        resumed_at = Some(other.start);
                        break;
                    }
                }

                if let Some(resumed) = resumed_at {
                    sessions[i].outcome = SessionOutcome::Resumed(resumed);
                } else if days_since_end > 14 && session.hours > 0.5 {
                    // Likely abandoned if no activity for 2 weeks and had some work
                    sessions[i].outcome = SessionOutcome::Abandoned;
                } else {
                    // Unknown - not enough info to determine
                    sessions[i].outcome = SessionOutcome::Ongoing;
                }
            } else {
                // For git/shell/vim/etc: if old enough, mark as completed
                if days_since_end > 1 {
                    sessions[i].outcome = SessionOutcome::Completed;
                }
            }
        }
    }

    fn is_related(&self, desc1: &str, desc2: &str) -> bool {
        let words1: Vec<String> = desc1
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();
        let words2: Vec<String> = desc2
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();

        let common_words: usize = words1
            .iter()
            .filter(|w| words2.contains(w) && w.len() > 3)
            .count();

        common_words >= 2
    }

    fn calculate_stats(&self, sessions: &[WorkSession]) -> TimelineStats {
        let total_sessions = sessions.len();
        let completed = sessions
            .iter()
            .filter(|s| matches!(s.outcome, SessionOutcome::Completed))
            .count();
        let abandoned = sessions
            .iter()
            .filter(|s| matches!(s.outcome, SessionOutcome::Abandoned))
            .count();
        let resumed = sessions
            .iter()
            .filter(|s| matches!(s.outcome, SessionOutcome::Resumed(_)))
            .count();
        let ongoing = sessions
            .iter()
            .filter(|s| matches!(s.outcome, SessionOutcome::Ongoing))
            .count();

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
            .filter(|s| {
                matches!(
                    s.outcome,
                    SessionOutcome::Abandoned | SessionOutcome::Resumed(_)
                )
            })
            .collect();

        let avg_abandonment_days = if !abandoned_sessions.is_empty() {
            let total_days: i64 = abandoned_sessions
                .iter()
                .map(|s| (Utc::now() - s.end).num_days())
                .sum();
            total_days as f64 / abandoned_sessions.len() as f64
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
            resumed,
            ongoing,
            completion_rate,
            avg_session_hours,
            avg_abandonment_days,
            longest_gap_days,
            most_worked_project,
            context_switches,
        }
    }

    // ========== MORE AI TOOLS ==========

    fn parse_windsurf_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let windsurf_dir = self.base_dir.join(".windsurf");

        if !windsurf_dir.exists() {
            return Ok(sessions);
        }

        // Similar to Cursor parsing
        if let Ok(entries) = fs::read_dir(&windsurf_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.contains("chat") || filename.contains("history") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let (Ok(created), Ok(modified)) =
                                (metadata.created(), metadata.modified())
                            {
                                let start_time: DateTime<Utc> = created.into();
                                let end_time: DateTime<Utc> = modified.into();
                                let hours = (end_time - start_time).num_seconds() as f64 / 3600.0;

                                sessions.push(WorkSession {
                                    id: filename.to_string(),
                                    start: start_time,
                                    end: end_time,
                                    project: "Windsurf".to_string(),
                                    description: "Windsurf session".to_string(),
                                    conversations: 1,
                                    messages: 0,
                                    outcome: SessionOutcome::Ongoing,
                                    resumed_from: None,
                                    hours,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_cody_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let cody_dirs = vec![
            self.base_dir.join(".cody"),
            self.base_dir
                .join(".config/Code/User/globalStorage/sourcegraph.cody-ai"),
        ];

        for cody_dir in cody_dirs {
            if cody_dir.exists() {
                if let Ok(entries) = fs::read_dir(&cody_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("json") {
                            if let Ok(metadata) = fs::metadata(&path) {
                                if let (Ok(created), Ok(modified)) =
                                    (metadata.created(), metadata.modified())
                                {
                                    let start_time: DateTime<Utc> = created.into();
                                    let end_time: DateTime<Utc> = modified.into();
                                    let hours =
                                        (end_time - start_time).num_seconds() as f64 / 3600.0;

                                    sessions.push(WorkSession {
                                        id: path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("unknown")
                                            .to_string(),
                                        start: start_time,
                                        end: end_time,
                                        project: "Cody".to_string(),
                                        description: "Cody session".to_string(),
                                        conversations: 1,
                                        messages: 0,
                                        outcome: SessionOutcome::Ongoing,
                                        resumed_from: None,
                                        hours,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_tabnine_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let tabnine_dir = self.base_dir.join(".tabnine");

        if !tabnine_dir.exists() {
            return Ok(sessions);
        }

        // Tabnine stores logs
        if let Ok(entries) = fs::read_dir(&tabnine_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.contains("log") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let (Ok(created), Ok(modified)) =
                                (metadata.created(), metadata.modified())
                            {
                                let start_time: DateTime<Utc> = created.into();
                                let end_time: DateTime<Utc> = modified.into();
                                let hours = (end_time - start_time).num_seconds() as f64 / 3600.0;

                                sessions.push(WorkSession {
                                    id: filename.to_string(),
                                    start: start_time,
                                    end: end_time,
                                    project: "Tabnine".to_string(),
                                    description: "Tabnine session".to_string(),
                                    conversations: 1,
                                    messages: 0,
                                    outcome: SessionOutcome::Ongoing,
                                    resumed_from: None,
                                    hours,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_copilot_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let copilot_dir = self
            .base_dir
            .join(".config/Code/User/globalStorage/github.copilot-chat");

        if !copilot_dir.exists() {
            return Ok(sessions);
        }

        // Parse Copilot chat history
        if let Ok(entries) = fs::read_dir(&copilot_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(chats) = data.get("chats").and_then(|c| c.as_array()) {
                                for chat in chats {
                                    if let Some(ts) = chat.get("timestamp").and_then(|t| t.as_i64())
                                    {
                                        let start_time = DateTime::from_timestamp(ts / 1000, 0)
                                            .unwrap_or(Utc::now());
                                        let end_time = start_time + Duration::hours(1); // Estimate 1 hour

                                        sessions.push(WorkSession {
                                            id: format!("copilot-{}", ts),
                                            start: start_time,
                                            end: end_time,
                                            project: "Copilot".to_string(),
                                            description: "GitHub Copilot chat".to_string(),
                                            conversations: 1,
                                            messages: chat
                                                .get("messages")
                                                .and_then(|m| m.as_array())
                                                .map(|a| a.len())
                                                .unwrap_or(0),
                                            outcome: SessionOutcome::Ongoing,
                                            resumed_from: None,
                                            hours: 1.0,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_codegpt_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let codegpt_dir = self
            .base_dir
            .join(".config/Code/User/globalStorage/danielsanmedium.dscodegpt");

        if codegpt_dir.exists() {
            // Parse CodeGPT logs (similar pattern)
            if let Ok(entries) = fs::read_dir(&codegpt_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let (Ok(created), Ok(modified)) =
                            (metadata.created(), metadata.modified())
                        {
                            let start_time: DateTime<Utc> = created.into();
                            let end_time: DateTime<Utc> = modified.into();
                            let hours = (end_time - start_time).num_seconds() as f64 / 3600.0;

                            sessions.push(WorkSession {
                                id: path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                start: start_time,
                                end: end_time,
                                project: "CodeGPT".to_string(),
                                description: "CodeGPT session".to_string(),
                                conversations: 1,
                                messages: 0,
                                outcome: SessionOutcome::Ongoing,
                                resumed_from: None,
                                hours,
                            });
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_bito_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let bito_dir = self
            .base_dir
            .join(".config/Code/User/globalStorage/bito.bito");

        if bito_dir.exists() {
            if let Ok(entries) = fs::read_dir(&bito_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let (Ok(created), Ok(modified)) =
                            (metadata.created(), metadata.modified())
                        {
                            let start_time: DateTime<Utc> = created.into();
                            let end_time: DateTime<Utc> = modified.into();
                            let hours = (end_time - start_time).num_seconds() as f64 / 3600.0;

                            sessions.push(WorkSession {
                                id: path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                start: start_time,
                                end: end_time,
                                project: "Bito".to_string(),
                                description: "Bito AI session".to_string(),
                                conversations: 1,
                                messages: 0,
                                outcome: SessionOutcome::Ongoing,
                                resumed_from: None,
                                hours,
                            });
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_amazonq_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let amazonq_dir = self
            .base_dir
            .join(".config/Code/User/globalStorage/amazonwebservices.amazon-q-vscode");

        if amazonq_dir.exists() {
            if let Ok(entries) = fs::read_dir(&amazonq_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let (Ok(created), Ok(modified)) =
                            (metadata.created(), metadata.modified())
                        {
                            let start_time: DateTime<Utc> = created.into();
                            let end_time: DateTime<Utc> = modified.into();
                            let hours = (end_time - start_time).num_seconds() as f64 / 3600.0;

                            sessions.push(WorkSession {
                                id: path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                start: start_time,
                                end: end_time,
                                project: "AmazonQ".to_string(),
                                description: "Amazon Q session".to_string(),
                                conversations: 1,
                                messages: 0,
                                outcome: SessionOutcome::Ongoing,
                                resumed_from: None,
                                hours,
                            });
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    fn parse_supermaven_logs(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();
        let supermaven_dir = self.base_dir.join(".supermaven");

        if supermaven_dir.exists() {
            if let Ok(entries) = fs::read_dir(&supermaven_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let (Ok(created), Ok(modified)) =
                            (metadata.created(), metadata.modified())
                        {
                            let start_time: DateTime<Utc> = created.into();
                            let end_time: DateTime<Utc> = modified.into();
                            let hours = (end_time - start_time).num_seconds() as f64 / 3600.0;

                            sessions.push(WorkSession {
                                id: path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string(),
                                start: start_time,
                                end: end_time,
                                project: "Supermaven".to_string(),
                                description: "Supermaven session".to_string(),
                                conversations: 1,
                                messages: 0,
                                outcome: SessionOutcome::Ongoing,
                                resumed_from: None,
                                hours,
                            });
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    // ========== GIT COMMITS - THE BIG ONE! ==========

    fn parse_git_commits(&self) -> Result<Vec<WorkSession>> {
        use std::process::Command;

        let mut sessions = Vec::new();

        // Find all git repos recursively
        let git_repos = self.find_git_repos()?;

        for repo_path in git_repos {
            // Get git log with timestamps
            let output = Command::new("git")
                .arg("-C")
                .arg(&repo_path)
                .arg("log")
                .arg("--all")
                .arg("--pretty=format:%H|%an|%ae|%at|%s")
                .arg("--no-merges") // Skip merge commits
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let log_text = String::from_utf8_lossy(&output.stdout);

                    for line in log_text.lines() {
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() >= 5 {
                            let _commit_hash = parts[0];
                            let _author = parts[1];
                            let timestamp = parts[3].parse::<i64>().unwrap_or(0);
                            let message = parts[4];

                            let commit_time =
                                DateTime::from_timestamp(timestamp, 0).unwrap_or(Utc::now());
                            let _end_time = commit_time + Duration::minutes(30); // Estimate 30min per commit

                            let repo_name = repo_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown-repo")
                                .to_string();

                            sessions.push(WorkSession {
                                id: format!("git-{}-{}", repo_name, timestamp),
                                start: commit_time - Duration::minutes(30),
                                end: commit_time,
                                project: format!("Git: {}", repo_name),
                                description: message.to_string(),
                                conversations: 0,
                                messages: 1,                        // 1 commit
                                outcome: SessionOutcome::Completed, // Commits are completed work!
                                resumed_from: None,
                                hours: 0.5,
                            });
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    pub fn find_git_repos(&self) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut repos = Vec::new();
        let max_depth = 5; // Don't go too deep

        for entry in WalkDir::new(&self.base_dir)
            .max_depth(max_depth)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden dirs except .git
                let name = e.file_name().to_str().unwrap_or("");
                !name.starts_with('.') || name == ".git"
            })
            .flatten()
        {
            if entry.file_type().is_dir() && entry.file_name() == ".git" {
                if let Some(parent) = entry.path().parent() {
                    repos.push(parent.to_path_buf());
                }
            }
        }

        Ok(repos)
    }

    // ========== SHELL HISTORY ==========

    fn parse_shell_history(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();

        // Parse bash history
        let bash_history = self.base_dir.join(".bash_history");
        if bash_history.exists() {
            if let Ok(content) = fs::read_to_string(&bash_history) {
                let commands: Vec<&str> = content.lines().collect();
                if !commands.is_empty() {
                    // Group commands into sessions (every 50 commands = 1 session)
                    for chunk in commands.chunks(50) {
                        if let Ok(metadata) = fs::metadata(&bash_history) {
                            if let Ok(modified) = metadata.modified() {
                                let end_time: DateTime<Utc> = modified.into();
                                let start_time = end_time - Duration::hours(2);

                                sessions.push(WorkSession {
                                    id: format!("bash-session-{}", chunk.len()),
                                    start: start_time,
                                    end: end_time,
                                    project: "Shell".to_string(),
                                    description: format!("{} bash commands", chunk.len()),
                                    conversations: 0,
                                    messages: chunk.len(),
                                    outcome: SessionOutcome::Completed,
                                    resumed_from: None,
                                    hours: 2.0,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Parse zsh history
        let zsh_history = self.base_dir.join(".zsh_history");
        if zsh_history.exists() {
            if let Ok(content) = fs::read_to_string(&zsh_history) {
                let commands: Vec<&str> = content.lines().collect();
                if !commands.is_empty() {
                    if let Ok(metadata) = fs::metadata(&zsh_history) {
                        if let Ok(modified) = metadata.modified() {
                            let end_time: DateTime<Utc> = modified.into();
                            let start_time = end_time - Duration::hours(2);

                            sessions.push(WorkSession {
                                id: "zsh-session".to_string(),
                                start: start_time,
                                end: end_time,
                                project: "Shell".to_string(),
                                description: format!("{} zsh commands", commands.len()),
                                conversations: 0,
                                messages: commands.len(),
                                outcome: SessionOutcome::Completed,
                                resumed_from: None,
                                hours: 2.0,
                            });
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    // ========== VIM SESSIONS ==========

    fn parse_vim_sessions(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();

        // Parse vim/nvim sessions
        let vim_dirs = vec![
            self.base_dir.join(".vim/sessions"),
            self.base_dir.join(".config/nvim/sessions"),
        ];

        for vim_dir in vim_dirs {
            if vim_dir.exists() {
                if let Ok(entries) = fs::read_dir(&vim_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let (Ok(created), Ok(modified)) =
                                (metadata.created(), metadata.modified())
                            {
                                let start_time: DateTime<Utc> = created.into();
                                let end_time: DateTime<Utc> = modified.into();
                                let hours = (end_time - start_time).num_seconds() as f64 / 3600.0;

                                sessions.push(WorkSession {
                                    id: path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("vim-session")
                                        .to_string(),
                                    start: start_time,
                                    end: end_time,
                                    project: "Vim".to_string(),
                                    description: "Vim editing session".to_string(),
                                    conversations: 0,
                                    messages: 0,
                                    outcome: SessionOutcome::Completed,
                                    resumed_from: None,
                                    hours,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    // ========== VSCODE SESSIONS ==========

    fn parse_vscode_sessions(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();

        // Parse VSCode workspace storage
        let vscode_dir = self.base_dir.join(".config/Code/User/workspaceStorage");

        if vscode_dir.exists() {
            if let Ok(entries) = fs::read_dir(&vscode_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let (Ok(created), Ok(modified)) =
                                (metadata.created(), metadata.modified())
                            {
                                let start_time: DateTime<Utc> = created.into();
                                let end_time: DateTime<Utc> = modified.into();
                                let hours = (end_time - start_time).num_seconds() as f64 / 3600.0;

                                sessions.push(WorkSession {
                                    id: path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("vscode")
                                        .to_string(),
                                    start: start_time,
                                    end: end_time,
                                    project: "VSCode".to_string(),
                                    description: "VSCode workspace session".to_string(),
                                    conversations: 0,
                                    messages: 0,
                                    outcome: SessionOutcome::Completed,
                                    resumed_from: None,
                                    hours,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    // ========== TMUX SESSIONS ==========

    fn parse_tmux_sessions(&self) -> Result<Vec<WorkSession>> {
        let mut sessions = Vec::new();

        // Parse tmux resurrect files
        let tmux_dirs = vec![
            self.base_dir.join(".tmux/resurrect"),
            self.base_dir.join(".config/tmux/resurrect"),
        ];

        for tmux_dir in tmux_dirs {
            if tmux_dir.exists() {
                if let Ok(entries) = fs::read_dir(&tmux_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Resurrect files have timestamps in name
                            if filename.contains("tmux_resurrect_") {
                                if let Ok(metadata) = fs::metadata(&path) {
                                    if let (Ok(created), Ok(modified)) =
                                        (metadata.created(), metadata.modified())
                                    {
                                        let start_time: DateTime<Utc> = created.into();
                                        let end_time: DateTime<Utc> = modified.into();
                                        let hours =
                                            (end_time - start_time).num_seconds() as f64 / 3600.0;

                                        sessions.push(WorkSession {
                                            id: filename.to_string(),
                                            start: start_time,
                                            end: end_time,
                                            project: "Tmux".to_string(),
                                            description: "Tmux terminal session".to_string(),
                                            conversations: 0,
                                            messages: 0,
                                            outcome: SessionOutcome::Completed,
                                            resumed_from: None,
                                            hours,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }
}
