//! PulledParticles — Verb (Pulled) × Noun (Particles).
//!
//! Particles attracted toward one or more gravity centers.
//! Default style: `Solid`. Default palette: `Monochrome(Blue)`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Palette, Speed, Style, resolve_color};
use super::Particle;

#[inline]
fn inv_sqrt(x: f32) -> f32 {
    x.sqrt().recip()
}

#[derive(Clone, Copy, Debug)]
pub struct GravityCenter {
    pub x: f32,
    pub y: f32,
    pub mass: f32,
}

/// Particles pulled toward gravity centers.
pub struct PulledParticles {
    pub particles: Vec<Particle>,
    pub gravity_centers: Vec<GravityCenter>,
    pub style: Style,
    pub palette: Palette,
    pub speed: Speed,
    pub density_setting: Density,
    // 4.0: reserved for future focus/active tracking
    #[allow(dead_code)]
    active: bool,
    #[allow(dead_code)]
    focused: bool,
    #[allow(dead_code)]
    rng: LcgRng,
}

impl PulledParticles {
    /// Construct with default style (`Solid`) and palette (`Monochrome(Blue)`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(999);
        let density_setting = Density::Normal;
        let count = ((((cols * rows) / 4) as f32) * density_setting.multiplier()).max(10.0) as usize;
        let particles = (0..count).map(|_| Particle {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(0.0, rows as f32),
            vx: rng.next_range(-0.5, 0.5),
            vy: rng.next_range(-0.5, 0.5),
            ch: '.',
            color: (100, 150, 255),
            life: 10.0,
        }).collect();
        let gravity_centers = vec![
            GravityCenter { x: (cols as f32) * 0.5, y: (rows as f32) * 0.5, mass: 5.0 },
        ];
        Self {
            particles,
            gravity_centers,
            style: Style::Solid,
            palette: Palette::BLUE,
            speed: Speed::Normal,
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
        for p in &mut self.particles {
            for gc in &self.gravity_centers {
                let dx = gc.x - p.x;
                let dy = gc.y - p.y;
                let sq_dist = dx * dx + dy * dy;
                let inv_dist = if sq_dist <= 1.0 {
                    1.0
                } else {
                    inv_sqrt(sq_dist)
                };
                p.vx += dx * inv_dist * gc.mass * dt * 0.1 * m;
                p.vy += dy * inv_dist * gc.mass * dt * 0.1 * m;
            }
            p.x += p.vx * dt * 20.0;
            p.y += p.vy * dt * 20.0;
        }
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        for p in &self.particles {
            let x = p.x as usize;
            let y = p.y as usize;
            if x < cols && y < rows {
                let idx = y * cols + x;
                if idx < grid.len() {
                    let speed = (p.vx * p.vx + p.vy * p.vy).sqrt();
                    let t = (speed / 4.0).clamp(0.0, 1.0);
                    let (r, g, b) = resolve_color(self.palette, t);
                    let ch = match self.style {
                        Style::Flared if speed > 2.0 => '+',
                        _ => p.ch,
                    };
                    grid[idx] = TerminalCell {
                        ch,
                        fg: (r, g, b),
                        bg: (0, 0, 0),
                        bold: false,
                    };
                }
            }
        }
    }
}

impl crate::interface::app::screensaver::Screensaver for PulledParticles {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if self.active {
            PulledParticles::draw(self, grid, cols, rows);
        }
    }
}
