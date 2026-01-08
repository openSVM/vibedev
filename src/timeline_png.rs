// Timeline PNG export - Generate visual timeline charts
use crate::timeline::{SessionOutcome, Timeline};
use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use plotters::prelude::*;
use std::path::{Path, PathBuf};

const WIDTH: u32 = 2400;
const HEIGHT: u32 = 1600;
const TITLE_FONT_SIZE: u32 = 48;
const LABEL_FONT_SIZE: u32 = 24;
const STATS_FONT_SIZE: u32 = 20;

pub struct TimelinePngRenderer {
    output_path: PathBuf,
}

impl TimelinePngRenderer {
    pub fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    pub fn render(&self, timeline: &Timeline) -> Result<()> {
        let root = BitMapBackend::new(&self.output_path, (WIDTH, HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        // Split into title, stats, and timeline sections
        let (title_area, rest) = root.split_vertically(80);
        let (stats_area, timeline_area) = rest.split_vertically(180);

        // Draw title
        self.draw_title(&title_area)?;

        // Draw stats
        self.draw_stats(&stats_area, timeline)?;

        // Draw timeline
        self.draw_timeline(&timeline_area, timeline)?;

        root.present()?;
        Ok(())
    }

    fn draw_title(&self, area: &DrawingArea<BitMapBackend, plotters::coord::Shift>) -> Result<()> {
        area.fill(&RGBColor(240, 240, 250))?;

        area.draw_text(
            "Your Coding Journey Timeline",
            &TextStyle::from(("sans-serif", TITLE_FONT_SIZE).into_font())
                .color(&RGBColor(40, 40, 80)),
            ((WIDTH / 2) as i32, 45),
        )?;

        Ok(())
    }

    fn draw_stats(
        &self,
        area: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
        timeline: &Timeline,
    ) -> Result<()> {
        area.fill(&RGBColor(250, 250, 250))?;

        let stats = &timeline.stats;

        // Draw stats boxes
        let stats_text = vec![
            format!("Total Sessions: {}", stats.total_sessions),
            format!("Completed: {}", stats.completed),
            format!("Abandoned: {}", stats.abandoned),
            format!("Resumed: {}", stats.resumed),
            format!("Ongoing: {}", stats.ongoing),
            format!("Completion Rate: {:.1}%", stats.completion_rate),
            format!("Avg Session: {:.1}h", stats.avg_session_hours),
            format!("Context Switches: {}", stats.context_switches),
            format!("Top Project: {}", stats.most_worked_project),
        ];

        let x_start = 50;
        let y_start = 30;
        let col_width = 480; // Narrower to fit 5 columns
        let cols = 5;

        for (idx, text) in stats_text.iter().enumerate() {
            let x = x_start + (idx % cols) * col_width;
            let y = y_start + (idx / cols) * 55;

            area.draw_text(
                text,
                &TextStyle::from(("sans-serif", STATS_FONT_SIZE).into_font())
                    .color(&RGBColor(60, 60, 100)),
                (x as i32, y as i32),
            )?;
        }

        Ok(())
    }

    fn draw_timeline(
        &self,
        area: &DrawingArea<BitMapBackend, plotters::coord::Shift>,
        timeline: &Timeline,
    ) -> Result<()> {
        if timeline.sessions.is_empty() {
            area.draw_text(
                "No sessions found",
                &TextStyle::from(("sans-serif", LABEL_FONT_SIZE).into_font())
                    .color(&RGBColor(150, 150, 150)),
                (WIDTH as i32 / 2, 200),
            )?;
            return Ok(());
        }

        // Get time range
        let first = timeline.sessions.first().unwrap();
        let last = timeline.sessions.last().unwrap();

        let start_time = first.start.timestamp();
        let end_time = last.end.timestamp().max(start_time + 3600); // At least 1 hour range

        // Calculate dimensions
        let margin_left = 150;
        let margin_right = 50;
        let margin_top = 50;
        let margin_bottom = 50;

        let chart_width = (WIDTH - margin_left - margin_right) as i32;
        let chart_height = (HEIGHT - 260 - margin_top - margin_bottom) as i32; // 260 = title + stats

        // Draw border
        area.draw(&Rectangle::new(
            [
                (margin_left as i32, margin_top as i32),
                (
                    (WIDTH - margin_right) as i32,
                    (HEIGHT - 260 - margin_bottom) as i32,
                ),
            ],
            ShapeStyle::from(RGBColor(200, 200, 200)).stroke_width(2),
        ))?;

        // Draw time axis
        let time_steps: i64 = 10;
        let time_step = (end_time - start_time) / time_steps;

        for i in 0..=time_steps {
            let t = start_time + i * time_step;
            let x = margin_left as i32 + (chart_width * i as i32 / time_steps as i32);
            let dt = DateTime::from_timestamp(t, 0).unwrap_or(Utc::now());

            // Draw vertical grid line
            area.draw(&PathElement::new(
                vec![
                    (x, margin_top as i32),
                    (x, (HEIGHT - 260 - margin_bottom) as i32),
                ],
                ShapeStyle::from(RGBColor(230, 230, 230)).stroke_width(1),
            ))?;

            // Draw date label
            let label = format!("{}/{}", dt.month(), dt.day());
            area.draw_text(
                &label,
                &TextStyle::from(("sans-serif", 14).into_font()).color(&RGBColor(100, 100, 100)),
                (x, (HEIGHT - 260 - margin_bottom + 10) as i32),
            )?;
        }

        // Calculate bar height - fit ALL sessions properly
        let num_sessions = timeline.sessions.len().min(500); // Cap at 500 for readability
        let sessions_to_show: Vec<_> = if timeline.sessions.len() > 500 {
            // Show first 250 and last 250
            timeline
                .sessions
                .iter()
                .take(250)
                .chain(timeline.sessions.iter().skip(timeline.sessions.len() - 250))
                .cloned()
                .collect()
        } else {
            timeline.sessions.clone()
        };

        let bar_height = (chart_height / num_sessions as i32).clamp(3, 20);

        // Draw sessions
        for (idx, session) in sessions_to_show.iter().enumerate() {
            let session_start = session.start.timestamp();
            let session_end = session.end.timestamp();

            let x1 = margin_left as i32
                + ((session_start - start_time) * chart_width as i64 / (end_time - start_time))
                    as i32;
            let x2 = margin_left as i32
                + ((session_end - start_time) * chart_width as i64 / (end_time - start_time))
                    as i32;

            let y = margin_top as i32 + (idx as i32 * bar_height) + 5;

            // Ensure minimum bar width for visibility
            let x2 = x2.max(x1 + 3);

            // Color based on outcome
            let color = match &session.outcome {
                SessionOutcome::Completed => RGBColor(50, 200, 100), // Green
                SessionOutcome::Abandoned => RGBColor(220, 80, 80),  // Red
                SessionOutcome::Resumed(_) => RGBColor(220, 180, 50), // Yellow
                SessionOutcome::Ongoing => RGBColor(80, 160, 220),   // Cyan
            };

            // Draw session bar
            area.draw(&Rectangle::new(
                [(x1, y), (x2, y + bar_height - 2)],
                ShapeStyle::from(color).filled(),
            ))?;

            // Draw project label for first occurrence or longer sessions
            if idx == 0 || session.hours > 1.0 {
                let label = format!("{} ({:.1}h)", session.project, session.hours);
                area.draw_text(
                    &label,
                    &TextStyle::from(("sans-serif", 12).into_font()).color(&RGBColor(60, 60, 60)),
                    (x1 - 5, y - 2),
                )?;
            }
        }

        // Draw legend
        let legend_x = margin_left as i32;
        let legend_y = (HEIGHT - 260 - margin_bottom + 40) as i32;

        let legend_items = [
            ("Completed", RGBColor(50, 200, 100)),
            ("Abandoned", RGBColor(220, 80, 80)),
            ("Resumed", RGBColor(220, 180, 50)),
            ("Ongoing", RGBColor(80, 160, 220)),
        ];

        for (idx, (label, color)) in legend_items.iter().enumerate() {
            let x = legend_x + (idx as i32 * 280);

            area.draw(&Rectangle::new(
                [(x, legend_y), (x + 20, legend_y + 15)],
                ShapeStyle::from(*color).filled(),
            ))?;

            area.draw_text(
                label,
                &TextStyle::from(("sans-serif", 16).into_font()).color(&RGBColor(60, 60, 60)),
                (x + 30, legend_y + 12),
            )?;
        }

        Ok(())
    }
}

pub fn export_timeline_png(timeline: &Timeline, output_path: &Path) -> Result<()> {
    let renderer = TimelinePngRenderer::new(output_path.to_path_buf());
    renderer.render(timeline)?;
    Ok(())
}
