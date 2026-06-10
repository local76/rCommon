//! FallingDroplets — Verb (Falling) × Noun (Droplets).
//!
//! Plain falling droplets (no trail). Default style: `Solid`. Default palette: `Monochrome(Blue)`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Direction, Palette, Speed, Style, resolve_color};
use super::RainDrop;

/// Falling droplet streaks.
pub struct FallingDroplets {
    pub(crate) drops: Vec<RainDrop>,
    pub style: Style,
    pub palette: Palette,
    pub speed: Speed,
    pub direction: Direction,
    pub density_setting: Density,
    // 4.0: reserved for future focus/active tracking
    #[allow(dead_code)]
    active: bool,
    #[allow(dead_code)]
    focused: bool,
    rng: LcgRng,
}

impl FallingDroplets {
    /// Construct with default style (`Solid`) and palette (`Monochrome(Blue)`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(777);
        let density_setting = Density::Normal;
        let count = ((((cols / 2).max(1)) as f32) * density_setting.multiplier()).max(1.0) as usize;
        let drops = (0..count).map(|_| RainDrop {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(0.0, rows as f32),
            speed: rng.next_range(0.5, 1.5),
            length: 1,
        }).collect();
        Self {
            drops,
            style: Style::Solid,
            palette: Palette::BLUE,
            speed: Speed::Normal,
            direction: Direction::Down,
            density_setting,
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

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_density(mut self, density: Density) -> Self {
        self.density_setting = density;
        self
    }

    pub fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        if !self.active {
            return;
        }
        let dt = dt.as_secs_f32();
        let m = self.speed.multiplier();
        let (sx, sy) = self.direction.signs();
        for drop in &mut self.drops {
            drop.x += sx * drop.speed * dt * 15.0 * m;
            drop.y += sy * drop.speed * dt * 15.0 * m;
            if drop.y > rows as f32 {
                drop.y = 0.0;
                drop.x = self.rng.next_range(0.0, cols as f32);
            }
        }
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        for drop in &self.drops {
            let y = drop.y as usize;
            let x = drop.x as usize;
            if y < rows && x < cols {
                let idx = y * cols + x;
                let (r, g, b) = resolve_color(self.palette, 0.0);
                grid[idx] = TerminalCell { ch: '|', fg: (r, g, b), bg: (0, 0, 0), bold: false };
            }
        }
    }
}

impl crate::interface::app::screensaver::Screensaver for FallingDroplets {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if self.active {
            FallingDroplets::draw(self, grid, cols, rows);
        }
    }
}
