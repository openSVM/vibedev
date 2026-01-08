// AI Impact Analyzer - Correlate AI usage with git commits to measure real productivity impact
use anyhow::Result;
use chrono::{DateTime, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPairProgrammingSession {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub claude_conversation_id: String,
    pub git_commits: Vec<GitCommitInfo>,
    pub project: String,
    pub lines_added: usize,
    pub lines_deleted: usize,
    pub files_changed: usize,
    pub languages: Vec<String>,
    pub conversation_turns: usize,
    pub ai_suggestions_accepted: usize, // Tool uses that led to commits
    pub session_type: PairProgrammingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PairProgrammingType {
    IntenseCollaboration, // Many commits during conversation
    ClaudeGuidedRefactor, // Large changes with AI guidance
    QuickFix,             // Single commit after short conversation
    LearningSession,      // Conversation but no commits (learning)
    CopyPasteFromClaude,  // Suspiciously fast commit after AI response
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitInfo {
    pub hash: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub language_breakdown: HashMap<String, usize>, // language -> lines changed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConversation {
    pub id: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub messages: usize,
    pub tool_uses: usize,
    pub file_operations: usize,
    pub project_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIImpactReport {
    pub total_sessions: usize,
    pub ai_assisted_commits: usize,
    pub solo_commits: usize,
    pub ai_assistance_rate: f64,

    // Productivity metrics
    pub avg_commit_velocity_with_ai: f64, // commits per hour
    pub avg_commit_velocity_without_ai: f64,
    pub velocity_improvement: f64, // percentage

    // Code volume metrics
    pub lines_written_with_ai: usize,
    pub lines_written_solo: usize,
    pub ai_contribution_percentage: f64,

    // Quality indicators
    pub avg_files_per_commit_with_ai: f64,
    pub avg_files_per_commit_solo: f64,
    pub refactor_sessions: usize,

    // Patterns
    pub most_ai_assisted_language: String,
    pub most_productive_hour_with_ai: u32,
    pub learning_curve: Vec<MonthlyAIDependency>,

    // Session breakdown
    pub pair_programming_sessions: Vec<AIPairProgrammingSession>,
    pub copy_paste_incidents: usize,
    pub deep_collaboration_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAIDependency {
    pub month: String,
    pub ai_assisted_commits: usize,
    pub solo_commits: usize,
    pub dependency_percentage: f64,
}

pub struct AIImpactAnalyzer {
    claude_conversations: Vec<ClaudeConversation>,
    git_commits: Vec<GitCommitInfo>,
    correlation_window: Duration, // How close in time for correlation
}

impl AIImpactAnalyzer {
    pub fn new() -> Self {
        Self {
            claude_conversations: Vec::new(),
            git_commits: Vec::new(),
            correlation_window: Duration::hours(2), // Commits within 2h of conversation = AI assisted
        }
    }

    pub fn load_claude_conversations(&mut self, claude_dir: &Path) -> Result<()> {
        use serde_json::Value;
        use std::fs;

        let projects_dir = claude_dir.join("projects");
        if !projects_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(projects_dir)? {
            let entry = entry?;
            let history_file = entry.path().join("history.jsonl");

            if history_file.exists() {
                let content = fs::read_to_string(&history_file)?;
                let _current_conversation: Option<ClaudeConversation> = None;
                let mut message_count = 0;
                let mut tool_use_count = 0;
                let mut file_op_count = 0;
                let mut start_time: Option<DateTime<Utc>> = None;
                let mut end_time: Option<DateTime<Utc>> = None;

                for line in content.lines() {
                    if let Ok(value) = serde_json::from_str::<Value>(line) {
                        // Extract timestamp
                        if let Some(ts) = value
                            .get("ts")
                            .or_else(|| value.get("timestamp"))
                            .and_then(|v| v.as_str())
                        {
                            if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
                                let dt_utc = dt.with_timezone(&Utc);
                                if start_time.is_none() {
                                    start_time = Some(dt_utc);
                                }
                                end_time = Some(dt_utc);
                            }
                        }

                        // Count messages and tool uses
                        if value.get("userMessage").is_some()
                            || value.get("assistantMessage").is_some()
                        {
                            message_count += 1;
                        }
                        if value.get("tool_use").is_some() || value.get("toolUse").is_some() {
                            tool_use_count += 1;
                        }
                        if let Some(tool) = value.get("tool").and_then(|t| t.as_str()) {
                            if tool.contains("edit")
                                || tool.contains("write")
                                || tool.contains("read")
                            {
                                file_op_count += 1;
                            }
                        }
                    }
                }

                if let (Some(start), Some(end)) = (start_time, end_time) {
                    self.claude_conversations.push(ClaudeConversation {
                        id: entry.file_name().to_string_lossy().to_string(),
                        start,
                        end,
                        messages: message_count,
                        tool_uses: tool_use_count,
                        file_operations: file_op_count,
                        project_path: entry.path().to_string_lossy().to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn load_git_commits(&mut self, git_repos: &[PathBuf]) -> Result<()> {
        use std::process::Command;

        for repo in git_repos {
            // Get detailed commit info with stats
            let output = Command::new("git")
                .arg("-C")
                .arg(repo)
                .args([
                    "log",
                    "--all",
                    "--numstat",
                    "--pretty=format:COMMIT|%H|%an|%at|%s",
                    "--no-merges",
                ])
                .output()?;

            if output.status.success() {
                let log_text = String::from_utf8_lossy(&output.stdout);
                let mut current_commit: Option<GitCommitInfo> = None;
                let mut insertions = 0;
                let mut deletions = 0;
                let mut files_changed = 0;
                let mut language_breakdown: HashMap<String, usize> = HashMap::new();

                for line in log_text.lines() {
                    if line.starts_with("COMMIT|") {
                        // Save previous commit
                        if let Some(mut commit) = current_commit.take() {
                            commit.insertions = insertions;
                            commit.deletions = deletions;
                            commit.files_changed = files_changed;
                            commit.language_breakdown = language_breakdown.clone();
                            self.git_commits.push(commit);
                        }

                        // Parse new commit
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() >= 5 {
                            let timestamp = parts[3].parse::<i64>().unwrap_or(0);
                            current_commit = Some(GitCommitInfo {
                                hash: parts[1].to_string(),
                                author: parts[2].to_string(),
                                timestamp: DateTime::from_timestamp(timestamp, 0)
                                    .unwrap_or(Utc::now()),
                                message: parts[4].to_string(),
                                files_changed: 0,
                                insertions: 0,
                                deletions: 0,
                                language_breakdown: HashMap::new(),
                            });
                            insertions = 0;
                            deletions = 0;
                            files_changed = 0;
                            language_breakdown.clear();
                        }
                    } else if !line.is_empty() && line.contains('\t') {
                        // Parse numstat line: insertions\tdeletions\tfilename
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 3 {
                            let ins = parts[0].parse::<usize>().unwrap_or(0);
                            let del = parts[1].parse::<usize>().unwrap_or(0);
                            let filename = parts[2];

                            insertions += ins;
                            deletions += del;
                            files_changed += 1;

                            // Detect language from extension
                            if let Some(ext) = filename.split('.').next_back() {
                                let lang = match ext {
                                    "rs" => "Rust",
                                    "js" | "jsx" => "JavaScript",
                                    "ts" | "tsx" => "TypeScript",
                                    "py" => "Python",
                                    "go" => "Go",
                                    "c" | "h" => "C",
                                    "cpp" | "hpp" | "cc" => "C++",
                                    "java" => "Java",
                                    "rb" => "Ruby",
                                    "php" => "PHP",
                                    "swift" => "Swift",
                                    "kt" => "Kotlin",
                                    _ => "Other",
                                };
                                *language_breakdown.entry(lang.to_string()).or_insert(0) +=
                                    ins + del;
                            }
                        }
                    }
                }

                // Save last commit
                if let Some(mut commit) = current_commit {
                    commit.insertions = insertions;
                    commit.deletions = deletions;
                    commit.files_changed = files_changed;
                    commit.language_breakdown = language_breakdown;
                    self.git_commits.push(commit);
                }
            }
        }

        // Sort commits by timestamp
        self.git_commits.sort_by_key(|c| c.timestamp);

        Ok(())
    }

    pub fn analyze(&self) -> AIImpactReport {
        let mut pair_sessions = Vec::new();
        let mut ai_assisted_commits = 0;
        let mut copy_paste_count = 0;
        let mut deep_collaboration_count = 0;

        // Find correlations between conversations and commits
        for conversation in &self.claude_conversations {
            let mut session_commits = Vec::new();

            for commit in &self.git_commits {
                // Check if commit happened during or shortly after conversation
                if commit.timestamp >= conversation.start
                    && commit.timestamp <= conversation.end + self.correlation_window
                {
                    session_commits.push(commit.clone());
                }
            }

            if !session_commits.is_empty() {
                ai_assisted_commits += session_commits.len();

                let total_insertions: usize = session_commits.iter().map(|c| c.insertions).sum();
                let total_deletions: usize = session_commits.iter().map(|c| c.deletions).sum();
                let total_files: usize = session_commits.iter().map(|c| c.files_changed).sum();

                // Merge language breakdowns
                let mut languages = HashMap::new();
                for commit in &session_commits {
                    for (lang, lines) in &commit.language_breakdown {
                        *languages.entry(lang.clone()).or_insert(0) += lines;
                    }
                }

                // Detect session type
                let session_type = self.detect_session_type(
                    conversation,
                    &session_commits,
                    total_insertions + total_deletions,
                );

                if matches!(session_type, PairProgrammingType::CopyPasteFromClaude) {
                    copy_paste_count += 1;
                }
                if matches!(session_type, PairProgrammingType::IntenseCollaboration) {
                    deep_collaboration_count += 1;
                }

                pair_sessions.push(AIPairProgrammingSession {
                    start: conversation.start,
                    end: conversation.end,
                    claude_conversation_id: conversation.id.clone(),
                    git_commits: session_commits,
                    project: conversation.project_path.clone(),
                    lines_added: total_insertions,
                    lines_deleted: total_deletions,
                    files_changed: total_files,
                    languages: languages.keys().cloned().collect(),
                    conversation_turns: conversation.messages,
                    ai_suggestions_accepted: conversation.tool_uses,
                    session_type,
                });
            }
        }

        let solo_commits = self.git_commits.len() - ai_assisted_commits;

        // Calculate velocities
        let (ai_velocity, solo_velocity) = self.calculate_velocities(&pair_sessions);

        // Calculate code volume
        let ai_lines: usize = pair_sessions
            .iter()
            .map(|s| s.lines_added + s.lines_deleted)
            .sum();
        let solo_lines: usize = self
            .git_commits
            .iter()
            .filter(|c| !self.is_commit_ai_assisted(c, &pair_sessions))
            .map(|c| c.insertions + c.deletions)
            .sum();

        // Language analysis
        let most_ai_lang = self.find_most_ai_assisted_language(&pair_sessions);

        // Time analysis
        let most_productive_hour = self.find_most_productive_hour(&pair_sessions);

        // Learning curve
        let learning_curve = self.calculate_learning_curve(&pair_sessions);

        // File complexity
        let (ai_files_avg, solo_files_avg) = self.calculate_avg_files_per_commit(&pair_sessions);

        AIImpactReport {
            total_sessions: pair_sessions.len(),
            ai_assisted_commits,
            solo_commits,
            ai_assistance_rate: ai_assisted_commits as f64
                / (ai_assisted_commits + solo_commits) as f64
                * 100.0,

            avg_commit_velocity_with_ai: ai_velocity,
            avg_commit_velocity_without_ai: solo_velocity,
            velocity_improvement: ((ai_velocity - solo_velocity) / solo_velocity * 100.0).max(0.0),

            lines_written_with_ai: ai_lines,
            lines_written_solo: solo_lines,
            ai_contribution_percentage: ai_lines as f64 / (ai_lines + solo_lines) as f64 * 100.0,

            avg_files_per_commit_with_ai: ai_files_avg,
            avg_files_per_commit_solo: solo_files_avg,
            refactor_sessions: pair_sessions
                .iter()
                .filter(|s| matches!(s.session_type, PairProgrammingType::ClaudeGuidedRefactor))
                .count(),

            most_ai_assisted_language: most_ai_lang,
            most_productive_hour_with_ai: most_productive_hour,
            learning_curve,

            pair_programming_sessions: pair_sessions,
            copy_paste_incidents: copy_paste_count,
            deep_collaboration_count,
        }
    }

    fn detect_session_type(
        &self,
        conversation: &ClaudeConversation,
        commits: &[GitCommitInfo],
        total_lines: usize,
    ) -> PairProgrammingType {
        let duration = conversation.end.signed_duration_since(conversation.start);
        let duration_minutes = duration.num_minutes();

        // Copy-paste detection: Large commit very quickly after AI response
        if commits.len() == 1 && duration_minutes < 5 && total_lines > 50 {
            return PairProgrammingType::CopyPasteFromClaude;
        }

        // Intense collaboration: Multiple commits during conversation
        if commits.len() >= 3 && conversation.tool_uses >= 5 {
            return PairProgrammingType::IntenseCollaboration;
        }

        // Refactor: Many deletions and restructuring
        let deletions: usize = commits.iter().map(|c| c.deletions).sum();
        if deletions as f64 / total_lines as f64 > 0.3 && total_lines > 100 {
            return PairProgrammingType::ClaudeGuidedRefactor;
        }

        // Quick fix
        if commits.len() == 1 && total_lines < 50 {
            return PairProgrammingType::QuickFix;
        }

        PairProgrammingType::IntenseCollaboration
    }

    fn is_commit_ai_assisted(
        &self,
        commit: &GitCommitInfo,
        sessions: &[AIPairProgrammingSession],
    ) -> bool {
        sessions
            .iter()
            .any(|s| s.git_commits.iter().any(|c| c.hash == commit.hash))
    }

    fn calculate_velocities(&self, sessions: &[AIPairProgrammingSession]) -> (f64, f64) {
        if sessions.is_empty() {
            return (0.0, 0.0);
        }

        let total_ai_hours: f64 = sessions
            .iter()
            .map(|s| s.end.signed_duration_since(s.start).num_minutes() as f64 / 60.0)
            .sum();
        let ai_velocity = sessions.iter().map(|s| s.git_commits.len()).sum::<usize>() as f64
            / total_ai_hours.max(0.1);

        // Estimate solo velocity from commits not in any session
        let solo_commits: Vec<_> = self
            .git_commits
            .iter()
            .filter(|c| !self.is_commit_ai_assisted(c, sessions))
            .collect();

        let solo_velocity = if solo_commits.len() > 1 {
            // Estimate based on time span
            let first = solo_commits.first().unwrap().timestamp;
            let last = solo_commits.last().unwrap().timestamp;
            let hours = last.signed_duration_since(first).num_hours() as f64;
            solo_commits.len() as f64 / hours.max(1.0)
        } else {
            0.0
        };

        (ai_velocity, solo_velocity)
    }

    fn find_most_ai_assisted_language(&self, sessions: &[AIPairProgrammingSession]) -> String {
        let mut language_commits: HashMap<String, usize> = HashMap::new();

        for session in sessions {
            for lang in &session.languages {
                *language_commits.entry(lang.clone()).or_insert(0) += 1;
            }
        }

        language_commits
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn find_most_productive_hour(&self, sessions: &[AIPairProgrammingSession]) -> u32 {
        let mut hour_productivity: HashMap<u32, usize> = HashMap::new();

        for session in sessions {
            let hour = session.start.hour();
            *hour_productivity.entry(hour).or_insert(0) += session.git_commits.len();
        }

        hour_productivity
            .into_iter()
            .max_by_key(|(_, commits)| *commits)
            .map(|(hour, _)| hour)
            .unwrap_or(0)
    }

    fn calculate_learning_curve(
        &self,
        sessions: &[AIPairProgrammingSession],
    ) -> Vec<MonthlyAIDependency> {
        use std::collections::BTreeMap;

        let mut monthly_data: BTreeMap<String, (usize, usize)> = BTreeMap::new();

        for session in sessions {
            let month = session.start.format("%Y-%m").to_string();
            let entry = monthly_data.entry(month).or_insert((0, 0));
            entry.0 += session.git_commits.len(); // AI-assisted commits
        }

        // Add solo commits
        for commit in &self.git_commits {
            if !self.is_commit_ai_assisted(commit, sessions) {
                let month = commit.timestamp.format("%Y-%m").to_string();
                let entry = monthly_data.entry(month).or_insert((0, 0));
                entry.1 += 1; // Solo commits
            }
        }

        monthly_data
            .into_iter()
            .map(|(month, (ai, solo))| MonthlyAIDependency {
                month,
                ai_assisted_commits: ai,
                solo_commits: solo,
                dependency_percentage: ai as f64 / (ai + solo) as f64 * 100.0,
            })
            .collect()
    }

    fn calculate_avg_files_per_commit(&self, sessions: &[AIPairProgrammingSession]) -> (f64, f64) {
        let ai_files: usize = sessions
            .iter()
            .flat_map(|s| &s.git_commits)
            .map(|c| c.files_changed)
            .sum();
        let ai_commits: usize = sessions.iter().map(|s| s.git_commits.len()).sum();
        let ai_avg = if ai_commits > 0 {
            ai_files as f64 / ai_commits as f64
        } else {
            0.0
        };

        let solo_commits: Vec<_> = self
            .git_commits
            .iter()
            .filter(|c| !self.is_commit_ai_assisted(c, sessions))
            .collect();
        let solo_files: usize = solo_commits.iter().map(|c| c.files_changed).sum();
        let solo_avg = if !solo_commits.is_empty() {
            solo_files as f64 / solo_commits.len() as f64
        } else {
            0.0
        };

        (ai_avg, solo_avg)
    }
}
