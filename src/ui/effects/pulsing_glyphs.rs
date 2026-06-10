//! PulsingGlyphs — Verb (Pulsing) × Noun (Glyphs).
//!
//! Characters that rhythmically scale/brighten in place (no net motion).
//! Useful as audio-visualizer "heartbeat" or focus indicator.
//! Default style: `Solid`. Default palette: `Monochrome(Accent)`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Palette, Speed, Style, resolve_color};

#[derive(Clone, Copy, Debug)]
pub struct PulseGlyph {
    pub x: f32,
    pub y: f32,
    pub ch: char,
    pub phase: f32,
    pub freq: f32,
}

/// Rhythmically pulsing characters in place.
pub struct PulsingGlyphs {
    pub glyphs: Vec<PulseGlyph>,
    pub char_pool: Vec<char>,
    pub style: Style,
    pub palette: Palette,
    pub speed: Speed,
    pub density_setting: Density,
    time: f32,
    // 4.0: reserved for future focus/active tracking
    #[allow(dead_code)]
    active: bool,
    #[allow(dead_code)]
    focused: bool,
    #[allow(dead_code)]
    rng: LcgRng,
}

impl PulsingGlyphs {
    /// Construct with default style (`Solid`) and palette (`Accent`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(0xDEAF);
        let density_setting = Density::Normal;
        let count = ((((cols * rows / 6) as f32) * density_setting.multiplier()).max(20.0)) as usize;
        let char_pool: Vec<char> = "·•●◦○◌".chars().collect();
        let glyphs = (0..count).map(|_| PulseGlyph {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(0.0, rows as f32),
            ch: char_pool[rng.next_usize(char_pool.len())],
            phase: rng.next_range(0.0, std::f32::consts::TAU),
            freq: rng.next_range(1.0, 3.0),
        }).collect();
        Self {
            glyphs,
            char_pool,
            style: Style::Solid,
            palette: Palette::ACCENT,
            speed: Speed::Normal,
            density_setting,
            time: 0.0,
            active: true,
            focused: true,
            rng,
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    pub fn with_speed(mut self, speed: Speed) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_density(mut self, density: Density) -> Self {
        self.density_setting = density;
        self
    }

    pub fn update(&mut self, dt: std::time::Duration, _cols: usize, _rows: usize) {
        if !self.active {
            return;
        }
        let dt = dt.as_secs_f32();
        self.time += dt * self.speed.multiplier();
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if !self.active {
            return;
        }
        for g in &self.glyphs {
            let x = g.x as usize;
            let y = g.y as usize;
            if x < cols && y < rows {
                let idx = y * cols + x;
                if idx < grid.len() {
                    // Pulse: sine wave 0..1, intensity 40..255
                    let pulse = (self.time * g.freq + g.phase).sin() * 0.5 + 0.5;
                    let intensity = (40.0 + pulse * 215.0) as u8;
                    let t = pulse;
                    let (r, g_c, b) = resolve_color(self.palette, t);
                    let ch = match self.style {
                        Style::Flared if pulse > 0.85 => '✦',
                        Style::Trailing => match (pulse * 4.0) as usize {
                            0 => '·',
                            1 => '•',
                            2 => '●',
                            _ => '◉',
                        },
                        _ => g.ch,
                    };
                    grid[idx] = TerminalCell {
                        ch,
                        fg: (
                            ((r as u16 * intensity as u16) >> 8) as u8,
                            ((g_c as u16 * intensity as u16) >> 8) as u8,
                            ((b as u16 * intensity as u16) >> 8) as u8,
                        ),
                        bg: (0, 0, 0),
                        bold: pulse > 0.6,
                    };
                }
            }
        }
    }
}

impl crate::ui::screensaver_renderer::Screensaver for PulsingGlyphs {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        PulsingGlyphs::draw(self, grid, cols, rows);
    }
}
