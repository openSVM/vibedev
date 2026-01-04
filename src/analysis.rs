use crate::models::*;
use crate::parsers::claude::ClaudeParser;
use crate::parsers::cline::ClineParser;
use crate::parsers::cursor::CursorParser;
use crate::parsers::generic::GenericParser;
use crate::parsers::{EntryCategory, LogParser};
use anyhow::Result;
use chrono::Timelike;
use std::collections::HashMap;
use tracing::{debug, info};

pub struct Analyzer {
    tool_filter: Option<String>,
    time_range_days: Option<u32>,
    check_compression: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            tool_filter: None,
            time_range_days: None,
            check_compression: true,
        }
    }

    pub fn with_tool_filter(mut self, tool: Option<String>) -> Self {
        self.tool_filter = tool;
        self
    }

    pub fn with_time_range(mut self, days: Option<u32>) -> Self {
        self.time_range_days = days;
        self
    }

    pub fn with_compression_check(mut self, check: bool) -> Self {
        self.check_compression = check;
        self
    }

    pub async fn analyze(&self) -> Result<AnalysisResults> {
        info!("Starting comprehensive analysis...");

        // Discover logs
        let base_dir = dirs::home_dir().expect("Could not determine home directory");
        let discovery = crate::discovery::LogDiscovery::new(base_dir, true);
        let findings = discovery.scan()?;

        // Analyze each tool
        let mut tools = HashMap::new();
        let mut total_sessions = 0;
        let mut total_prompts = 0;

        for location in &findings.locations {
            let tool_name = location.tool.name().to_string();

            if let Some(ref filter) = self.tool_filter {
                if !tool_name.to_lowercase().contains(&filter.to_lowercase()) {
                    continue;
                }
            }

            let analysis = self.analyze_tool(location).await?;
            total_sessions += analysis.session_count;
            total_prompts += analysis.prompt_count;

            tools.insert(tool_name.clone(), analysis);
        }

        // Estimate tokens (rough: 4 chars per token)
        let total_tokens = total_prompts * 100; // Assume 100 tokens per prompt

        // Generate recommendations
        let recommendations = self.generate_recommendations(&tools, &findings)?;

        // Calculate cost estimate
        let cost_estimate = self.estimate_costs(total_tokens, &tools)?;

        // Find peak usage
        let peak_hour = self.find_peak_hour(&tools);
        let most_used = self.find_most_used_tool(&tools);

        let compressible = self.calculate_compressible(&findings);
        let old_files = self.calculate_old_files(&findings, 30);

        let global_metrics = GlobalMetrics {
            total_storage: findings.total_size_bytes,
            compressible_bytes: compressible,
            old_files_bytes: old_files,
            total_sessions,
            total_prompts,
            estimated_tokens: total_tokens,
            peak_usage_hour: peak_hour,
            most_used_tool: most_used,
        };

        Ok(AnalysisResults {
            tools,
            global_metrics,
            recommendations,
            cost_estimate: Some(cost_estimate),
        })
    }

    async fn analyze_tool(&self, location: &LogLocation) -> Result<ToolAnalysis> {
        // Select appropriate parser based on tool
        let parsers: Vec<Box<dyn LogParser>> = vec![
            Box::new(ClaudeParser),
            Box::new(ClineParser),
            Box::new(CursorParser),
            Box::new(GenericParser),
        ];

        let mut session_count = estimate_sessions(location);
        let mut prompt_count = estimate_prompts(location);
        let mut hourly_distribution: HashMap<u8, u64> = HashMap::new();
        let mut user_prompts = 0u64;
        let mut assistant_responses = 0u64;

        // Find parser that can handle this location
        for parser in &parsers {
            if parser.can_parse(&location.path) {
                debug!("Using parser for: {}", location.path.display());
                match parser.parse(&location.path) {
                    Ok(parsed) => {
                        // Extract metrics from parsed log
                        session_count = parsed.metadata.entry_count.max(1);

                        for entry in &parsed.entries {
                            match entry.category {
                                EntryCategory::UserPrompt => user_prompts += 1,
                                EntryCategory::AssistantResponse => assistant_responses += 1,
                                _ => {}
                            }

                            // Build hourly distribution
                            if let Some(ts) = entry.timestamp {
                                let hour = ts.hour() as u8;
                                *hourly_distribution.entry(hour).or_insert(0) += 1;
                            }
                        }

                        prompt_count = user_prompts;
                        break;
                    }
                    Err(e) => {
                        debug!("Parser error for {}: {}", location.path.display(), e);
                    }
                }
            }
        }

        let avg_session_length = if session_count > 0 {
            (user_prompts + assistant_responses) as f64 / session_count as f64
        } else {
            0.0
        };

        Ok(ToolAnalysis {
            tool_name: location.tool.name().to_string(),
            total_size: location.size_bytes,
            file_count: location.file_count,
            session_count,
            prompt_count,
            avg_session_length,
            date_range: (location.oldest_entry, location.newest_entry),
            usage_patterns: UsagePatterns {
                hourly_distribution,
                daily_distribution: HashMap::new(),
                top_commands: vec![],
                top_projects: vec![],
                avg_session_duration_mins: avg_session_length * 2.0, // rough estimate
            },
            issues: vec![],
            storage_breakdown: HashMap::new(),
        })
    }

    fn generate_recommendations(
        &self,
        tools: &HashMap<String, ToolAnalysis>,
        findings: &DiscoveryFindings,
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Storage optimization
        let total_mb = findings.total_size_bytes / 1024 / 1024;
        if total_mb > 500 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Storage,
                priority: Priority::High,
                title: "Large storage footprint detected".to_string(),
                description: format!("AI tool logs are consuming {} of disk space", format_bytes(findings.total_size_bytes)),
                action: "Create a backup archive of your AI logs to preserve this valuable learning data: vibecheck backup".to_string(),
                estimated_savings: Some(findings.total_size_bytes / 2),
                effort: Effort::Minutes,
            });
        }

        // Tool-specific recommendations
        for (tool_name, analysis) in tools {
            // High prompt count suggests power user - recommend optimization
            if analysis.prompt_count > 1000 {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Performance,
                    priority: Priority::Medium,
                    title: format!("High usage detected for {}", tool_name),
                    description: format!(
                        "{} has {} prompts - consider reviewing for patterns",
                        tool_name, analysis.prompt_count
                    ),
                    action:
                        "Run 'vibecheck insights' to analyze usage patterns and optimize workflows"
                            .to_string(),
                    estimated_savings: None,
                    effort: Effort::Minutes,
                });
            }

            // Low session efficiency
            if analysis.avg_session_length < 5.0 && analysis.session_count > 10 {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Performance,
                    priority: Priority::Low,
                    title: format!("Short sessions detected for {}", tool_name),
                    description: format!(
                        "Average session has only {:.1} messages - consider batching requests",
                        analysis.avg_session_length
                    ),
                    action: "Group related tasks into single sessions for better context retention"
                        .to_string(),
                    estimated_savings: None,
                    effort: Effort::Hours,
                });
            }

            // Large tool-specific storage
            if analysis.total_size > 100 * 1024 * 1024 {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Storage,
                    priority: Priority::Medium,
                    title: format!("{} logs consuming significant space", tool_name),
                    description: format!(
                        "{} is using {}",
                        tool_name,
                        format_bytes(analysis.total_size)
                    ),
                    action: format!(
                        "Run 'vibecheck backup --tool {}' to archive old logs",
                        tool_name.to_lowercase()
                    ),
                    estimated_savings: Some(analysis.total_size / 2),
                    effort: Effort::Minutes,
                });
            }
        }

        // Multi-tool recommendations
        if tools.len() > 3 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Performance,
                priority: Priority::Low,
                title: "Multiple AI tools detected".to_string(),
                description: format!(
                    "You're using {} different AI tools - consider consolidating",
                    tools.len()
                ),
                action: "Run 'vibecheck compare' to see which tools are most effective".to_string(),
                estimated_savings: None,
                effort: Effort::Hours,
            });
        }

        Ok(recommendations)
    }

    fn estimate_costs(
        &self,
        total_tokens: u64,
        tools: &HashMap<String, ToolAnalysis>,
    ) -> Result<CostEstimate> {
        // Rough cost estimation
        // Assume Claude Sonnet: $3/M input, $15/M output
        // Rough split: 60% input, 40% output

        let input_tokens = (total_tokens as f64 * 0.6) as u64;
        let output_tokens = (total_tokens as f64 * 0.4) as u64;

        let input_cost = (input_tokens as f64 / 1_000_000.0) * 3.0;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * 15.0;

        let monthly_cost = input_cost + output_cost;

        let mut breakdown = HashMap::new();
        for name in tools.keys() {
            breakdown.insert(name.clone(), monthly_cost / tools.len() as f64);
        }

        Ok(CostEstimate {
            monthly_cost_usd: monthly_cost,
            token_count: total_tokens,
            breakdown_by_tool: breakdown,
            optimization_potential: monthly_cost * 0.3, // 30% potential savings
        })
    }

    fn find_peak_hour(&self, tools: &HashMap<String, ToolAnalysis>) -> u8 {
        let mut hourly = HashMap::new();

        for analysis in tools.values() {
            for (hour, count) in &analysis.usage_patterns.hourly_distribution {
                *hourly.entry(*hour).or_insert(0) += count;
            }
        }

        hourly
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| *hour)
            .unwrap_or(14) // Default to 2 PM
    }

    fn find_most_used_tool(&self, tools: &HashMap<String, ToolAnalysis>) -> String {
        tools
            .iter()
            .max_by_key(|(_, analysis)| analysis.prompt_count)
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn calculate_compressible(&self, findings: &DiscoveryFindings) -> u64 {
        findings
            .locations
            .iter()
            .filter(|loc| matches!(loc.log_type, LogType::Debug | LogType::FileHistory))
            .map(|loc| loc.size_bytes / 2) // Assume 50% compression
            .sum()
    }

    fn calculate_old_files(&self, findings: &DiscoveryFindings, _age_days: u32) -> u64 {
        // Simplified - would check actual file ages
        findings.total_size_bytes / 3 // Assume 1/3 are old
    }

    pub async fn compare_tools(&self) -> Result<ToolComparison> {
        let results = self.analyze().await?;

        Ok(ToolComparison {
            tools: results.tools,
        })
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ToolComparison {
    pub tools: HashMap<String, ToolAnalysis>,
}

impl ToolComparison {
    pub fn print(&self, format: &str) -> Result<()> {
        match format {
            "json" => {
                // Output as JSON
                let json_output: Vec<_> = self
                    .tools
                    .iter()
                    .map(|(name, analysis)| {
                        serde_json::json!({
                            "tool": name,
                            "size_bytes": analysis.total_size,
                            "size_human": format_bytes(analysis.total_size),
                            "sessions": analysis.session_count,
                            "prompts": analysis.prompt_count,
                            "avg_session_length": analysis.avg_session_length,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            }
            "csv" => {
                // Output as CSV
                println!("Tool,Size,Sessions,Prompts,AvgLength");
                for (name, analysis) in &self.tools {
                    println!(
                        "{},{},{},{},{:.1}",
                        name,
                        analysis.total_size,
                        analysis.session_count,
                        analysis.prompt_count,
                        analysis.avg_session_length
                    );
                }
            }
            _ => {
                // Default: table format
                use comfy_table::presets::UTF8_FULL;
                use comfy_table::*;

                let mut table = Table::new();
                table.load_preset(UTF8_FULL).set_header(vec![
                    "Tool",
                    "Size",
                    "Sessions",
                    "Prompts",
                    "Avg Length",
                ]);

                for (name, analysis) in &self.tools {
                    table.add_row(vec![
                        name.clone(),
                        format_bytes(analysis.total_size),
                        analysis.session_count.to_string(),
                        analysis.prompt_count.to_string(),
                        format!("{:.1}", analysis.avg_session_length),
                    ]);
                }

                println!("{table}");
            }
        }
        Ok(())
    }
}

fn estimate_sessions(location: &LogLocation) -> usize {
    match location.log_type {
        LogType::Debug => location.file_count,
        LogType::Session => location.file_count,
        _ => 0,
    }
}

fn estimate_prompts(location: &LogLocation) -> u64 {
    // Rough estimate based on file size
    location.size_bytes / (1024 * 5) // Assume 5KB per prompt
}
