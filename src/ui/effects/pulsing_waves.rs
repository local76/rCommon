//! PulsingWaves — Verb (Pulsing) × Noun (Waves).
//!
//! Horizontal sine waves whose amplitude pulses rhythmically.
//! Useful as audio visualizer or ambient backdrop.
//! Default style: `Solid`. Default palette: `Heat`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Palette, Speed, Style, resolve_color};

#[derive(Clone, Copy, Debug)]
pub struct WaveLine {
    pub row: f32,
    pub freq: f32,
    pub speed: f32,
    pub phase: f32,
}

/// Stack of horizontal sine waves with pulsing amplitude.
pub struct PulsingWaves {
    pub lines: Vec<WaveLine>,
    pub char_pool: Vec<char>,
    pub style: Style,
    pub palette: Palette,
    pub speed: Speed,
    pub density_setting: Density,
    time: f32,
    pulse: f32,
    // 4.0: reserved for future focus/active tracking
    #[allow(dead_code)]
    active: bool,
    #[allow(dead_code)]
    focused: bool,
    #[allow(dead_code)]
    rng: LcgRng,
}

impl PulsingWaves {
    /// Construct with default style (`Solid`) and palette (`Heat`).
    pub fn new(_cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(0xCAFE);
        let density_setting = Density::Normal;
        let count = ((((rows / 2).max(3)) as f32) * density_setting.multiplier()).max(3.0) as usize;
        let lines = (0..count).map(|i| WaveLine {
            row: (i as f32 / count as f32) * rows as f32,
            freq: rng.next_range(0.1, 0.4),
            speed: rng.next_range(0.5, 2.0),
            phase: rng.next_range(0.0, std::f32::consts::TAU),
        }).collect();
        let char_pool: Vec<char> = "▁▂▃▄▅▆▇█".chars().collect();
        Self {
            lines,
            char_pool,
            style: Style::Solid,
            palette: Palette::HEAT,
            speed: Speed::Normal,
            density_setting,
            time: 0.0,
            pulse: 0.0,
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
        let m = self.speed.multiplier();
        self.time += dt * m;
        self.pulse = (self.time * 1.5).sin() * 0.5 + 0.5;
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if !self.active {
            return;
        }
        let amp = 2.0 + self.pulse * (rows as f32 * 0.3);
        for line in &self.lines {
            for x in 0..cols {
                let phase = x as f32 * line.freq + self.time * line.speed + line.phase;
                let offset = (phase.sin() * amp) as isize;
                let y = line.row as isize + offset;
                if y >= 0 && (y as usize) < rows {
                    let idx = y as usize * cols + x;
                    if idx < grid.len() {
                        let t = (y as f32 / rows as f32).clamp(0.0, 1.0);
                        let (r, g, b) = resolve_color(self.palette, t);
                        let intensity = (160.0 + self.pulse * 95.0) as u8;
                        let bucket = offset.unsigned_abs().min(self.char_pool.len() - 1);
                        let ch = match self.style {
                            Style::Flared if self.pulse > 0.7 => '~',
                            Style::Trailing => self.char_pool[bucket],
                            _ => self.char_pool[bucket.min(4)],
                        };
                        grid[idx] = TerminalCell {
                            ch,
                            fg: (
                                ((r as u16 * intensity as u16) >> 8) as u8,
                                ((g as u16 * intensity as u16) >> 8) as u8,
                                ((b as u16 * intensity as u16) >> 8) as u8,
                            ),
                            bg: (0, 0, 0),
                            bold: self.pulse > 0.6,
                        };
                    }
                }
            }
        }
    }
}

impl crate::interface::app::screensaver::Screensaver for PulsingWaves {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        PulsingWaves::draw(self, grid, cols, rows);
    }
}
