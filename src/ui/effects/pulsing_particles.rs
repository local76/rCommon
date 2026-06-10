//! PulsingParticles — Verb (Pulsing) × Noun (Particles).
//!
//! Particles that pulse (scale/brighten) in place, no net motion.
//! Useful as ambient focus indicator or starfield-style twinkle.
//! Default style: `Solid`. Default palette: `Accent`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Palette, Speed, Style, resolve_color};
use super::Particle;

/// Particles pulsing in place.
pub struct PulsingParticles {
    pub particles: Vec<Particle>,
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

impl PulsingParticles {
    /// Construct with default style (`Solid`) and palette (`Accent`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(0xC0FFEE);
        let density_setting = Density::Normal;
        let count = ((((cols * rows / 5) as f32) * density_setting.multiplier()).max(15.0)) as usize;
        let particles = (0..count).map(|_| Particle {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(0.0, rows as f32),
            vx: 0.0,
            vy: 0.0,
            ch: '.',
            color: (200, 200, 200),
            life: rng.next_range(0.0, std::f32::consts::TAU),
        }).collect();
        Self {
            particles,
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
        for p in &self.particles {
            let x = p.x as usize;
            let y = p.y as usize;
            if x < cols && y < rows {
                let idx = y * cols + x;
                if idx < grid.len() {
                    let pulse = (self.time + p.life).sin() * 0.5 + 0.5;
                    let t = pulse;
                    let (r, g_c, b) = resolve_color(self.palette, t);
                    let intensity = (40.0 + pulse * 215.0) as u8;
                    let ch = match self.style {
                        Style::Flared if pulse > 0.85 => '*',
                        Style::Trailing => match (pulse * 4.0) as usize {
                            0 => '·',
                            1 => '.',
                            2 => 'o',
                            _ => 'O',
                        },
                        _ => p.ch,
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

impl crate::interface::app::screensaver::Screensaver for PulsingParticles {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        PulsingParticles::draw(self, grid, cols, rows);
    }
}
