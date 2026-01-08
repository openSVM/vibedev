// Shell history sanitizer - Strip API keys, secrets, and sensitive data
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub struct HistorySanitizer {
    patterns: Vec<(Regex, &'static str)>,
}

impl HistorySanitizer {
    pub fn new() -> Self {
        let patterns = vec![
            // API Keys
            (Regex::new(r"sk-[a-zA-Z0-9]{48}").unwrap(), "[REDACTED_OPENAI_KEY]"),
            (Regex::new(r"sk-ant-[a-zA-Z0-9-_]{95,}").unwrap(), "[REDACTED_ANTHROPIC_KEY]"),
            (Regex::new(r"ghp_[a-zA-Z0-9]{36}").unwrap(), "[REDACTED_GITHUB_TOKEN]"),
            (Regex::new(r"gho_[a-zA-Z0-9]{36}").unwrap(), "[REDACTED_GITHUB_OAUTH]"),
            (Regex::new(r"github_pat_[a-zA-Z0-9_]{82}").unwrap(), "[REDACTED_GITHUB_PAT]"),
            (Regex::new(r"glpat-[a-zA-Z0-9_-]{20}").unwrap(), "[REDACTED_GITLAB_TOKEN]"),
            (Regex::new(r"xox[baprs]-[a-zA-Z0-9-]{10,}").unwrap(), "[REDACTED_SLACK_TOKEN]"),
            (Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(), "[REDACTED_AWS_KEY]"),
            (Regex::new(r"ya29\.[a-zA-Z0-9_-]{100,}").unwrap(), "[REDACTED_GOOGLE_OAUTH]"),
            (Regex::new(r"AIza[a-zA-Z0-9_-]{35}").unwrap(), "[REDACTED_GOOGLE_API_KEY]"),

            // Environment variables with secrets
            (Regex::new(r#"export\s+[A-Z_]*API[A-Z_]*KEY[A-Z_]*=['"]?[^'"\s]+['"]?"#).unwrap(), "export [REDACTED_API_KEY_ENV]"),
            (Regex::new(r#"export\s+[A-Z_]*TOKEN[A-Z_]*=['"]?[^'"\s]+['"]?"#).unwrap(), "export [REDACTED_TOKEN_ENV]"),
            (Regex::new(r#"export\s+[A-Z_]*SECRET[A-Z_]*=['"]?[^'"\s]+['"]?"#).unwrap(), "export [REDACTED_SECRET_ENV]"),
            (Regex::new(r#"export\s+[A-Z_]*PASSWORD[A-Z_]*=['"]?[^'"\s]+['"]?"#).unwrap(), "export [REDACTED_PASSWORD_ENV]"),

            // Bearer tokens and Authorization headers
            (Regex::new(r"Bearer\s+[a-zA-Z0-9._-]+").unwrap(), "Bearer [REDACTED_TOKEN]"),
            (Regex::new(r"Authorization:\s*[^\s]+\s+[a-zA-Z0-9._-]+").unwrap(), "Authorization: [REDACTED]"),

            // Passwords in commands
            (Regex::new(r#"-p\s+['"]?[^'"\s]{6,}['"]?"#).unwrap(), "-p [REDACTED_PASSWORD]"),
            (Regex::new(r#"--password[=\s]['"]?[^'"\s]{6,}['"]?"#).unwrap(), "--password=[REDACTED]"),
            (Regex::new(r#"password=['"]?[^'"\s&]{6,}['"]?"#).unwrap(), "password=[REDACTED]"),

            // Database connection strings
            (Regex::new(r"(postgres|mysql|mongodb)://[^:]+:[^@]+@").unwrap(), "$1://[USER]:[REDACTED]@"),

            // Private keys (detect but redact entire command)
            (Regex::new(r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----").unwrap(), "[REDACTED_PRIVATE_KEY_COMMAND]"),

            // SSH keys in commands
            (Regex::new(r"ssh\s+-i\s+[^\s]+id_[^\s]+").unwrap(), "ssh -i [REDACTED_SSH_KEY]"),

            // Docker login
            (Regex::new(r"docker\s+login\s+.*-p\s+[^\s]+").unwrap(), "docker login [REDACTED_CREDENTIALS]"),

            // Git credentials
            (Regex::new(r"https://[^:]+:[^@]+@github\.com").unwrap(), "https://[USER]:[REDACTED]@github.com"),
            (Regex::new(r"https://[^:]+:[^@]+@gitlab\.com").unwrap(), "https://[USER]:[REDACTED]@gitlab.com"),

            // Email addresses (optional - user might want to keep these)
            // (Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(), "[EMAIL]"),

            // IP addresses with credentials
            (Regex::new(r"//[^:]+:[^@]+@\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap(), "//[USER]:[REDACTED]@[IP]"),

            // Generic secret=value patterns (aggressive)
            (Regex::new(r#"(?i)(secret|api[-_]?key|token|auth[-_]?key|password)\s*[:=]\s*['"]?[a-zA-Z0-9._-]{16,}['"]?"#).unwrap(), "$1=[REDACTED]"),
        ];

        Self { patterns }
    }

    /// Sanitize a single line from shell history
    pub fn sanitize_line(&self, line: &str) -> String {
        let mut sanitized = line.to_string();

        for (pattern, replacement) in &self.patterns {
            sanitized = pattern.replace_all(&sanitized, *replacement).to_string();
        }

        sanitized
    }

    /// Sanitize entire history file
    pub fn sanitize_file(&self, input_path: &PathBuf) -> Result<String> {
        let content = fs::read_to_string(input_path)?;
        let sanitized = content
            .lines()
            .map(|line| self.sanitize_line(line))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(sanitized)
    }

    /// Find and sanitize all shell history files
    pub fn find_and_sanitize_history(&self, home_dir: &Path) -> Result<Vec<(String, String)>> {
        let mut histories = Vec::new();

        // Common shell history locations
        let history_files = vec![
            (".bash_history", "bash"),
            (".zsh_history", "zsh"),
            (".zhistory", "zsh"),
            (".sh_history", "sh"),
            (".fish/fish_history", "fish"),
        ];

        for (file, shell_type) in history_files {
            let path = home_dir.join(file);
            if path.exists() {
                match self.sanitize_file(&path) {
                    Ok(sanitized) => {
                        let name = format!("{}_history_sanitized.txt", shell_type);
                        histories.push((name, sanitized));
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not sanitize {}: {}", file, e);
                    }
                }
            }
        }

        Ok(histories)
    }
}

impl Default for HistorySanitizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_api_keys() {
        let sanitizer = HistorySanitizer::new();

        let test_cases = vec![
            (
                "export OPENAI_API_KEY=sk-abcd1234efgh5678ijkl9012mnop3456qrst7890uvwx",
                "export [REDACTED_API_KEY_ENV]",
            ),
            (
                "curl -H 'Authorization: Bearer sk-ant-api03-abc123def456'",
                "curl -H 'Authorization: Bearer [REDACTED_ANTHROPIC_KEY]'",
            ),
            (
                "git clone https://user:ghp_abc123def456ghi789jkl012mno345@github.com/repo.git",
                "git clone https://[USER]:[REDACTED]@github.com/repo.git",
            ),
            (
                "mysql -u root -p MySecretPass123 -h localhost",
                "mysql -u root [REDACTED_PASSWORD] -h localhost",
            ),
        ];

        for (input, expected) in test_cases {
            let result = sanitizer.sanitize_line(input);
            assert!(
                result.contains("[REDACTED"),
                "Failed to sanitize: {}\nGot: {}\nExpected: {}",
                input,
                result,
                expected
            );
        }
    }

    #[test]
    fn test_preserves_normal_commands() {
        let sanitizer = HistorySanitizer::new();

        let normal_commands = vec![
            "ls -la",
            "cd /home/user/projects",
            "git status",
            "cargo build --release",
            "echo 'Hello World'",
        ];

        for cmd in normal_commands {
            let result = sanitizer.sanitize_line(cmd);
            assert_eq!(result, cmd, "Normal command was modified: {}", cmd);
        }
    }
}
