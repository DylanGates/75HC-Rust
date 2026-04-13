use colored::*;
use std::io::{self, Write};

pub struct ProgressBar {
    width: usize,
    filled_char: char,
    empty_char: char,
    color_filled: Color,
    color_empty: Color,
}

#[derive(Clone, Copy)]
pub enum Color {
    Green,
    Yellow,
    Red,
    Blue,
    Cyan,
    White,
    Dimmed,
}

impl ProgressBar {
    pub fn new(width: usize) -> Self {
        Self {
            width,
            filled_char: '█',
            empty_char: '░',
            color_filled: Color::Green,
            color_empty: Color::Dimmed,
        }
    }

    pub fn with_chars(mut self, filled: char, empty: char) -> Self {
        self.filled_char = filled;
        self.empty_char = empty;
        self
    }

    pub fn with_colors(mut self, filled: Color, empty: Color) -> Self {
        self.color_filled = filled;
        self.color_empty = empty;
        self
    }

    pub fn render(&self, percent: f64) -> String {
        let clamped = percent.clamp(0.0, 100.0);
        let filled_count = ((clamped / 100.0) * self.width as f64).round() as usize;
        let empty_count = self.width.saturating_sub(filled_count);

        let filled_str: String = std::iter::repeat(self.filled_char).take(filled_count).collect();
        let empty_str: String = std::iter::repeat(self.empty_char).take(empty_count).collect();

        format!(
            "{}{} {:.1}%",
            self.colorize(&filled_str, self.color_filled),
            self.colorize(&empty_str, self.color_empty),
            clamped
        )
    }

    fn colorize(&self, text: &str, color: Color) -> String {
        match color {
            Color::Green => text.green(),
            Color::Yellow => text.yellow(),
            Color::Red => text.red(),
            Color::Blue => text.blue(),
            Color::Cyan => text.cyan(),
            Color::White => text.white(),
            Color::Dimmed => text.dimmed(),
        }.to_string()
    }

    /// Render with animation for CLI feedback
    pub fn render_with_label(&self, percent: f64, label: &str) -> String {
        format!("{} {}", self.render(percent), label)
    }
}

pub struct StreakBar;

impl StreakBar {
    pub fn render(streak: u32, best: u32) -> String {
        let bar = ProgressBar::new(10)
            .with_chars('🔥', '○')
            .with_colors(Color::Yellow, Color::Dimmed);
        
        let percent = if best > 0 {
            (streak as f64 / best as f64) * 100.0
        } else {
            0.0
        };
        
        format!("{} {} (best: {})", bar.render(percent), streak, best)
    }
}

pub struct WeeklyProgressBar;

impl WeeklyProgressBar {
    pub fn render(current: u32, target: u32) -> String {
        let bar = ProgressBar::new(8)
            .with_chars('✓', '·')
            .with_colors(Color::Green, Color::Dimmed);
        
        let percent = if target > 0 {
            (current as f64 / target as f64) * 100.0
        } else {
            0.0
        };
        
        let color = if percent >= 100.0 {
            Color::Green
        } else if percent >= 50.0 {
            Color::Yellow
        } else {
            Color::Red
        };
        
        let bar = ProgressBar::new(8)
            .with_chars('█', '░')
            .with_colors(color, Color::Dimmed);
        
        format!("{}/{} {}", current, target, bar.render(percent.min(100.0)))
    }
}