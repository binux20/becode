//! Theme system for TUI

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub border: String,
    pub muted: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            colors: ThemeColors {
                background: "#1a1a2e".to_string(),
                foreground: "#eaeaea".to_string(),
                primary: "#ffd700".to_string(),    // Bee yellow!
                secondary: "#4a9eff".to_string(),
                accent: "#ff6b6b".to_string(),
                success: "#4ade80".to_string(),
                warning: "#fbbf24".to_string(),
                error: "#ef4444".to_string(),
                border: "#3a3a5a".to_string(),
                muted: "#6b7280".to_string(),
            },
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            colors: ThemeColors {
                background: "#ffffff".to_string(),
                foreground: "#1a1a2e".to_string(),
                primary: "#d4a000".to_string(),
                secondary: "#2563eb".to_string(),
                accent: "#dc2626".to_string(),
                success: "#16a34a".to_string(),
                warning: "#ca8a04".to_string(),
                error: "#dc2626".to_string(),
                border: "#d1d5db".to_string(),
                muted: "#9ca3af".to_string(),
            },
        }
    }

    /// Hacker theme (green on black)
    pub fn hacker() -> Self {
        Self {
            name: "hacker".to_string(),
            colors: ThemeColors {
                background: "#0a0a0a".to_string(),
                foreground: "#00ff00".to_string(),
                primary: "#00ff00".to_string(),
                secondary: "#00aa00".to_string(),
                accent: "#ffff00".to_string(),
                success: "#00ff00".to_string(),
                warning: "#ffff00".to_string(),
                error: "#ff0000".to_string(),
                border: "#004400".to_string(),
                muted: "#006600".to_string(),
            },
        }
    }

    /// Bee yellow theme
    pub fn bee_yellow() -> Self {
        Self {
            name: "bee-yellow".to_string(),
            colors: ThemeColors {
                background: "#1a1a0a".to_string(),
                foreground: "#ffd700".to_string(),
                primary: "#ffd700".to_string(),
                secondary: "#ffaa00".to_string(),
                accent: "#ffffff".to_string(),
                success: "#88ff00".to_string(),
                warning: "#ffaa00".to_string(),
                error: "#ff4444".to_string(),
                border: "#444400".to_string(),
                muted: "#aa8800".to_string(),
            },
        }
    }

    /// Get theme by name
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "hacker" => Self::hacker(),
            "bee-yellow" | "bee" => Self::bee_yellow(),
            _ => Self::dark(),
        }
    }

    /// Parse hex color to ratatui Color
    pub fn parse_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Color::White;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

        Color::Rgb(r, g, b)
    }

    // Style helpers
    pub fn primary_style(&self) -> Style {
        Style::default().fg(Self::parse_color(&self.colors.primary))
    }

    pub fn secondary_style(&self) -> Style {
        Style::default().fg(Self::parse_color(&self.colors.secondary))
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(Self::parse_color(&self.colors.success))
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(Self::parse_color(&self.colors.error))
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(Self::parse_color(&self.colors.warning))
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(Self::parse_color(&self.colors.muted))
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(Self::parse_color(&self.colors.border))
    }

    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(Self::parse_color(&self.colors.primary))
            .add_modifier(Modifier::BOLD)
    }
}
