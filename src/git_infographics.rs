// Git Infographics Generator - Beautiful visualizations from git history
use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike};
use indicatif::{ProgressBar, ProgressStyle};
use md5::{Digest, Md5};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

const CHART_WIDTH: u32 = 1200;
const CHART_HEIGHT: u32 = 800;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum InfographicsError {
    #[error("Git repository not found: {0}")]
    RepoNotFound(PathBuf),
    #[error("Failed to parse git log: {0}")]
    ParseError(String),
    #[error("Failed to generate chart: {0}")]
    ChartError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InfographicsConfig {
    pub chart_width: u32,
    pub chart_height: u32,
    pub top_n_contributors: usize,
    pub use_cache: bool,
    pub show_progress: bool,
    pub charts: Option<Vec<String>>, // None = all charts
}

impl Default for InfographicsConfig {
    fn default() -> Self {
        Self {
            chart_width: CHART_WIDTH,
            chart_height: CHART_HEIGHT,
            top_n_contributors: 15,
            use_cache: false,
            show_progress: false,
            charts: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
    pub message: String,
    pub insertions: usize,
    pub deletions: usize,
    pub files_changed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStats {
    pub commits: Vec<GitCommit>,
    pub total_commits: usize,
    pub total_authors: usize,
    pub date_range: (NaiveDate, NaiveDate),
    pub commits_by_author: HashMap<String, usize>,
    pub commits_by_hour: [usize; 24],
    pub commits_by_weekday: [usize; 7],
    pub commits_by_date: HashMap<NaiveDate, usize>,
    pub lines_by_author: HashMap<String, (usize, usize)>, // (additions, deletions)
    pub message_lengths: Vec<usize>,
}

impl GitStats {
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
            total_commits: 0,
            total_authors: 0,
            date_range: (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            ),
            commits_by_author: HashMap::new(),
            commits_by_hour: [0; 24],
            commits_by_weekday: [0; 7],
            commits_by_date: HashMap::new(),
            lines_by_author: HashMap::new(),
            message_lengths: Vec::new(),
        }
    }
}

pub struct GitInfographicsGenerator {
    pub git_dirs: Vec<PathBuf>,
    pub output_dir: PathBuf,
    pub config: InfographicsConfig,
}

impl GitInfographicsGenerator {
    pub fn new(git_dirs: Vec<PathBuf>, output_dir: PathBuf) -> Self {
        Self {
            git_dirs,
            output_dir,
            config: InfographicsConfig::default(),
        }
    }

    pub fn with_config(mut self, config: InfographicsConfig) -> Self {
        self.config = config;
        self
    }

    fn get_cache_path(&self) -> PathBuf {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("vibedev")
            .join("git-stats");
        fs::create_dir_all(&cache_dir).ok();

        // Hash repo paths for cache key
        let repos_str = self
            .git_dirs
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(":");

        let mut hasher = Md5::new();
        hasher.update(repos_str.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        cache_dir.join(format!("{}.cache", hash))
    }

    fn load_cached_stats(&self) -> Option<GitStats> {
        if !self.config.use_cache {
            return None;
        }

        let cache_path = self.get_cache_path();
        if !cache_path.exists() {
            return None;
        }

        // Check if cache is fresh (< 1 hour old)
        if let Ok(metadata) = fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > 3600 {
                        return None; // Cache expired
                    }
                }
            }
        }

        fs::read(&cache_path)
            .ok()
            .and_then(|data| bincode::deserialize(&data).ok())
    }

    fn save_cached_stats(&self, stats: &GitStats) -> Result<()> {
        if !self.config.use_cache {
            return Ok(());
        }

        let cache_path = self.get_cache_path();
        let data = bincode::serialize(stats)?;
        fs::write(&cache_path, data)?;
        Ok(())
    }

    /// Collect git stats from multiple repositories
    pub fn collect_stats(&self) -> Result<GitStats> {
        // Try cache first
        if let Some(cached) = self.load_cached_stats() {
            if self.config.show_progress {
                println!("✓ Using cached statistics");
            }
            return Ok(cached);
        }

        let pb = if self.config.show_progress {
            let pb = ProgressBar::new(self.git_dirs.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed}] {bar:40.cyan/blue} {pos}/{len} repos {msg}")
                    .unwrap()
                    .progress_chars("=>-"),
            );
            Some(pb)
        } else {
            None
        };

        let mut all_commits = Vec::new();

        for git_dir in &self.git_dirs {
            if let Ok(commits) = self.parse_git_log(git_dir) {
                all_commits.extend(commits);
            }
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Done!");
        }

        if all_commits.is_empty() {
            return Ok(GitStats::new());
        }

        // Sort by timestamp
        all_commits.sort_by_key(|c| c.timestamp);

        let mut stats = GitStats::new();
        stats.commits = all_commits.clone();
        stats.total_commits = all_commits.len();

        // Calculate date range
        if let (Some(first), Some(last)) = (all_commits.first(), all_commits.last()) {
            let first_date = DateTime::from_timestamp(first.timestamp, 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                .date_naive();
            let last_date = DateTime::from_timestamp(last.timestamp, 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                .date_naive();
            stats.date_range = (first_date, last_date);
        }

        // Aggregate stats
        for commit in &all_commits {
            // Author stats
            *stats
                .commits_by_author
                .entry(commit.author.clone())
                .or_insert(0) += 1;

            // Lines of code
            let entry = stats
                .lines_by_author
                .entry(commit.author.clone())
                .or_insert((0, 0));
            entry.0 += commit.insertions;
            entry.1 += commit.deletions;

            // Time-based stats
            if let Some(dt) = DateTime::from_timestamp(commit.timestamp, 0) {
                let hour = dt.hour() as usize;
                let weekday = dt.weekday().num_days_from_monday() as usize;
                let date = dt.date_naive();

                stats.commits_by_hour[hour] += 1;
                stats.commits_by_weekday[weekday] += 1;
                *stats.commits_by_date.entry(date).or_insert(0) += 1;
            }

            // Message length
            stats.message_lengths.push(commit.message.len());
        }

        stats.total_authors = stats.commits_by_author.len();

        // Save to cache
        self.save_cached_stats(&stats)?;

        Ok(stats)
    }

    /// Parse git log from a repository
    fn parse_git_log(&self, git_dir: &Path) -> Result<Vec<GitCommit>> {
        let output = Command::new("git")
            .current_dir(git_dir)
            .args([
                "log",
                "--all",
                "--numstat",
                "--pretty=format:COMMIT|%H|%an|%ae|%at|%s",
            ])
            .output()?;

        let log_text = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();
        let mut current_commit: Option<GitCommit> = None;

        for line in log_text.lines() {
            if line.starts_with("COMMIT|") {
                // Save previous commit
                if let Some(commit) = current_commit.take() {
                    commits.push(commit);
                }

                // Parse new commit
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 6 {
                    current_commit = Some(GitCommit {
                        hash: parts[1].to_string(),
                        author: parts[2].to_string(),
                        email: parts[3].to_string(),
                        timestamp: parts[4].parse().unwrap_or(0),
                        message: parts[5].to_string(),
                        insertions: 0,
                        deletions: 0,
                        files_changed: 0,
                    });
                }
            } else if !line.is_empty() {
                // Parse numstat line (insertions, deletions, filename)
                if let Some(ref mut commit) = current_commit {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let (Ok(ins), Ok(del)) =
                            (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                        {
                            commit.insertions += ins;
                            commit.deletions += del;
                            commit.files_changed += 1;
                        }
                    }
                }
            }
        }

        // Don't forget last commit
        if let Some(commit) = current_commit {
            commits.push(commit);
        }

        Ok(commits)
    }

    /// Generate all infographics
    pub fn generate_all(&self, stats: &GitStats) -> Result<Vec<PathBuf>> {
        fs::create_dir_all(&self.output_dir)?;

        let pb = if self.config.show_progress {
            let pb = ProgressBar::new(7);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed}] {bar:40.green/blue} {pos}/{len} charts {msg}")
                    .unwrap()
                    .progress_chars("█▓▒░"),
            );
            Some(pb)
        } else {
            None
        };

        let mut generated = Vec::new();

        // 1. Commit heatmap (calendar view)
        if let Some(ref pb) = pb {
            pb.set_message("commit_heatmap");
        }
        if let Ok(path) = self.generate_commit_heatmap(stats) {
            generated.push(path);
        }
        if let Some(ref pb) = pb {
            pb.inc(1);
        }

        // 2. Top contributors bar chart
        if let Some(ref pb) = pb {
            pb.set_message("top_contributors");
        }
        if let Ok(path) = self.generate_top_contributors(stats) {
            generated.push(path);
        }
        if let Some(ref pb) = pb {
            pb.inc(1);
        }

        // 3. Activity timeline
        if let Some(ref pb) = pb {
            pb.set_message("activity_timeline");
        }
        if let Ok(path) = self.generate_activity_timeline(stats) {
            generated.push(path);
        }
        if let Some(ref pb) = pb {
            pb.inc(1);
        }

        // 4. Hourly activity heatmap
        if let Some(ref pb) = pb {
            pb.set_message("hourly_activity");
        }
        if let Ok(path) = self.generate_hourly_heatmap(stats) {
            generated.push(path);
        }
        if let Some(ref pb) = pb {
            pb.inc(1);
        }

        // 5. Weekday distribution
        if let Some(ref pb) = pb {
            pb.set_message("weekday_distribution");
        }
        if let Ok(path) = self.generate_weekday_distribution(stats) {
            generated.push(path);
        }
        if let Some(ref pb) = pb {
            pb.inc(1);
        }

        // 6. Commit message quality
        if let Some(ref pb) = pb {
            pb.set_message("message_quality");
        }
        if let Ok(path) = self.generate_message_quality(stats) {
            generated.push(path);
        }
        if let Some(ref pb) = pb {
            pb.inc(1);
        }

        // 7. Code contribution (lines added/deleted)
        if let Some(ref pb) = pb {
            pb.set_message("code_contribution");
        }
        if let Ok(path) = self.generate_code_contribution(stats) {
            generated.push(path);
        }
        if let Some(ref pb) = pb {
            pb.inc(1);
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Complete!");
        }

        Ok(generated)
    }

    /// Generate commit heatmap (GitHub-style calendar)
    fn generate_commit_heatmap(&self, stats: &GitStats) -> Result<PathBuf> {
        let path = self.output_dir.join("commit_heatmap.png");
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        let (start_date, end_date) = stats.date_range;
        let days = (end_date - start_date).num_days() as usize + 1;
        let weeks = days.div_ceil(7);

        // Find max commits per day for color scaling
        let max_commits = stats.commits_by_date.values().max().copied().unwrap_or(1);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("Git Activity Heatmap ({} - {})", start_date, end_date),
                ("sans-serif", 40).into_font(),
            )
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..weeks, 0..7)?;

        chart
            .configure_mesh()
            .y_labels(7)
            .y_label_formatter(&|y| {
                match y {
                    0 => "Mon",
                    1 => "Tue",
                    2 => "Wed",
                    3 => "Thu",
                    4 => "Fri",
                    5 => "Sat",
                    6 => "Sun",
                    _ => "",
                }
                .to_string()
            })
            .draw()?;

        // Draw heatmap squares
        let mut current_date = start_date;
        for week in 0..weeks {
            for day in 0..7 {
                if current_date > end_date {
                    break;
                }

                let commits = stats
                    .commits_by_date
                    .get(&current_date)
                    .copied()
                    .unwrap_or(0);
                let intensity = (commits as f64 / max_commits as f64 * 255.0) as u8;
                let color = RGBColor(255 - intensity, 255, 255 - intensity);

                chart.draw_series(std::iter::once(Rectangle::new(
                    [(week, day), (week, day + 1)],
                    color.filled(),
                )))?;

                current_date += Duration::days(1);
            }
        }

        root.present()?;
        Ok(path_clone)
    }

    /// Generate top contributors bar chart
    fn generate_top_contributors(&self, stats: &GitStats) -> Result<PathBuf> {
        let path = self.output_dir.join("top_contributors.png");
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        // Get top 15 contributors
        let mut sorted: Vec<_> = stats.commits_by_author.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        let top_15: Vec<_> = sorted.into_iter().take(15).collect();

        if top_15.is_empty() {
            return Ok(path_clone);
        }

        let max_commits = *top_15.first().unwrap().1;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Top Contributors by Commit Count",
                ("sans-serif", 40).into_font(),
            )
            .margin(20)
            .x_label_area_size(150)
            .y_label_area_size(60)
            .build_cartesian_2d(0..(top_15.len() as i32), 0..max_commits)?;

        chart
            .configure_mesh()
            .x_labels(top_15.len())
            .x_label_formatter(&|x: &i32| {
                let idx = *x as usize;
                if idx < top_15.len() {
                    top_15[idx].0.chars().take(15).collect()
                } else {
                    String::new()
                }
            })
            .draw()?;

        chart.draw_series(top_15.iter().enumerate().map(|(i, (_, &commits))| {
            let color = Palette99::pick(i).mix(0.9);
            Rectangle::new([(i as i32, 0), (i as i32, commits)], color.filled())
        }))?;

        root.present()?;
        Ok(path_clone)
    }

    /// Generate activity timeline (commits over time)
    fn generate_activity_timeline(&self, stats: &GitStats) -> Result<PathBuf> {
        let path = self.output_dir.join("activity_timeline.png");
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        // Group by month
        let mut monthly_commits: HashMap<(i32, u32), usize> = HashMap::new();
        for (date, &count) in &stats.commits_by_date {
            let key = (date.year(), date.month());
            *monthly_commits.entry(key).or_insert(0) += count;
        }

        let mut months: Vec<_> = monthly_commits.keys().collect();
        months.sort();

        if months.is_empty() {
            return Ok(path_clone);
        }

        let max_commits = monthly_commits.values().max().copied().unwrap_or(1);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Commit Activity Over Time (Monthly)",
                ("sans-serif", 40).into_font(),
            )
            .margin(20)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .build_cartesian_2d(0..months.len(), 0..max_commits)?;

        chart
            .configure_mesh()
            .x_label_formatter(&|x| {
                let idx = *x;
                if idx < months.len() {
                    let (year, month) = months[idx];
                    format!("{}-{:02}", year, month)
                } else {
                    String::new()
                }
            })
            .draw()?;

        chart.draw_series(LineSeries::new(
            months
                .iter()
                .enumerate()
                .map(|(i, &&key)| (i, monthly_commits[&key])),
            &RED,
        ))?;

        chart.draw_series(PointSeries::of_element(
            months
                .iter()
                .enumerate()
                .map(|(i, &&key)| (i, monthly_commits[&key])),
            5,
            &RED,
            &|c, s, st| EmptyElement::at(c) + Circle::new((0, 0), s, st.filled()),
        ))?;

        root.present()?;
        Ok(path_clone)
    }

    /// Generate hourly activity heatmap
    fn generate_hourly_heatmap(&self, stats: &GitStats) -> Result<PathBuf> {
        let path = self.output_dir.join("hourly_activity.png");
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path, (CHART_WIDTH, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_commits = *stats.commits_by_hour.iter().max().unwrap_or(&1);

        let mut chart = ChartBuilder::on(&root)
            .caption("Commits by Hour of Day", ("sans-serif", 40).into_font())
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..24, 0..max_commits)?;

        chart
            .configure_mesh()
            .x_labels(24)
            .x_label_formatter(&|x| format!("{}h", x))
            .draw()?;

        chart.draw_series(
            stats
                .commits_by_hour
                .iter()
                .enumerate()
                .map(|(hour, &commits)| {
                    let intensity = (commits as f64 / max_commits as f64 * 255.0) as u8;
                    let color = RGBColor(intensity, 100, 255 - intensity);
                    Rectangle::new(
                        [(hour as i32, 0), (hour as i32 + 1, commits)],
                        color.filled(),
                    )
                }),
        )?;

        root.present()?;
        Ok(path_clone)
    }

    /// Generate weekday distribution
    fn generate_weekday_distribution(&self, stats: &GitStats) -> Result<PathBuf> {
        let path = self.output_dir.join("weekday_distribution.png");
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path, (CHART_WIDTH, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let weekday_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let max_commits = *stats.commits_by_weekday.iter().max().unwrap_or(&1);

        let mut chart = ChartBuilder::on(&root)
            .caption("Commits by Day of Week", ("sans-serif", 40).into_font())
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..7, 0..max_commits)?;

        chart
            .configure_mesh()
            .x_labels(7)
            .x_label_formatter(&|x: &i32| {
                let idx = (*x as usize).min(6);
                weekday_names[idx].to_string()
            })
            .draw()?;

        chart.draw_series(
            stats
                .commits_by_weekday
                .iter()
                .enumerate()
                .map(|(day, &commits)| {
                    let color = if day < 5 {
                        BLUE.mix(0.7)
                    } else {
                        GREEN.mix(0.7)
                    };
                    Rectangle::new([(day as i32, 0), (day as i32 + 1, commits)], color.filled())
                }),
        )?;

        root.present()?;
        Ok(path_clone)
    }

    /// Generate commit message quality histogram
    fn generate_message_quality(&self, stats: &GitStats) -> Result<PathBuf> {
        let path = self.output_dir.join("message_quality.png");
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        // Bucket message lengths
        let mut buckets = [0; 10]; // <10, 10-20, 20-30, ..., 80-90, 90+
        for &len in &stats.message_lengths {
            let bucket = (len / 10).min(9);
            buckets[bucket] += 1;
        }

        let max_count = *buckets.iter().max().unwrap_or(&1);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Commit Message Length Distribution",
                ("sans-serif", 40).into_font(),
            )
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0..10, 0..max_count)?;

        chart
            .configure_mesh()
            .x_labels(10)
            .x_label_formatter(&|x| {
                if *x == 9 {
                    "90+".to_string()
                } else {
                    format!("{}-{}", x * 10, (x + 1) * 10)
                }
            })
            .draw()?;

        chart.draw_series(buckets.iter().enumerate().map(|(i, &count)| {
            // Color code: red for too short, green for optimal (50-72), yellow otherwise
            let color = if i < 5 {
                RED.mix(0.7) // Too short
            } else if (5..=7).contains(&i) {
                GREEN.mix(0.7) // Optimal
            } else {
                YELLOW.mix(0.7) // Too long
            };
            Rectangle::new([(i as i32, 0), (i as i32 + 1, count)], color.filled())
        }))?;

        root.present()?;
        Ok(path_clone)
    }

    /// Generate code contribution chart (lines added/deleted by top contributors)
    fn generate_code_contribution(&self, stats: &GitStats) -> Result<PathBuf> {
        let path = self.output_dir.join("code_contribution.png");
        let path_clone = path.clone();
        let root = BitMapBackend::new(&path, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        // Get top 10 by total lines changed
        let mut sorted: Vec<_> = stats
            .lines_by_author
            .iter()
            .map(|(author, (add, del))| (author.clone(), add + del))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        let top_10: Vec<_> = sorted.into_iter().take(10).map(|(a, _)| a).collect();

        if top_10.is_empty() {
            return Ok(path_clone);
        }

        let max_lines = top_10
            .iter()
            .map(|author| {
                let (add, del) = stats.lines_by_author.get(author).unwrap_or(&(0, 0));
                add + del
            })
            .max()
            .unwrap_or(1);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Code Contribution (Lines Changed)",
                ("sans-serif", 40).into_font(),
            )
            .margin(20)
            .x_label_area_size(150)
            .y_label_area_size(80)
            .build_cartesian_2d(0..(top_10.len() as i32 * 2), 0..max_lines)?;

        chart
            .configure_mesh()
            .x_labels(top_10.len())
            .x_label_formatter(&|x: &i32| {
                let idx = (*x / 2) as usize;
                if idx < top_10.len() {
                    top_10[idx].chars().take(15).collect()
                } else {
                    String::new()
                }
            })
            .draw()?;

        // Draw stacked bars (additions + deletions)
        for (i, author) in top_10.iter().enumerate() {
            let (additions, deletions) = stats.lines_by_author.get(author).unwrap_or(&(0, 0));

            let base_x = (i * 2) as i32;

            // Additions (green)
            chart.draw_series(std::iter::once(Rectangle::new(
                [(base_x, 0), (base_x + 1, *additions)],
                GREEN.mix(0.6).filled(),
            )))?;

            // Deletions (red)
            chart.draw_series(std::iter::once(Rectangle::new(
                [(base_x + 1, 0), (base_x + 2, *deletions)],
                RED.mix(0.6).filled(),
            )))?;
        }

        root.present()?;
        Ok(path_clone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_stats_creation() {
        let stats = GitStats::new();
        assert_eq!(stats.total_commits, 0);
        assert_eq!(stats.total_authors, 0);
    }
}
