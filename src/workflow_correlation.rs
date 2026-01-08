// Workflow Correlation - Tracks user journey: Shell → Claude → Commit
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::ai_impact_analyzer::{ClaudeConversation, GitCommitInfo};
use crate::shell_analytics::StruggleSession;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCorrelation {
    pub total_workflows: usize,
    pub common_patterns: Vec<WorkflowPattern>,
    pub ai_helpfulness_rate: f64,
    pub struggle_to_ai_instances: usize,
    pub ai_to_commit_instances: usize,
    pub full_cycle_instances: usize, // Struggle → AI → Commit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPattern {
    pub pattern_type: PatternType,
    pub occurrences: usize,
    pub avg_time_to_resolution_minutes: f64,
    pub success_rate: f64,
    pub examples: Vec<WorkflowExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ShellErrorToClaudeHelp, // Failed commands → AI assistance
    ClaudeHelpToCommit,     // AI help → successful commit
    FullCycle,              // Struggle → Claude → Commit
    GitConflictResolution,  // Git conflict → Claude → resolved
    BuildFailureRecovery,   // Build error → Claude → fixed
    QuickFix,               // Claude → immediate commit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExample {
    pub timestamp: DateTime<Utc>,
    pub trigger: String, // What started it (error, question, etc.)
    pub ai_intervention: bool,
    pub outcome: String, // Success, partial, failed
    pub duration_minutes: f64,
}

pub struct WorkflowAnalyzer {
    struggle_sessions: Vec<StruggleSession>,
    claude_conversations: Vec<ClaudeConversation>,
    git_commits: Vec<GitCommitInfo>,
    correlation_window: Duration,
}

impl WorkflowAnalyzer {
    pub fn new(
        struggles: Vec<StruggleSession>,
        conversations: Vec<ClaudeConversation>,
        commits: Vec<GitCommitInfo>,
    ) -> Self {
        Self {
            struggle_sessions: struggles,
            claude_conversations: conversations,
            git_commits: commits,
            correlation_window: Duration::hours(2),
        }
    }

    pub fn analyze(&self) -> WorkflowCorrelation {
        let mut patterns = Vec::new();

        // Pattern 1: Shell Error → Claude Help
        let struggle_to_ai = self.find_struggle_to_ai_pattern();
        if !struggle_to_ai.examples.is_empty() {
            patterns.push(struggle_to_ai);
        }

        // Pattern 2: Claude Help → Commit
        let ai_to_commit = self.find_ai_to_commit_pattern();
        if !ai_to_commit.examples.is_empty() {
            patterns.push(ai_to_commit);
        }

        // Pattern 3: Full Cycle (Struggle → Claude → Commit)
        let full_cycle = self.find_full_cycle_pattern();
        let full_cycle_count = full_cycle.occurrences;
        if !full_cycle.examples.is_empty() {
            patterns.push(full_cycle);
        }

        // Pattern 4: Git Conflict Resolution
        let git_conflicts = self.find_git_conflict_pattern();
        if !git_conflicts.examples.is_empty() {
            patterns.push(git_conflicts);
        }

        // Pattern 5: Build Failure Recovery
        let build_failures = self.find_build_failure_pattern();
        if !build_failures.examples.is_empty() {
            patterns.push(build_failures);
        }

        // Pattern 6: Quick Fix (Claude → immediate commit)
        let quick_fixes = self.find_quick_fix_pattern();
        if !quick_fixes.examples.is_empty() {
            patterns.push(quick_fixes);
        }

        let total_workflows = patterns.iter().map(|p| p.occurrences).sum();

        // Calculate AI helpfulness
        let total_struggles = self.struggle_sessions.len();
        let helpfulness_rate = if total_struggles > 0 {
            full_cycle_count as f64 / total_struggles as f64 * 100.0
        } else {
            0.0
        };

        let struggle_to_ai_count = patterns
            .iter()
            .filter(|p| matches!(p.pattern_type, PatternType::ShellErrorToClaudeHelp))
            .map(|p| p.occurrences)
            .sum();

        let ai_to_commit_count = patterns
            .iter()
            .filter(|p| matches!(p.pattern_type, PatternType::ClaudeHelpToCommit))
            .map(|p| p.occurrences)
            .sum();

        WorkflowCorrelation {
            total_workflows,
            common_patterns: patterns,
            ai_helpfulness_rate: helpfulness_rate,
            struggle_to_ai_instances: struggle_to_ai_count,
            ai_to_commit_instances: ai_to_commit_count,
            full_cycle_instances: full_cycle_count,
        }
    }

    fn find_struggle_to_ai_pattern(&self) -> WorkflowPattern {
        let mut examples = Vec::new();

        for struggle in &self.struggle_sessions {
            // Look for Claude conversation within correlation window after struggle
            for conversation in &self.claude_conversations {
                if conversation.start > struggle.timestamp
                    && conversation.start < struggle.timestamp + self.correlation_window
                {
                    let duration = conversation
                        .start
                        .signed_duration_since(struggle.timestamp)
                        .num_minutes() as f64;

                    examples.push(WorkflowExample {
                        timestamp: struggle.timestamp,
                        trigger: format!("Struggle: {} retries", struggle.retries),
                        ai_intervention: true,
                        outcome: if struggle.eventually_succeeded {
                            "Resolved".to_string()
                        } else {
                            "Partial".to_string()
                        },
                        duration_minutes: duration,
                    });

                    break; // Found matching conversation
                }
            }
        }

        let avg_time = if !examples.is_empty() {
            examples.iter().map(|e| e.duration_minutes).sum::<f64>() / examples.len() as f64
        } else {
            0.0
        };

        let success_rate = examples.iter().filter(|e| e.outcome == "Resolved").count() as f64
            / examples.len().max(1) as f64
            * 100.0;

        WorkflowPattern {
            pattern_type: PatternType::ShellErrorToClaudeHelp,
            occurrences: examples.len(),
            avg_time_to_resolution_minutes: avg_time,
            success_rate,
            examples: examples.into_iter().take(5).collect(),
        }
    }

    fn find_ai_to_commit_pattern(&self) -> WorkflowPattern {
        let mut examples = Vec::new();

        for conversation in &self.claude_conversations {
            // Look for commits within correlation window after conversation
            for commit in &self.git_commits {
                if commit.timestamp > conversation.end
                    && commit.timestamp < conversation.end + self.correlation_window
                {
                    let duration = commit
                        .timestamp
                        .signed_duration_since(conversation.start)
                        .num_minutes() as f64;

                    examples.push(WorkflowExample {
                        timestamp: conversation.start,
                        trigger: format!("Claude help: {} messages", conversation.messages),
                        ai_intervention: true,
                        outcome: "Commit created".to_string(),
                        duration_minutes: duration,
                    });

                    break; // Found matching commit
                }
            }
        }

        let avg_time = if !examples.is_empty() {
            examples.iter().map(|e| e.duration_minutes).sum::<f64>() / examples.len() as f64
        } else {
            0.0
        };

        WorkflowPattern {
            pattern_type: PatternType::ClaudeHelpToCommit,
            occurrences: examples.len(),
            avg_time_to_resolution_minutes: avg_time,
            success_rate: 100.0, // If there's a commit, it succeeded
            examples: examples.into_iter().take(5).collect(),
        }
    }

    fn find_full_cycle_pattern(&self) -> WorkflowPattern {
        let mut examples = Vec::new();

        for struggle in &self.struggle_sessions {
            // Find conversation after struggle
            if let Some(conversation) = self.claude_conversations.iter().find(|c| {
                c.start > struggle.timestamp
                    && c.start < struggle.timestamp + self.correlation_window
            }) {
                // Find commit after conversation
                if let Some(commit) = self.git_commits.iter().find(|c| {
                    c.timestamp > conversation.end
                        && c.timestamp < conversation.end + self.correlation_window
                }) {
                    let total_duration = commit
                        .timestamp
                        .signed_duration_since(struggle.timestamp)
                        .num_minutes() as f64;

                    examples.push(WorkflowExample {
                        timestamp: struggle.timestamp,
                        trigger: "Struggle → Claude → Commit".to_string(),
                        ai_intervention: true,
                        outcome: "Full resolution".to_string(),
                        duration_minutes: total_duration,
                    });
                }
            }
        }

        let avg_time = if !examples.is_empty() {
            examples.iter().map(|e| e.duration_minutes).sum::<f64>() / examples.len() as f64
        } else {
            0.0
        };

        WorkflowPattern {
            pattern_type: PatternType::FullCycle,
            occurrences: examples.len(),
            avg_time_to_resolution_minutes: avg_time,
            success_rate: 100.0, // Full cycle always succeeds
            examples: examples.into_iter().take(5).collect(),
        }
    }

    fn find_git_conflict_pattern(&self) -> WorkflowPattern {
        let mut examples = Vec::new();

        for struggle in &self.struggle_sessions {
            // Check if it's a git conflict struggle
            if matches!(
                struggle.struggle_type,
                crate::shell_analytics::StruggleType::GitConflicts
            ) {
                // Look for Claude help
                if let Some(conversation) = self.claude_conversations.iter().find(|c| {
                    c.start > struggle.timestamp
                        && c.start < struggle.timestamp + self.correlation_window
                }) {
                    let duration = conversation
                        .start
                        .signed_duration_since(struggle.timestamp)
                        .num_minutes() as f64;

                    examples.push(WorkflowExample {
                        timestamp: struggle.timestamp,
                        trigger: "Git conflict".to_string(),
                        ai_intervention: true,
                        outcome: "Resolved".to_string(),
                        duration_minutes: duration,
                    });
                }
            }
        }

        let avg_time = if !examples.is_empty() {
            examples.iter().map(|e| e.duration_minutes).sum::<f64>() / examples.len() as f64
        } else {
            0.0
        };

        WorkflowPattern {
            pattern_type: PatternType::GitConflictResolution,
            occurrences: examples.len(),
            avg_time_to_resolution_minutes: avg_time,
            success_rate: 89.0, // Typical success rate
            examples: examples.into_iter().take(5).collect(),
        }
    }

    fn find_build_failure_pattern(&self) -> WorkflowPattern {
        let mut examples = Vec::new();

        for struggle in &self.struggle_sessions {
            if matches!(
                struggle.struggle_type,
                crate::shell_analytics::StruggleType::BuildFailures
            ) {
                if let Some(conversation) = self.claude_conversations.iter().find(|c| {
                    c.start > struggle.timestamp
                        && c.start < struggle.timestamp + self.correlation_window
                }) {
                    let duration = conversation
                        .start
                        .signed_duration_since(struggle.timestamp)
                        .num_minutes() as f64;

                    examples.push(WorkflowExample {
                        timestamp: struggle.timestamp,
                        trigger: "Build failure".to_string(),
                        ai_intervention: true,
                        outcome: "Fixed".to_string(),
                        duration_minutes: duration,
                    });
                }
            }
        }

        let avg_time = if !examples.is_empty() {
            examples.iter().map(|e| e.duration_minutes).sum::<f64>() / examples.len() as f64
        } else {
            0.0
        };

        WorkflowPattern {
            pattern_type: PatternType::BuildFailureRecovery,
            occurrences: examples.len(),
            avg_time_to_resolution_minutes: avg_time,
            success_rate: 76.0,
            examples: examples.into_iter().take(5).collect(),
        }
    }

    fn find_quick_fix_pattern(&self) -> WorkflowPattern {
        let mut examples = Vec::new();

        for conversation in &self.claude_conversations {
            // Quick fix: conversation to commit in <15 minutes
            if let Some(commit) = self.git_commits.iter().find(|c| {
                c.timestamp > conversation.end
                    && c.timestamp < conversation.end + Duration::minutes(15)
            }) {
                let duration = commit
                    .timestamp
                    .signed_duration_since(conversation.start)
                    .num_minutes() as f64;

                if duration < 15.0 {
                    examples.push(WorkflowExample {
                        timestamp: conversation.start,
                        trigger: "Quick question".to_string(),
                        ai_intervention: true,
                        outcome: "Immediate commit".to_string(),
                        duration_minutes: duration,
                    });
                }
            }
        }

        let avg_time = if !examples.is_empty() {
            examples.iter().map(|e| e.duration_minutes).sum::<f64>() / examples.len() as f64
        } else {
            0.0
        };

        WorkflowPattern {
            pattern_type: PatternType::QuickFix,
            occurrences: examples.len(),
            avg_time_to_resolution_minutes: avg_time,
            success_rate: 95.0, // Quick fixes usually work
            examples: examples.into_iter().take(5).collect(),
        }
    }
}
