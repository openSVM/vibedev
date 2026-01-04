use crate::models::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Type alias for date range tuple
type DateRange = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

pub struct LogDiscovery {
    base_dir: PathBuf,
    include_hidden: bool,
}

impl LogDiscovery {
    pub fn new(base_dir: PathBuf, include_hidden: bool) -> Self {
        Self {
            base_dir,
            include_hidden,
        }
    }

    pub fn scan(&self) -> Result<DiscoveryFindings> {
        info!("Scanning from: {}", self.base_dir.display());

        let locations = Vec::new();
        let tools_found = HashSet::new();

        // Known AI tool directories
        let search_patterns = vec![
            // Claude Code
            ".claude",
            "Library/Application Support/Claude",
            "AppData/Roaming/Claude",
            // Cursor (main directories)
            ".cursor",
            ".cursor/extensions",  // 1.2 GB of extension data
            "Library/Application Support/Cursor",
            "AppData/Roaming/Cursor",
            // VSCode extension data (CRITICAL - where most logs are!)
            ".config/Code/User/globalStorage/saoudrizwan.claude-dev",          // Cline
            ".config/Code/User/globalStorage/rooveterinaryinc.roo-cline",      // Roo-Cline
            ".config/Code/User/globalStorage/github.copilot-chat",              // Copilot
            ".config/Code/User/globalStorage/github.copilot",                   // Copilot
            ".config/Code/User/globalStorage/continue.continue",                // Continue.dev
            ".config/Code/User/globalStorage/sourcegraph.cody-ai",              // Cody
            ".config/Code/User/globalStorage/kilocode.kilo-code",               // Kilo
            // Cursor extension data
            ".config/Cursor/User/globalStorage/rooveterinaryinc.roo-cline",
            ".config/Cursor/User/globalStorage/saoudrizwan.claude-dev",
            ".config/Cursor/User/globalStorage/github.copilot-chat",            // Cursor Copilot
            ".config/Cursor/User/globalStorage/kilocode.kilo-code",             // Cursor Kilo
            ".config/Cursor/User/globalStorage",                                 // Cursor state DBs
            // Kiro extension data
            ".config/Kiro/User/globalStorage/rooveterinaryinc.roo-cline",
            ".config/Kiro/User/globalStorage/saoudrizwan.claude-dev",
            ".kiro/extensions",
            // Flatpak VSCode (can have 40+ GB of data!)
            ".var/app/com.visualstudio.code/config/Code/User/globalStorage/saoudrizwan.claude-dev",
            ".var/app/com.visualstudio.code/config/Code/User/globalStorage/rooveterinaryinc.roo-cline",
            ".var/app/com.visualstudio.code/config/Code/User/globalStorage/github.copilot-chat",
            ".var/app/com.visualstudio.code/config/Code/User/globalStorage/kilocode.kilo-code",
            ".var/app/com.visualstudio.code/config/Code/User/globalStorage",
            ".var/app/com.visualstudio.code/config/Code/logs",
            // Flatpak Cursor
            ".var/app/com.cursor.Cursor/config/Cursor/User/globalStorage",
            // Flatpak Android Studio / JetBrains
            ".var/app/com.google.AndroidStudio",
            ".var/app/com.jetbrains.IntelliJ-IDEA-Community",
            ".var/app/com.jetbrains.PyCharm-Community",
            // Cline
            ".config/cline",
            "Library/Application Support/Cline",
            "AppData/Roaming/Cline",
            // Kiro
            ".config/Kiro",
            ".kiro",
            // Roo Code
            ".config/roo-code",
            ".roocode",
            "Library/Application Support/Roo",
            // Kilo
            ".config/kilo",
            ".kilo",
            // VSCode & extensions
            ".vscode",
            ".vscode-server",
            ".var/app/com.visualstudio.code",
            // Editor logs (contain extension runtime logs)
            ".config/Code/logs",
            ".config/Cursor/logs",
            ".config/github-copilot",
            // Windsurf
            ".windsurf",
            ".config/windsurf",
            "Library/Application Support/Windsurf",
            // Continue.dev
            ".continue",
            ".config/continue",
            // Aider
            ".aider",
            "Library/Caches/aider",
            // Cody (Sourcegraph)
            ".cody",
            ".config/cody",
            "Library/Application Support/Cody",
            // Tabnine
            ".tabnine",
            "Library/Application Support/Tabnine",
            // Amazon Q / CodeWhisperer
            ".aws/codewhisperer",
            ".config/amazonq",
            // CodeGPT
            ".codegpt",
            // Bito
            ".bito",
            // Supermaven
            ".supermaven",
            // JetBrains IDEs (native installs)
            ".local/share/JetBrains",
            ".config/JetBrains",
            "Library/Application Support/JetBrains",
            ".AndroidStudio",
            ".IntelliJIdea",
            ".PyCharm",
            ".WebStorm",
            ".PhpStorm",
            ".CLion",
            ".GoLand",
            ".RustRover",
        ];

        // Parallel scan using rayon
        let locations_mutex = Mutex::new(locations);
        let tools_mutex = Mutex::new(tools_found);

        search_patterns.par_iter().for_each(|pattern| {
            let search_path = self.base_dir.join(pattern);
            if search_path.exists() {
                debug!("Found: {}", search_path.display());
                let mut local_locations = Vec::new();
                let mut local_tools = HashSet::new();

                if self
                    .scan_directory(&search_path, &mut local_locations, &mut local_tools)
                    .is_ok()
                {
                    // Merge results
                    if let Ok(mut locs) = locations_mutex.lock() {
                        locs.extend(local_locations);
                    }
                    if let Ok(mut tools) = tools_mutex.lock() {
                        tools.extend(local_tools);
                    }
                }
            }
        });

        let mut locations = locations_mutex.into_inner().unwrap();
        let tools_found = tools_mutex.into_inner().unwrap();

        // Additional scan for logs in common locations
        self.scan_logs_directory(
            &mut locations,
            &mut tools_found.clone().into_iter().collect(),
        )?;

        let total_size_bytes = locations.iter().map(|l| l.size_bytes).sum();
        let total_files = locations.iter().map(|l| l.file_count).sum();
        let tools_found: Vec<_> = tools_found.into_iter().collect();

        Ok(DiscoveryFindings {
            locations,
            total_size_bytes,
            total_files,
            tools_found,
        })
    }

    fn scan_directory(
        &self,
        path: &Path,
        locations: &mut Vec<LogLocation>,
        tools_found: &mut HashSet<AiTool>,
    ) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let Some(tool) = AiTool::from_path(path) else {
            return Ok(());
        };

        // Check for known subdirectories and files
        let subdirs = vec![
            ("debug", LogType::Debug),
            ("file-history", LogType::FileHistory),
            ("history.jsonl", LogType::History),
            ("sessions", LogType::Session),
            ("session-env", LogType::Session),
            ("telemetry", LogType::Telemetry),
            ("shell-snapshots", LogType::ShellSnapshot),
            ("todos", LogType::Todo),
            ("plugins", LogType::Plugin),
            ("cache", LogType::Cache),
            ("logs", LogType::Debug),
            // VSCode extension-specific directories
            ("tasks", LogType::Session), // Cline/Roo-Cline conversation data
            ("checkpoints", LogType::FileHistory), // Cline checkpoints
            ("settings", LogType::Cache),
            ("state", LogType::Cache),
            ("puppeteer", LogType::Cache),
            ("copilotCli", LogType::Cache),
            ("debugCommand", LogType::Debug),
            ("logContextRecordings", LogType::Debug),
            // Individual files (state databases, embeddings)
            ("state.vscdb", LogType::Session), // Cursor/VSCode state DB
            ("state.vscdb.backup", LogType::Session), // Cursor/VSCode state DB backup
            ("commandEmbeddings.json", LogType::Cache), // Copilot embeddings
            ("settingEmbeddings.json", LogType::Cache), // Copilot embeddings
            ("toolEmbeddingsCache.bin", LogType::Cache), // Copilot embeddings
        ];

        for (subdir_name, log_type) in subdirs {
            let subdir_path = path.join(subdir_name);
            if subdir_path.exists() {
                match self.analyze_location(&subdir_path, &tool, log_type) {
                    Ok(Some(loc)) => {
                        tools_found.insert(tool.clone());
                        locations.push(loc);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        warn!("Error analyzing {}: {}", subdir_path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    fn analyze_location(
        &self,
        path: &PathBuf,
        tool: &AiTool,
        log_type: LogType,
    ) -> Result<Option<LogLocation>> {
        let metadata = fs::metadata(path)?;

        let (size_bytes, file_count) = if metadata.is_dir() {
            self.calculate_dir_size(path)?
        } else {
            (metadata.len(), 1)
        };

        if size_bytes == 0 {
            return Ok(None);
        }

        // Try to get date range
        let (oldest, newest) = self.get_date_range(path)?;

        Ok(Some(LogLocation {
            tool: tool.clone(),
            path: path.clone(),
            log_type,
            size_bytes,
            file_count,
            oldest_entry: oldest,
            newest_entry: newest,
        }))
    }

    fn calculate_dir_size(&self, path: &PathBuf) -> Result<(u64, usize)> {
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                // Skip hidden files/dirs if include_hidden is false
                if !self.include_hidden {
                    if let Some(name) = e.file_name().to_str() {
                        if name.starts_with('.') && name != "." && name != ".." {
                            return false;
                        }
                    }
                }
                true
            })
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }

        Ok((total_size, file_count))
    }

    fn get_date_range(&self, path: &Path) -> Result<DateRange> {
        // Simplified - would need to actually parse log contents for real dates
        let metadata = fs::metadata(path)?;

        let modified = metadata.modified().ok().map(DateTime::<Utc>::from);

        Ok((modified, modified))
    }

    fn scan_logs_directory(
        &self,
        _locations: &mut Vec<LogLocation>,
        _tools_found: &mut HashSet<AiTool>,
    ) -> Result<()> {
        // Additional scanning logic for other log locations
        Ok(())
    }
}
