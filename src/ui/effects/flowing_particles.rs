//! FlowingParticles — Verb (Flowing) × Noun (Particles).
//!
//! Ambient particles drifting in arbitrary directions with downward gravity.
//! Default style: `Solid`. Default palette: `Monochrome(White)`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Direction, Palette, Speed, Style, resolve_color};
use super::Particle;

/// Drifting particles with gravity.
pub struct FlowingParticles {
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

impl FlowingParticles {
    /// Construct with default style (`Solid`) and palette (`Monochrome(White)`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(12345);
        let density_setting = Density::Normal;
        let count = ((((cols * rows) / 3) as f32) * density_setting.multiplier()).max(10.0) as usize;
        let particles = (0..count).map(|_| Particle {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(0.0, rows as f32),
            vx: rng.next_range(-1.0, 1.0),
            vy: rng.next_range(-2.0, -0.5),
            ch: '*',
            color: (255, 200, 50),
            life: rng.next_range(0.5, 2.0),
        }).collect();

        Self {
            particles,
            style: Style::Solid,
            palette: Palette::WHITE,
            speed: Speed::Normal,
            direction: Direction::Right,
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
            p.x += p.vx * dt * 30.0 * m;
            p.y += p.vy * dt * 30.0 * m;
            p.life -= dt;
            p.vy += 1.5 * dt * m;
            if p.life <= 0.0 {
                p.x = self.rng.next_range(0.0, cols as f32);
                p.y = self.rng.next_range(0.0, rows as f32);
                p.vx = self.rng.next_range(-1.0, 1.0);
                p.vy = self.rng.next_range(-2.0, -0.5);
                p.life = self.rng.next_range(0.5, 2.0);
            }
        }
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        for p in &self.particles {
            let x = p.x as usize;
            let y = p.y as usize;
            if x < cols && y < rows {
                let idx = y * cols + x;
                if idx < grid.len() && p.life > 0.0 {
                    let alpha = (p.life * 200.0) as u8;
                    let t = (1.0 - p.life.clamp(0.0, 1.0)).clamp(0.0, 1.0);
                    let (r, g, b) = resolve_color(self.palette, t);
                    let (br, bg, bb) = match self.style {
                        Style::Flared if p.life > 1.5 => (255, 255, 255),
                        _ => (r, g, b),
                    };
                    grid[idx] = TerminalCell {
                        ch: p.ch,
                        fg: (
                            br.min(alpha),
                            bg.min(alpha),
                            bb.min(alpha),
                        ),
                        bg: (0, 0, 0),
                        bold: true,
                    };
                }
            }
        }
    }
}

impl crate::ui::screensaver_renderer::Screensaver for FlowingParticles {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if self.active {
            FlowingParticles::draw(self, grid, cols, rows);
        }
    }
}
