// Claude configuration management module
// Handles switching between different Claude Code API providers

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Supported Claude Code providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaudeProvider {
    #[serde(rename = "z.ai")]
    ZAi,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "chatgpt")]
    ChatGPT,
    #[serde(rename = "litellm")]
    LiteLLM,
    #[serde(rename = "custom")]
    Custom,
}

impl ClaudeProvider {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "z.ai" | "zai" => Ok(ClaudeProvider::ZAi),
            "openrouter" => Ok(ClaudeProvider::OpenRouter),
            "chatgpt" | "openai" => Ok(ClaudeProvider::ChatGPT),
            "litellm" => Ok(ClaudeProvider::LiteLLM),
            "custom" => Ok(ClaudeProvider::Custom),
            _ => Err(anyhow!("Unknown provider: {}. Supported: z.ai, openrouter, chatgpt, litellm, custom", s)),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ClaudeProvider::ZAi => "z.ai",
            ClaudeProvider::OpenRouter => "openrouter",
            ClaudeProvider::ChatGPT => "chatgpt",
            ClaudeProvider::LiteLLM => "litellm",
            ClaudeProvider::Custom => "custom",
        }
    }

    /// Get default endpoint for the provider
    pub fn default_endpoint(&self) -> &str {
        match self {
            ClaudeProvider::ZAi => "https://api.z.ai/v1",
            ClaudeProvider::OpenRouter => "https://openrouter.ai/api/v1",
            ClaudeProvider::ChatGPT => "https://api.openai.com/v1",
            ClaudeProvider::LiteLLM => "http://localhost:4000",
            ClaudeProvider::Custom => "",
        }
    }

    /// Get default model for the provider
    pub fn default_model(&self) -> &str {
        match self {
            ClaudeProvider::ZAi => "claude-3-5-sonnet-20241022",
            ClaudeProvider::OpenRouter => "anthropic/claude-3.5-sonnet",
            ClaudeProvider::ChatGPT => "gpt-4",
            ClaudeProvider::LiteLLM => "claude-3-5-sonnet-20241022",
            ClaudeProvider::Custom => "",
        }
    }
}

/// Claude Code configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub provider: ClaudeProvider,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
}

impl ClaudeConfig {
    /// Create a new configuration for a provider
    pub fn new(provider: ClaudeProvider, api_key: String) -> Self {
        Self {
            endpoint: provider.default_endpoint().to_string(),
            model: provider.default_model().to_string(),
            provider,
            api_key,
            organization_id: None,
        }
    }

    /// Create a custom configuration
    pub fn custom(endpoint: String, api_key: String, model: String) -> Self {
        Self {
            provider: ClaudeProvider::Custom,
            endpoint,
            api_key,
            model,
            organization_id: None,
        }
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path();
        if !config_path.exists() {
            return Err(anyhow!("No Claude configuration found. Use 'vibedev claude set' to configure a provider."));
        }

        let content = fs::read_to_string(&config_path)?;
        let config: ClaudeConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self)?;
        fs::write(&config_path, json)?;
        Ok(())
    }

    /// Get the config file path
    pub fn config_file_path() -> PathBuf {
        dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .expect("Could not determine config directory or home directory")
            .join("vibedev")
            .join("claude_config.json")
    }

    /// Write Claude Code's actual config files
    pub fn write_claude_code_config(&self) -> Result<()> {
        // Claude Code stores its config in different locations based on OS
        let claude_config_paths = Self::get_claude_code_config_paths();
        
        let mut wrote_any = false;
        for path in claude_config_paths {
            if let Some(parent) = path.parent() {
                if parent.exists() || self.try_create_claude_dir(parent) {
                    // Create a simple config structure for Claude Code
                    let config_content = self.generate_claude_code_config();
                    
                    match fs::write(&path, config_content) {
                        Ok(_) => {
                            println!("✓ Updated Claude Code config: {}", path.display());
                            wrote_any = true;
                        }
                        Err(e) => {
                            eprintln!("⚠ Could not write to {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        if wrote_any {
            Ok(())
        } else {
            Err(anyhow!("Could not find or create Claude Code configuration directory. Is Claude Code installed?"))
        }
    }

    fn try_create_claude_dir(&self, path: &std::path::Path) -> bool {
        fs::create_dir_all(path).is_ok()
    }

    /// Get potential Claude Code config file paths
    fn get_claude_code_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        if let Some(home) = dirs::home_dir() {
            // Linux/macOS
            paths.push(home.join(".claude").join("config.json"));
            paths.push(home.join(".config/claude/config.json"));
            
            // macOS specific
            if cfg!(target_os = "macos") {
                paths.push(home.join("Library/Application Support/Claude/config.json"));
            }
            
            // Windows specific
            if cfg!(target_os = "windows") {
                if let Some(appdata) = std::env::var_os("APPDATA") {
                    paths.push(PathBuf::from(appdata).join("Claude/config.json"));
                }
            }
        }
        
        paths
    }

    /// Generate Claude Code compatible configuration
    fn generate_claude_code_config(&self) -> String {
        #[derive(serde::Serialize)]
        struct ClaudeCodeConfig<'a> {
            #[serde(rename = "apiUrl")]
            api_url: &'a str,
            #[serde(rename = "apiKey")]
            api_key: &'a str,
            model: &'a str,
            provider: &'a str,
        }
        
        let config = ClaudeCodeConfig {
            api_url: &self.endpoint,
            api_key: &self.api_key,
            model: &self.model,
            provider: self.provider.name(),
        };
        
        serde_json::to_string(&config).unwrap_or_default()
    }
}

/// List all supported providers
pub fn list_providers() {
    use colored::Colorize;
    
    println!("\n{}", "Supported Claude Code Providers:".cyan().bold());
    println!();
    
    let providers = vec![
        (ClaudeProvider::ZAi, "Z.ai - High-performance Claude API proxy"),
        (ClaudeProvider::OpenRouter, "OpenRouter - Unified LLM API with multiple models"),
        (ClaudeProvider::ChatGPT, "OpenAI ChatGPT - GPT-4 and other OpenAI models"),
        (ClaudeProvider::LiteLLM, "LiteLLM - Local proxy for multiple providers"),
    ];
    
    for (provider, description) in providers {
        println!("  {} - {}", provider.name().green(), description);
        println!("    Endpoint: {}", provider.default_endpoint());
        println!("    Model: {}", provider.default_model());
        println!();
    }
    
    println!("  {} - Custom provider with your own endpoint", "custom".green());
    println!();
}

/// Display current configuration
pub fn show_current_config() -> Result<()> {
    use colored::Colorize;
    
    match ClaudeConfig::load() {
        Ok(config) => {
            println!("\n{}", "Current Claude Code Configuration:".cyan().bold());
            println!();
            println!("  Provider: {}", config.provider.name().green());
            println!("  Endpoint: {}", config.endpoint);
            println!("  Model: {}", config.model);
            if config.api_key.len() > 4 {
                println!("  API Key: ****{}", &config.api_key[config.api_key.len() - 4..]);
            } else {
                println!("  API Key: ****");
            }
            if let Some(org_id) = &config.organization_id {
                println!("  Organization ID: {}", org_id);
            }
            println!();
            println!("  Config file: {}", ClaudeConfig::config_file_path().display());
            Ok(())
        }
        Err(_) => {
            println!("\n{}", "No Claude configuration found.".yellow());
            println!("Use 'vibedev claude set <provider>' to configure a provider.");
            println!();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_from_str() {
        assert_eq!(ClaudeProvider::from_str("z.ai").unwrap(), ClaudeProvider::ZAi);
        assert_eq!(ClaudeProvider::from_str("zai").unwrap(), ClaudeProvider::ZAi);
        assert_eq!(ClaudeProvider::from_str("openrouter").unwrap(), ClaudeProvider::OpenRouter);
        assert_eq!(ClaudeProvider::from_str("chatgpt").unwrap(), ClaudeProvider::ChatGPT);
        assert_eq!(ClaudeProvider::from_str("openai").unwrap(), ClaudeProvider::ChatGPT);
        assert_eq!(ClaudeProvider::from_str("litellm").unwrap(), ClaudeProvider::LiteLLM);
        assert!(ClaudeProvider::from_str("unknown").is_err());
    }

    #[test]
    fn test_provider_endpoints() {
        assert_eq!(ClaudeProvider::ZAi.default_endpoint(), "https://api.z.ai/v1");
        assert_eq!(ClaudeProvider::OpenRouter.default_endpoint(), "https://openrouter.ai/api/v1");
        assert_eq!(ClaudeProvider::ChatGPT.default_endpoint(), "https://api.openai.com/v1");
        assert_eq!(ClaudeProvider::LiteLLM.default_endpoint(), "http://localhost:4000");
    }

    #[test]
    fn test_config_creation() {
        let config = ClaudeConfig::new(ClaudeProvider::ZAi, "test-key-123".to_string());
        assert_eq!(config.provider, ClaudeProvider::ZAi);
        assert_eq!(config.endpoint, "https://api.z.ai/v1");
        assert_eq!(config.api_key, "test-key-123");
        assert!(config.model.contains("claude"));
    }
}
