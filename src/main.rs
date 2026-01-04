use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod advanced_analytics;
mod analysis;
mod backup;
mod claude_code_parser;
mod comprehensive_analyzer;
mod discovery;
mod metrics;
mod models;
mod parsers;
mod prepare;
mod report;
mod sanitizer;
mod viral_insights;
mod work_hours_analyzer;
// mod infographics;  // Temporarily disabled due to compilation errors
mod cache;
mod daemon;
mod dataset_extractor;
mod deep_insights;
mod embedded_llm;
mod extraction_utils;
mod extractors;
mod html_report;
mod llm_chat;
mod report_analyzer;
mod tui;
mod ultra_deep;

use analysis::Analyzer;
use backup::BackupManager;
use comprehensive_analyzer::ComprehensiveAnalyzer;
use discovery::LogDiscovery;
use prepare::DatasetPreparer;
use report::ReportGenerator;
use work_hours_analyzer::{generate_hours_chart, generate_tool_chart, generate_weekday_chart};
// use infographics::InfographicGenerator;  // Temporarily disabled
use dataset_extractor::DatasetExtractor;
use deep_insights::DeepAnalyzer;
use html_report::HtmlReportGenerator;
use report_analyzer::ReportAnalyzer;
use ultra_deep::UltraDeepAnalyzer;

#[derive(Parser)]
#[command(name = "vibedev")]
#[command(about = "Analyze AI coding assistant logs and generate insights", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan system for AI tool logs
    Discover {
        /// Base directory to search (default: $HOME)
        #[arg(short, long)]
        base_dir: Option<PathBuf>,

        /// Include hidden directories
        #[arg(long, default_value = "true")]
        hidden: bool,
    },

    /// Analyze discovered logs
    Analyze {
        /// Output format (text, json, html, markdown)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Analyze only specific tool
        #[arg(long)]
        tool: Option<String>,

        /// Time range in days (e.g., 30 for last 30 days)
        #[arg(long)]
        days: Option<u32>,

        /// Skip compression analysis (faster)
        #[arg(long)]
        skip_compression: bool,
    },

    /// Create backup archive of AI logs
    Backup {
        /// Output directory for backup (default: $HOME)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Specific tool to backup (default: all)
        #[arg(short, long)]
        tool: Option<String>,

        /// Compression level (0-9, default: 6)
        #[arg(short, long, default_value = "6")]
        compression: u32,

        /// Include a timestamp in filename
        #[arg(long, default_value = "true")]
        timestamp: bool,
    },

    /// Restore AI logs from backup archive
    Restore {
        /// Path to backup archive (tar.gz)
        #[arg(short, long)]
        backup: PathBuf,

        /// Output directory for restored files (default: $HOME)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Prepare sanitized dataset for finetuning
    Prepare {
        /// Output directory for dataset (default: $HOME)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show real-time statistics
    Stats {
        /// Refresh interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,
    },

    /// Compare multiple tools
    Compare {
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Generate comprehensive insights (conversations, tokens, costs, productivity)
    Insights {
        /// Output file for JSON report
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Generate PNG infographics
        #[arg(long)]
        infographics: bool,

        /// Output directory for infographics
        #[arg(long)]
        infographics_dir: Option<PathBuf>,

        /// Generate interactive HTML report with D3.js visualizations
        #[arg(long)]
        html: bool,

        /// Output path for HTML report
        #[arg(long)]
        html_output: Option<PathBuf>,
    },

    /// Extract all 37 datasets from backup
    ExtractDatasets {
        /// Path to backup ZIP file
        #[arg(short, long)]
        backup: PathBuf,

        /// Output directory for all datasets
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze extracted datasets and generate comprehensive reports
    AnalyzeDatasets {
        /// Directory containing extracted datasets (default: $HOME/ai-datasets)
        #[arg(short, long)]
        datasets_dir: Option<PathBuf>,

        /// Output directory for reports
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Deep analysis: temporal patterns, learning curves, hidden insights
    DeepAnalysis {
        /// Directory containing extracted datasets (default: $HOME/ai-datasets)
        #[arg(short, long)]
        datasets_dir: Option<PathBuf>,

        /// Output directory for reports
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// ULTRA DEEP: conversation autopsy, anti-patterns, productivity killers
    UltraDeep {
        /// Directory containing extracted datasets (default: $HOME/ai-datasets)
        #[arg(short, long)]
        datasets_dir: Option<PathBuf>,

        /// Output directory for reports
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Interactive TUI mode (like dust) - multi-threaded visualization
    Tui {
        /// Base directory to scan (default: $HOME)
        #[arg(short, long)]
        base_dir: Option<PathBuf>,

        /// Output to CLI instead of interactive TUI
        #[arg(long)]
        cli: bool,
    },

    /// Chat with your data using embedded offline LLM
    Chat {
        /// Model to use (run 'vibedev models' to see available)
        #[arg(short, long)]
        model: Option<String>,

        /// Single question (non-interactive)
        #[arg(short, long)]
        query: Option<String>,

        /// Run automatic analysis
        #[arg(long)]
        analyze: bool,

        /// Get recommendations for a specific topic (storage, patterns, optimize, cleanup)
        #[arg(long)]
        topic: Option<String>,

        /// List available analysis topics
        #[arg(long)]
        list_topics: bool,

        /// Load existing analysis data/datasets from path
        #[arg(long)]
        with_data: Option<PathBuf>,

        /// Device to use for inference (auto, cpu, cuda, metal)
        #[arg(long, default_value = "auto")]
        device: String,

        /// Quantization level (f32, f16, bf16)
        #[arg(long, default_value = "auto")]
        precision: String,
    },

    /// Manage offline LLM models (download, switch, remove)
    Models {
        /// Action: list, download, use, remove
        #[arg(default_value = "list")]
        action: String,

        /// Model ID for download/use/remove actions
        model_id: Option<String>,
    },

    /// Manage LLM daemon (keeps model loaded for fast queries)
    Daemon {
        /// Action: start, stop, status, restart
        #[arg(default_value = "status")]
        action: String,

        /// Model to load (only for start)
        #[arg(short, long)]
        model: Option<String>,

        /// Device (auto, cpu, cuda, metal)
        #[arg(long, default_value = "auto")]
        device: String,

        /// Precision (auto, f32, f16, bf16)
        #[arg(long, default_value = "auto")]
        precision: String,
    },
}

/// Load analysis data from a directory (JSON files, reports, datasets)
fn load_analysis_data(path: &PathBuf) -> Result<String> {
    use std::fs;

    let mut context = String::new();
    context.push_str("=== Loaded Analysis Data ===\n\n");

    // Look for common analysis files
    let files_to_check = [
        "ai-insights-report.json",
        "analysis_report.json",
        "deep_insights.json",
        "ultra_deep_insights.json",
        "MANIFEST.json",
        "ANALYSIS_REPORT.md",
    ];

    let mut found_any = false;

    for filename in &files_to_check {
        let file_path = path.join(filename);
        if file_path.exists() {
            found_any = true;
            context.push_str(&format!("--- {} ---\n", filename));

            let content = fs::read_to_string(&file_path)?;

            // Truncate large files to fit in context
            if content.len() > 8000 {
                context.push_str(&content[..8000]);
                context.push_str("\n... [truncated]\n");
            } else {
                context.push_str(&content);
            }
            context.push_str("\n\n");
        }
    }

    // Also check for dataset files
    let dataset_files = [
        "conversations.jsonl",
        "bug_patterns.jsonl",
        "prompt_patterns.jsonl",
        "tool_sequences.jsonl",
        "productivity_metrics.jsonl",
    ];

    for filename in &dataset_files {
        let file_path = path.join(filename);
        if file_path.exists() {
            found_any = true;
            context.push_str(&format!("--- {} (sample) ---\n", filename));

            let content = fs::read_to_string(&file_path)?;
            // Take first 20 lines as sample
            let sample: String = content.lines().take(20).collect::<Vec<_>>().join("\n");
            context.push_str(&sample);
            context.push_str("\n... [more entries]\n\n");
        }
    }

    if !found_any {
        // Try to list what's in the directory
        context.push_str("Files found:\n");
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.take(20).flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    context.push_str(&format!("  - {}\n", name));
                }
            }
        }
    }

    Ok(context)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    match cli.command {
        Commands::Discover { base_dir, hidden } => {
            info!("🔍 Discovering AI tool logs...");
            let base = base_dir
                .unwrap_or_else(|| dirs::home_dir().expect("Could not determine home directory"));

            let discovery = LogDiscovery::new(base, hidden);
            let findings = discovery.scan()?;

            println!("\n📊 Discovery Results:\n");
            findings.print_summary();

            Ok(())
        }

        Commands::Analyze {
            format,
            output,
            tool,
            days,
            skip_compression,
        } => {
            info!("📈 Analyzing logs...");

            let analyzer = Analyzer::new()
                .with_tool_filter(tool)
                .with_time_range(days)
                .with_compression_check(!skip_compression);

            let results = analyzer.analyze().await?;

            let generator = ReportGenerator::new(format.as_str());
            let report = generator.generate(&results)?;

            if let Some(out_path) = output {
                generator.write_to_file(&report, &out_path)?;
                println!("✅ Report saved to: {}", out_path.display());
            } else {
                println!("{}", report);
            }

            Ok(())
        }

        Commands::Backup {
            output,
            tool,
            compression,
            timestamp,
        } => {
            info!("💾 Creating backup archive...");

            let output_dir = output
                .unwrap_or_else(|| dirs::home_dir().expect("Could not determine home directory"));

            let manager = BackupManager::new(output_dir, compression);
            let archive_path = manager.create_backup(tool, timestamp).await?;

            println!("✅ Backup created: {}", archive_path.display());

            Ok(())
        }

        Commands::Restore { backup, output } => {
            info!("📦 Restoring from backup archive...");

            backup::restore_backup(&backup, output)?;

            Ok(())
        }

        Commands::Prepare { output } => {
            info!("🔧 Preparing sanitized dataset for finetuning...");

            let output_dir = output
                .unwrap_or_else(|| dirs::home_dir().expect("Could not determine home directory"));

            let preparer = DatasetPreparer::new(output_dir);
            let _results = preparer.prepare_dataset().await?;

            Ok(())
        }

        Commands::Stats { interval } => {
            info!("📊 Starting real-time statistics monitor...");
            metrics::live_stats(interval).await?;
            Ok(())
        }

        Commands::Compare { format } => {
            info!("⚖️  Comparing AI tools...");

            let analyzer = Analyzer::new();
            let comparison = analyzer.compare_tools().await?;

            comparison.print(format.as_str())?;

            Ok(())
        }

        Commands::Insights {
            output,
            infographics,
            infographics_dir: _,
            html,
            html_output,
        } => {
            info!("🔍 Generating comprehensive insights from 52+ GB data...");

            let base_dir = dirs::home_dir().expect("Could not determine home directory");
            let analyzer = ComprehensiveAnalyzer::new(base_dir);

            let insights = analyzer.analyze()?;

            // Save JSON report
            let json = serde_json::to_string_pretty(&insights)?;
            let output_path = output.unwrap_or_else(|| {
                let home = dirs::home_dir().expect("Could not determine home directory");
                home.join("ai-insights-report.json")
            });

            fs::write(&output_path, &json)?;

            println!("\n✅ Comprehensive Insights Report\n");
            println!("📊 Conversations:");
            println!("  Total: {}", insights.conversations.total_conversations);
            println!("  Messages: {}", insights.conversations.total_messages);
            println!(
                "  User: {} | Assistant: {}",
                insights.conversations.user_messages, insights.conversations.assistant_messages
            );
            println!(
                "  Avg length: {:.1} messages/conversation",
                insights.conversations.average_conversation_length
            );

            println!("\n💰 Tokens:");
            println!("  Input: {}", insights.token_usage.total_input_tokens);
            println!("  Output: {}", insights.token_usage.total_output_tokens);
            println!("  Total: {}", insights.token_usage.total_tokens);

            println!("\n💵 Cost Analysis:");
            println!("  Total: ${:.2}", insights.cost_analysis.total_cost_usd);
            println!(
                "  Monthly estimate: ${:.2}",
                insights.cost_analysis.monthly_estimate
            );
            println!(
                "  Potential savings: ${:.2}",
                insights.cost_analysis.potential_savings
            );

            println!("\n⏱️  Work Hours:");
            println!(
                "  Total hours: {:.1}h ({:.0} days)",
                insights.work_hours.total_hours,
                insights.work_hours.total_hours / 8.0
            );
            println!("  Total sessions: {}", insights.work_hours.total_sessions);
            println!(
                "  Average session: {:.1}h",
                insights.work_hours.average_session_hours
            );
            println!(
                "  Daily average: {:.1}h/day",
                insights.work_hours.daily_average
            );
            println!(
                "  Weekly average: {:.1}h/week",
                insights.work_hours.weekly_average
            );
            println!("  Busiest day: {}", insights.work_hours.busiest_day);
            println!("  Busiest hour: {:02}:00", insights.work_hours.busiest_hour);
            println!(
                "  Work-life balance: {:.0}/100",
                insights.work_hours.work_life_balance_score
            );

            // Print charts
            println!(
                "{}",
                generate_hours_chart(&insights.work_hours.hours_by_hour_of_day)
            );
            println!(
                "{}",
                generate_weekday_chart(&insights.work_hours.hours_by_weekday)
            );
            println!(
                "{}",
                generate_tool_chart(&insights.work_hours.hours_by_tool)
            );

            println!("\n📁 Full report saved: {}", output_path.display());

            if infographics {
                println!("\n⚠️  PNG infographics temporarily disabled (compilation errors)");
                println!("   Use --html flag for interactive HTML visualizations instead!");
            }

            if html {
                println!("\n🌐 Generating interactive HTML report with D3.js...");
                let html_path = html_output.unwrap_or_else(|| {
                    dirs::home_dir()
                        .expect("Could not determine home directory")
                        .join("ai-insights-dashboard.html")
                });

                let html_gen = HtmlReportGenerator::new(html_path.clone());
                html_gen.generate(&insights)?;

                println!("\n✅ Interactive HTML report generated!");
                println!("📁 Open in browser: file://{}", html_path.display());

                // Try to open in default browser
                if open::that(&html_path).is_err() {
                    println!("   (Could not auto-open browser - please open manually)");
                }
            }

            Ok(())
        }

        Commands::ExtractDatasets { backup, output } => {
            info!("🚀 Extracting 37 datasets from backup...");

            // First, run comprehensive analysis to get insights
            let base_dir = dirs::home_dir().expect("Could not determine home directory");
            let analyzer = ComprehensiveAnalyzer::new(base_dir);
            let insights = analyzer.analyze()?;

            let output_dir = output.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Could not determine home directory")
                    .join("ai-datasets")
            });

            let extractor = DatasetExtractor::new(backup, output_dir.clone(), insights);
            let manifest = extractor.extract_all()?;

            println!("\n📊 Dataset Extraction Summary:");
            println!("   Total datasets: {}", manifest.total_datasets);
            println!("   Time taken: {}s", manifest.extraction_time_seconds);
            println!("   Output size: {:.2} GB", manifest.total_output_size_gb);
            println!("\n📁 All datasets saved to: {}", output_dir.display());
            println!(
                "📄 Manifest: {}",
                output_dir.join("MANIFEST.json").display()
            );

            Ok(())
        }

        Commands::AnalyzeDatasets {
            datasets_dir,
            output,
        } => {
            info!("📊 Analyzing extracted datasets...");

            let datasets_path = datasets_dir.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Could not determine home directory")
                    .join("ai-datasets")
            });

            let output_path = output.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Could not determine home directory")
                    .join("ai-datasets")
            });

            let analyzer = ReportAnalyzer::new(datasets_path);
            let report = analyzer.generate_comprehensive_report()?;

            analyzer.save_report(&report, &output_path)?;

            // Print summary
            println!("\n📊 Dataset Analysis Complete!\n");

            println!("🐛 Bug Patterns:");
            println!("   Total patterns: {}", report.bug_analysis.total_patterns);
            println!(
                "   Time wasted: {:.1} hours ({:.0} work days)",
                report.bug_analysis.total_time_wasted_hours,
                report.bug_analysis.total_time_wasted_hours / 8.0
            );
            println!("   Cost: ${:.2}\n", report.bug_analysis.total_cost_wasted);

            println!("💬 Prompt Engineering:");
            println!("   Total prompts: {}", report.prompt_analysis.total_prompts);
            for (spec, rate) in &report.prompt_analysis.success_rate_by_specificity {
                println!("   {} specificity: {:.1}% success", spec, rate);
            }

            println!("\n🎯 Top Recommendations:\n");
            for (idx, rec) in report.recommendations.iter().take(3).enumerate() {
                println!("{}. [{}] {}", idx + 1, rec.priority, rec.title);
                println!("   {}", rec.description);
                println!("   Savings: {}\n", rec.potential_savings);
            }

            println!("📁 Full reports saved to:");
            println!(
                "   JSON: {}",
                output_path.join("analysis_report.json").display()
            );
            println!(
                "   Markdown: {}",
                output_path.join("ANALYSIS_REPORT.md").display()
            );

            Ok(())
        }

        Commands::DeepAnalysis {
            datasets_dir,
            output,
        } => {
            info!("🔬 Running deep analysis...");

            let datasets_path = datasets_dir.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Could not determine home directory")
                    .join("ai-datasets")
            });

            let output_path = output.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Could not determine home directory")
                    .join("ai-datasets")
            });

            let analyzer = DeepAnalyzer::new(datasets_path);
            let insights = analyzer.analyze()?;

            // Save JSON
            let json = serde_json::to_string_pretty(&insights)?;
            fs::write(output_path.join("deep_insights.json"), &json)?;

            // Print key findings
            println!("\n🔬 Deep Insights Report\n");

            println!("⏰ Temporal Patterns:");
            println!(
                "   Peak performance windows: {}",
                insights.temporal_patterns.peak_performance_windows.len()
            );
            for window in insights
                .temporal_patterns
                .peak_performance_windows
                .iter()
                .take(3)
            {
                println!(
                    "   - {}:00-{}:00 (efficiency: {:.2}, {} tasks)",
                    window.hour_start,
                    window.hour_end,
                    window.efficiency_score,
                    window.tasks_completed
                );
            }

            println!("\n🧠 Conversation Intelligence:");
            println!(
                "   Successful patterns identified: {}",
                insights.conversation_intelligence.successful_patterns.len()
            );
            for pattern in insights
                .conversation_intelligence
                .successful_patterns
                .iter()
                .take(3)
            {
                println!(
                    "   - {} ({} examples, {:.0}% success)",
                    pattern.pattern_name, pattern.examples, pattern.success_rate
                );
            }

            println!("\n📈 Task Complexity:");
            for outcome in &insights.task_complexity_analysis.complexity_vs_outcome {
                println!(
                    "   {} tasks: {}% success rate ({} attempts, {:.1}h avg)",
                    outcome.complexity,
                    outcome.success_rate as i32,
                    outcome.attempts,
                    outcome.avg_time_hours
                );
            }

            println!(
                "\n🔍 Hidden Patterns Discovered: {}",
                insights.hidden_patterns.len()
            );
            for (idx, pattern) in insights.hidden_patterns.iter().enumerate() {
                println!(
                    "\n{}. [{}] {}",
                    idx + 1,
                    pattern.significance,
                    pattern.title
                );
                println!("   {}", pattern.description);
                println!("   💡 {}", pattern.actionable_insight);
            }

            println!(
                "\n📁 Full analysis: {}",
                output_path.join("deep_insights.json").display()
            );

            Ok(())
        }

        Commands::UltraDeep {
            datasets_dir,
            output,
        } => {
            info!("🔬 ULTRA DEEP ANALYSIS...");

            let datasets_path = datasets_dir.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Could not determine home directory")
                    .join("ai-datasets")
            });

            let output_path = output.unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Could not determine home directory")
                    .join("ai-datasets")
            });

            let analyzer = UltraDeepAnalyzer::new(datasets_path);
            let insights = analyzer.analyze()?;

            // Save JSON
            let json = serde_json::to_string_pretty(&insights)?;
            fs::write(output_path.join("ultra_deep_insights.json"), &json)?;

            // Print findings
            println!("\n💀 ULTRA DEEP INSIGHTS - THE BRUTAL TRUTH\n");
            println!("═══════════════════════════════════════════════\n");

            println!("🧬 CONVERSATION AUTOPSY:");
            println!(
                "   Death spirals detected: {}",
                insights.conversation_autopsy.death_spirals.len()
            );
            println!(
                "   Zombie conversations: {} (long but unproductive)",
                insights.conversation_autopsy.zombie_conversations
            );
            println!(
                "   Abandonment rate: {:.1}%",
                insights.conversation_autopsy.abandonment_rate
            );

            if !insights.conversation_autopsy.death_spirals.is_empty() {
                let worst = &insights.conversation_autopsy.death_spirals[0];
                println!("\n   ☠️  Worst Death Spiral:");
                println!(
                    "      {} turns wasted repeating same errors",
                    worst.total_turns
                );
                println!("      Cost: {:.1} hours", worst.wasted_hours);
                println!("      Escape: {}", worst.escape_route);
            }

            println!("\n🚫 ANTI-PATTERNS IDENTIFIED:");
            for (idx, ap) in insights.anti_patterns.iter().enumerate() {
                println!(
                    "\n   {}. {} [{:.0} hours wasted!]",
                    idx + 1,
                    ap.name,
                    ap.total_cost_hours
                );
                println!("      {}", ap.description);
                println!("      How to avoid:");
                for tactic in &ap.how_to_avoid {
                    println!("      • {}", tactic);
                }
            }

            println!("\n☠️  PRODUCTIVITY KILLERS:");
            for killer in &insights.productivity_killers {
                println!(
                    "\n   🔴 {} [{}]",
                    killer.killer_name,
                    killer.severity.to_uppercase()
                );
                println!("      {}", killer.description);
                println!("      Cost: {:.0} hours wasted", killer.hours_wasted);
                println!("      Prevention:");
                for tactic in killer.prevention_tactics.iter().take(2) {
                    println!("      • {}", tactic);
                }
            }

            println!("\n✨ SUCCESS BLUEPRINTS:");
            for bp in &insights.success_blueprints {
                println!(
                    "\n   📋 {} ({:.0}% success rate)",
                    bp.blueprint_name, bp.success_rate
                );
                println!("      Steps:");
                for step in &bp.step_by_step {
                    println!("      {}", step);
                }
            }

            println!("\n🛠️  TOOL SEQUENCE MASTERY:");
            if !insights.tool_sequence_mastery.winning_sequences.is_empty() {
                println!("   ✅ Winning Sequences:");
                for seq in insights
                    .tool_sequence_mastery
                    .winning_sequences
                    .iter()
                    .take(3)
                {
                    println!(
                        "      {} → {:.0}% success",
                        seq.sequence.join(" → "),
                        seq.success_rate
                    );
                }
            }
            if !insights.tool_sequence_mastery.losing_sequences.is_empty() {
                println!("\n   ❌ Losing Sequences:");
                for seq in insights
                    .tool_sequence_mastery
                    .losing_sequences
                    .iter()
                    .take(3)
                {
                    println!(
                        "      {} → {:.0}% failure",
                        seq.sequence.join(" → "),
                        seq.failure_rate
                    );
                    println!("         Better: {}", seq.better_alternative);
                }
            }

            println!("\n🔥 BURNOUT DETECTION:");
            println!(
                "   Burnout sessions found: {}",
                insights.burnout_detection.burnout_sessions.len()
            );
            println!(
                "   Optimal session length: {:.0} minutes",
                insights.burnout_detection.optimal_session_length
            );
            println!(
                "   Marathon sessions without breaks: {}",
                insights
                    .session_dynamics
                    .breaks_analysis
                    .sessions_without_breaks
            );
            println!(
                "   Cost of no breaks: {:.0} hours",
                insights
                    .session_dynamics
                    .breaks_analysis
                    .cost_of_no_breaks_hours
            );

            println!("\n🔄 RECOVERY STRATEGIES:");
            for strategy in &insights.recovery_strategies {
                println!("\n   When: {}", strategy.stuck_scenario);
                println!("   Do this:");
                for step in &strategy.recovery_steps {
                    println!("      {}", step);
                }
            }

            println!("\n═══════════════════════════════════════════════");
            println!(
                "\n📁 Full report: {}",
                output_path.join("ultra_deep_insights.json").display()
            );

            Ok(())
        }

        Commands::Tui { base_dir, cli } => {
            let base = base_dir
                .unwrap_or_else(|| dirs::home_dir().expect("Could not determine home directory"));

            if cli {
                tui::print_cli_output(base)?;
            } else {
                tui::run_tui(base)?;
            }

            Ok(())
        }

        Commands::Chat {
            model,
            query,
            analyze,
            topic,
            list_topics,
            with_data,
            device,
            precision,
        } => {
            use colored::Colorize;
            use embedded_llm::{DeviceType, Quantization};

            // List available topics
            if list_topics {
                println!("{}", "Available Analysis Topics:".cyan().bold());
                println!();
                for (name, description) in llm_chat::ANALYSIS_PROMPTS {
                    println!("  {} - {}", name.green(), description);
                }
                println!();
                println!("Usage: vibedev chat --topic <name>");
                return Ok(());
            }

            // Check if daemon is running - use it for fast queries
            if daemon::is_running() {
                println!("{}", "Using daemon (model already loaded)".green());

                // Build context
                let context = if let Some(data_path) = &with_data {
                    println!("{} {}", "Loading data from:".cyan(), data_path.display());
                    Some(load_analysis_data(data_path)?)
                } else {
                    None
                };

                if analyze {
                    let prompt = "Analyze this AI tool usage data. What are the top 3 actionable recommendations?";
                    match daemon::query(prompt, context.as_deref()) {
                        Ok(response) => println!("\n{}", response),
                        Err(e) => println!("{}: {}", "Error".red(), e),
                    }
                } else if let Some(t) = &topic {
                    let prompt = format!("Give me specific recommendations for: {}", t);
                    match daemon::query(&prompt, context.as_deref()) {
                        Ok(response) => println!("\n{}", response),
                        Err(e) => println!("{}: {}", "Error".red(), e),
                    }
                } else if let Some(q) = &query {
                    match daemon::query(q, context.as_deref()) {
                        Ok(response) => println!("\n{}", response),
                        Err(e) => println!("{}: {}", "Error".red(), e),
                    }
                } else {
                    println!("\nDaemon ready. Use --query, --analyze, or --topic.");
                    println!("Example: vibedev chat --query \"What should I optimize?\"");
                }

                return Ok(());
            }

            // Parse device type
            let device_type = match device.to_lowercase().as_str() {
                "auto" => None, // Let it auto-detect
                "cpu" => Some(DeviceType::Cpu),
                "cuda" | "gpu" => Some(DeviceType::Cuda(0)),
                "metal" => Some(DeviceType::Metal),
                _ => {
                    println!(
                        "{}: Unknown device '{}'. Use: auto, cpu, cuda, metal",
                        "Error".red(),
                        device
                    );
                    return Ok(());
                }
            };

            // Parse precision/quantization
            let quantization = match precision.to_lowercase().as_str() {
                "auto" => None, // Let it auto-select based on device
                "f32" | "fp32" => Some(Quantization::F32),
                "f16" | "fp16" => Some(Quantization::F16),
                "bf16" => Some(Quantization::BF16),
                _ => {
                    println!(
                        "{}: Unknown precision '{}'. Use: auto, f32, f16, bf16",
                        "Error".red(),
                        precision
                    );
                    return Ok(());
                }
            };

            // Setup LLM chat (embedded, offline) with device/quantization options
            let mut chat = llm_chat::LlmChat::new_with_options(model, device_type, quantization);

            // Check if any model is downloaded
            if !chat.has_model() {
                println!("{}", "No LLM model downloaded!".red().bold());
                println!("\nTo get started, download a model:");
                println!("  vibedev models download qwen-coder-1.5b  # Recommended (~3GB)");
                println!("  vibedev models download qwen-coder-0.5b  # Smaller (~1GB)");
                println!("\nRun 'vibedev models' to see all available models.");
                return Ok(());
            }

            // Build context from data source
            let context = if let Some(data_path) = with_data {
                // Load existing analysis data
                println!("{} {}", "Loading data from:".cyan(), data_path.display());
                load_analysis_data(&data_path)?
            } else {
                // Scan for live data
                let base_dir = dirs::home_dir().expect("Could not determine home directory");
                println!("{}", "Scanning AI tool logs...".cyan());
                let discovery = LogDiscovery::new(base_dir, true);
                let findings = discovery.scan()?;

                let mut tool_sizes = std::collections::HashMap::new();
                for loc in &findings.locations {
                    *tool_sizes
                        .entry(loc.tool.name().to_string())
                        .or_insert(0u64) += loc.size_bytes;
                }

                llm_chat::LlmChat::generate_context(
                    &tool_sizes,
                    findings.total_size_bytes,
                    findings.total_files,
                    findings.locations.len(),
                )
            };

            chat.set_context(&context);
            println!("{}", chat.backend_name().cyan());

            if analyze {
                // Run automatic analysis
                println!("\n{}\n", "Running AI Analysis...".cyan().bold());
                match chat.analyze().await {
                    Ok(response) => println!("{}", response),
                    Err(e) => println!("{}: {}", "Error".red(), e),
                }
            } else if let Some(t) = topic {
                // Topic-specific recommendations
                println!(
                    "\n{}: {}\n",
                    "Getting recommendations for".cyan(),
                    t.yellow()
                );
                match chat.get_recommendations(&t).await {
                    Ok(response) => println!("{}", response),
                    Err(e) => println!("{}: {}", "Error".red(), e),
                }
            } else if let Some(q) = query {
                // Single question mode
                match chat.chat(&q).await {
                    Ok(response) => println!("\n{}", response),
                    Err(e) => println!("{}: {}", "Error".red(), e),
                }
            } else {
                // Interactive chat mode
                println!("\n{}", "vibedev AI Chat (offline)".cyan().bold());
                println!("Type your questions about your AI tool usage. Type 'quit' to exit.");
                println!("Tip: Type 'topics' to see available analysis topics.\n");

                let stdin = std::io::stdin();
                let mut input = String::new();

                loop {
                    print!("{} ", "You:".green().bold());
                    use std::io::Write;
                    std::io::stdout().flush()?;

                    input.clear();
                    stdin.read_line(&mut input)?;
                    let trimmed = input.trim();

                    if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit")
                    {
                        break;
                    }

                    if trimmed.eq_ignore_ascii_case("topics") {
                        println!("\n{}", "Available Analysis Topics:".cyan());
                        for (name, desc) in llm_chat::ANALYSIS_PROMPTS {
                            println!("  {} - {}", name.green(), desc);
                        }
                        println!();
                        continue;
                    }

                    if trimmed.is_empty() {
                        continue;
                    }

                    print!("{} ", "AI:".cyan().bold());
                    std::io::stdout().flush()?;

                    match chat.chat(trimmed).await {
                        Ok(response) => println!("{}\n", response),
                        Err(e) => println!("{}: {}\n", "Error".red(), e),
                    }
                }
            }

            Ok(())
        }

        Commands::Models { action, model_id } => {
            use colored::Colorize;

            match action.as_str() {
                "list" | "ls" => {
                    embedded_llm::list_models();
                }

                "download" | "dl" | "get" => {
                    let id = model_id.ok_or_else(|| {
                        anyhow::anyhow!("Please specify a model ID. Run 'vibedev models' to see available models.")
                    })?;

                    match embedded_llm::download_model(&id) {
                        Ok(_) => {
                            println!(
                                "\n{} Model '{}' downloaded successfully!",
                                "Success:".green().bold(),
                                id
                            );
                            println!("Run 'vibedev models use {}' to activate it.", id);
                        }
                        Err(e) => {
                            println!("{}: {}", "Error".red().bold(), e);
                        }
                    }
                }

                "use" | "switch" | "set" => {
                    let id = model_id.ok_or_else(|| {
                        anyhow::anyhow!("Please specify a model ID. Run 'vibedev models' to see downloaded models.")
                    })?;

                    let downloaded = embedded_llm::get_downloaded_models();
                    if !downloaded.contains(&id) {
                        println!("{}: Model '{}' not downloaded.", "Error".red().bold(), id);
                        println!("Run 'vibedev models download {}' first.", id);
                        return Ok(());
                    }

                    embedded_llm::set_current_model(&id)?;
                    println!("{} Now using model '{}'", "Success:".green().bold(), id);
                }

                "remove" | "rm" | "delete" => {
                    let id = model_id
                        .ok_or_else(|| anyhow::anyhow!("Please specify a model ID to remove."))?;

                    let model_dir = embedded_llm::get_models_dir().join(&id);
                    if model_dir.exists() {
                        std::fs::remove_dir_all(&model_dir)?;
                        println!("{} Model '{}' removed.", "Success:".green().bold(), id);

                        // Clear current model if it was the removed one
                        if embedded_llm::get_current_model().as_deref() == Some(id.as_str()) {
                            let config = embedded_llm::get_config_dir().join("current_model");
                            let _ = std::fs::remove_file(config);
                        }
                    } else {
                        println!("{}: Model '{}' not found.", "Error".red().bold(), id);
                    }
                }

                "info" => {
                    if let Some(id) = model_id {
                        if let Some(info) = embedded_llm::get_model_info(&id) {
                            println!("\nModel: {}", info.name.cyan().bold());
                            println!("  ID: {}", info.id);
                            println!("  Parameters: {}", info.params);
                            println!("  Size: ~{}GB", info.size_gb);
                            println!("  HuggingFace: {}", info.hf_repo);
                            println!("  Description: {}", info.description);
                        } else {
                            println!("{}: Unknown model '{}'", "Error".red().bold(), id);
                        }
                    } else {
                        println!("Please specify a model ID. Run 'vibedev models' to see available models.");
                    }
                }

                "device" | "gpu" => {
                    let device = embedded_llm::detect_device();
                    println!("\n{}", "Device Detection:".cyan().bold());
                    println!("  Detected: {:?}", device);

                    #[cfg(feature = "cuda")]
                    println!("  CUDA: {} (feature enabled)", "Available".green());
                    #[cfg(not(feature = "cuda"))]
                    println!(
                        "  CUDA: {} (rebuild with --features cuda)",
                        "Not enabled".yellow()
                    );

                    #[cfg(feature = "metal")]
                    println!("  Metal: {} (feature enabled)", "Available".green());
                    #[cfg(not(feature = "metal"))]
                    println!(
                        "  Metal: {} (rebuild with --features metal)",
                        "Not enabled".yellow()
                    );

                    println!("\n{}", "Quantization Options:".cyan().bold());
                    println!("  F32  - Full precision (default for CPU)");
                    println!("  F16  - Half precision (recommended for GPU)");
                    println!("  BF16 - BFloat16 (GPU only)");

                    println!("\n{}", "Usage:".cyan().bold());
                    println!("  vibedev chat --device cuda --precision f16  # GPU with F16");
                    println!("  vibedev chat --device cpu --precision f32   # CPU with F32");
                }

                _ => {
                    println!("{}: Unknown action '{}'\n", "Error".red().bold(), action);
                    println!("Available actions:");
                    println!("  list              - Show all available models");
                    println!("  download <id>     - Download a model");
                    println!("  use <id>          - Switch to a downloaded model");
                    println!("  remove <id>       - Remove a downloaded model");
                    println!("  info <id>         - Show model details");
                    println!("  device            - Show GPU/device detection info");
                }
            }

            Ok(())
        }

        Commands::Daemon {
            action,
            model,
            device,
            precision,
        } => {
            use colored::Colorize;
            use embedded_llm::{DeviceType, Quantization};

            match action.as_str() {
                "status" => {
                    let info = daemon::info();
                    println!("\n{}", "Daemon Status:".cyan().bold());
                    if info.running {
                        println!("  Status: {}", "Running".green());
                        if let Some(m) = info.model {
                            println!("  Model: {}", m);
                        }
                        if let Some(pid) = info.pid {
                            println!("  PID: {}", pid);
                        }
                        println!("  Socket: {}", info.socket.display());
                    } else {
                        println!("  Status: {}", "Not running".yellow());
                        println!("\nStart with: vibedev daemon start");
                    }
                }

                "start" => {
                    if daemon::is_running() {
                        println!(
                            "{}: Daemon already running. Use 'vibedev daemon restart' to restart.",
                            "Error".red().bold()
                        );
                        return Ok(());
                    }

                    // Parse device
                    let device_type = match device.to_lowercase().as_str() {
                        "auto" => None,
                        "cpu" => Some(DeviceType::Cpu),
                        "cuda" | "gpu" => Some(DeviceType::Cuda(0)),
                        "metal" => Some(DeviceType::Metal),
                        _ => {
                            println!("{}: Unknown device '{}'", "Error".red(), device);
                            return Ok(());
                        }
                    };

                    // Parse precision
                    let quantization = match precision.to_lowercase().as_str() {
                        "auto" => None,
                        "f32" | "fp32" => Some(Quantization::F32),
                        "f16" | "fp16" => Some(Quantization::F16),
                        "bf16" => Some(Quantization::BF16),
                        _ => {
                            println!("{}: Unknown precision '{}'", "Error".red(), precision);
                            return Ok(());
                        }
                    };

                    daemon::start(model.as_deref(), device_type, quantization)?;
                }

                "stop" => {
                    if !daemon::is_running() {
                        println!("Daemon is not running.");
                        return Ok(());
                    }

                    daemon::stop()?;
                    println!("{} Daemon stopped.", "Success:".green().bold());
                }

                "restart" => {
                    if daemon::is_running() {
                        println!("Stopping daemon...");
                        daemon::stop()?;
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }

                    // Parse device
                    let device_type = match device.to_lowercase().as_str() {
                        "auto" => None,
                        "cpu" => Some(DeviceType::Cpu),
                        "cuda" | "gpu" => Some(DeviceType::Cuda(0)),
                        "metal" => Some(DeviceType::Metal),
                        _ => None,
                    };

                    let quantization = match precision.to_lowercase().as_str() {
                        "auto" => None,
                        "f32" | "fp32" => Some(Quantization::F32),
                        "f16" | "fp16" => Some(Quantization::F16),
                        "bf16" => Some(Quantization::BF16),
                        _ => None,
                    };

                    daemon::start(model.as_deref(), device_type, quantization)?;
                }

                _ => {
                    println!("{}: Unknown action '{}'\n", "Error".red().bold(), action);
                    println!("Available actions:");
                    println!("  status   - Check daemon status");
                    println!("  start    - Start the daemon");
                    println!("  stop     - Stop the daemon");
                    println!("  restart  - Restart the daemon");
                }
            }

            Ok(())
        }
    }
}
