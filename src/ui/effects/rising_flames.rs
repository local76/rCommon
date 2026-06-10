//! RisingFlames — Verb (Rising) × Noun (Flames).
//!
//! Heat plume of rising bright particles. Default style: `Solid`. Default palette: `Heat`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Direction, Palette, Speed, Style, resolve_color};
use super::Particle;

/// Rising flame plume.
pub struct RisingFlames {
    pub particles: Vec<Particle>,
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

impl RisingFlames {
    /// Construct with default style (`Solid`) and palette (`Heat`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(111);
        let density_setting = Density::Normal;
        let count = ((((cols * rows) / 5) as f32) * density_setting.multiplier()).max(10.0) as usize;
        let particles = (0..count).map(|_| Particle {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(0.0, rows as f32),
            vx: rng.next_range(-0.2, 0.2),
            vy: rng.next_range(-1.0, -0.3),
            ch: '^',
            color: (255, 100, 0),
            life: rng.next_range(0.8, 1.5),
        }).collect();
        Self {
            particles,
            style: Style::Solid,
            palette: Palette::HEAT,
            speed: Speed::Normal,
            direction: Direction::Up,
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
        for p in &mut self.particles {
            p.x += p.vx * dt * 10.0 * m;
            p.y += p.vy * dt * 10.0 * m;
            p.life -= dt;
            p.vy -= 0.1 * dt * m;
            if p.life <= 0.0 {
                p.x = self.rng.next_range(0.0, cols as f32);
                p.y = self.rng.next_range(0.0, rows as f32);
                p.life = self.rng.next_range(0.8, 1.5);
                p.vy = self.rng.next_range(-1.0, -0.3);
            }
        }
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        for p in &self.particles {
            let x = p.x as usize;
            let y = p.y as usize;
            if x < cols && y < rows {
                let idx = y * cols + x;
                if idx < grid.len() {
                    // Heat t maps life -> temperature
                    let t = (1.0 - p.life.clamp(0.0, 1.5) / 1.5).clamp(0.0, 1.0);
                    let (r, g, b) = resolve_color(self.palette, t);
                    let ch = match self.style {
                        Style::Flared if t > 0.7 => '*',
                        _ => p.ch,
                    };
                    grid[idx] = TerminalCell {
                        ch,
                        fg: (r, g, b),
                        bg: (0, 0, 0),
                        bold: true,
                    };
                }
            }
        }
    }
}

impl crate::ui::screensaver_renderer::Screensaver for RisingFlames {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if self.active {
            RisingFlames::draw(self, grid, cols, rows);
        }
    }
}
