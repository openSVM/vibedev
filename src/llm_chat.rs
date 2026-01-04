// LLM Chat module - Embedded offline LLM with model selection
use crate::embedded_llm::{
    get_current_model, get_downloaded_models, DeviceType, EmbeddedLlm, Quantization,
};
use anyhow::Result;
use std::collections::HashMap;

pub struct LlmChat {
    llm: Option<EmbeddedLlm>,
    model_id: Option<String>,
    device_type: Option<DeviceType>,
    quantization: Option<Quantization>,
    context: String,
}

impl LlmChat {
    pub fn new(model: Option<String>) -> Self {
        Self {
            llm: None,
            model_id: model,
            device_type: None,
            quantization: None,
            context: String::new(),
        }
    }

    pub fn new_with_options(
        model: Option<String>,
        device_type: Option<DeviceType>,
        quantization: Option<Quantization>,
    ) -> Self {
        Self {
            llm: None,
            model_id: model,
            device_type,
            quantization,
            context: String::new(),
        }
    }

    pub fn backend_name(&self) -> String {
        if let Some(ref llm) = self.llm {
            format!("{} (offline)", llm.model_name())
        } else {
            let current = get_current_model().unwrap_or_else(|| "none".to_string());
            let downloaded = get_downloaded_models();
            if downloaded.is_empty() {
                "No model downloaded - run: vibecheck models".to_string()
            } else {
                format!("{} (offline)", current)
            }
        }
    }

    /// Initialize the embedded LLM
    pub fn init(&mut self) -> Result<()> {
        if self.llm.is_none() {
            let mut llm = EmbeddedLlm::new_with_options(
                self.model_id.as_deref(),
                self.device_type,
                self.quantization,
            )?;
            llm.set_context(&self.context);
            self.llm = Some(llm);
        }
        Ok(())
    }

    /// Check if any model is available
    pub fn has_model(&self) -> bool {
        !get_downloaded_models().is_empty()
    }

    /// Set the data context for the chat
    pub fn set_context(&mut self, findings_summary: &str) {
        self.context = format!(
            r#"AI coding tool usage data:
{}

Help analyze usage patterns and suggest optimizations."#,
            findings_summary
        );

        if let Some(ref mut llm) = self.llm {
            llm.set_context(&self.context);
        }
    }

    /// Generate context from findings
    pub fn generate_context(
        tool_sizes: &HashMap<String, u64>,
        total_size: u64,
        total_files: usize,
        locations_count: usize,
    ) -> String {
        let mut summary = format!(
            "Total Storage: {}\nTotal Files: {}\nLocations: {}\n\nBreakdown by Tool:\n",
            format_bytes(total_size),
            total_files,
            locations_count
        );

        let mut sorted: Vec<_> = tool_sizes.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        for (name, size) in sorted {
            let pct = (*size as f64 / total_size as f64) * 100.0;
            summary.push_str(&format!(
                "- {}: {} ({:.1}%)\n",
                name,
                format_bytes(*size),
                pct
            ));
        }

        summary
    }

    /// Send a chat message and get response
    pub async fn chat(&mut self, user_message: &str) -> Result<String> {
        if self.llm.is_none() {
            self.init()?;
        }

        self.llm
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("LLM not initialized"))?
            .generate(user_message)
    }

    /// Analyze data and provide insights
    pub async fn analyze(&mut self) -> Result<String> {
        if self.llm.is_none() {
            self.init()?;
        }

        self.llm
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("LLM not initialized"))?
            .analyze()
    }

    /// Get specific recommendations
    pub async fn get_recommendations(&mut self, focus: &str) -> Result<String> {
        if self.llm.is_none() {
            self.init()?;
        }

        self.llm
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("LLM not initialized"))?
            .get_recommendations(focus)
    }
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

/// Predefined analysis prompts
pub const ANALYSIS_PROMPTS: &[(&str, &str)] = &[
    (
        "overview",
        "Give me a high-level overview of my AI tool usage",
    ),
    ("storage", "How can I reduce my AI tool storage usage?"),
    ("patterns", "What usage patterns do you see in my data?"),
    ("compare", "Compare my usage across different AI tools"),
    (
        "optimize",
        "How can I optimize my AI-assisted coding workflow?",
    ),
    (
        "cleanup",
        "What files/logs can I safely delete to free space?",
    ),
];
