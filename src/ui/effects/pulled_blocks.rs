//! PulledBlocks — Verb (Pulled) × Noun (Blocks).
//!
//! Small block shapes pulled toward gravity centers. Combines
//! `PulledParticles` motion with `Blocks` rendering.
//! Default style: `Solid`. Default palette: `Monochrome(Blue)`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Palette, Speed, Style, resolve_color};
use super::flowing_blocks::SHAPES;

#[inline]
fn inv_sqrt(x: f32) -> f32 {
    x.sqrt().recip()
}

#[derive(Clone, Copy, Debug)]
pub struct BlockParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ch: char,
    pub shape_idx: usize,
    pub life: f32,
}

/// Block shapes pulled toward gravity centers.
pub struct PulledBlocks {
    pub particles: Vec<BlockParticle>,
    pub gravity_centers: Vec<super::pulled_particles::GravityCenter>,
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

impl PulledBlocks {
    /// Construct with default style (`Solid`) and palette (`Monochrome(Blue)`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(0xB10C5);
        let density_setting = Density::Normal;
        let count = ((((cols * rows) / 8) as f32) * density_setting.multiplier()).max(8.0) as usize;
        let pool = ['█', '▓', '▒'];
        let particles = (0..count).map(|_| BlockParticle {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(0.0, rows as f32),
            vx: rng.next_range(-0.5, 0.5),
            vy: rng.next_range(-0.5, 0.5),
            ch: pool[rng.next_usize(pool.len())],
            shape_idx: rng.next_usize(SHAPES.len()),
            life: 10.0,
        }).collect();
        let gravity_centers = vec![
            super::pulled_particles::GravityCenter {
                x: (cols as f32) * 0.5,
                y: (rows as f32) * 0.5,
                mass: 5.0,
            },
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
                let inv_dist = if sq_dist <= 1.0 { 1.0 } else { inv_sqrt(sq_dist) };
                p.vx += dx * inv_dist * gc.mass * dt * 0.1 * m;
                p.vy += dy * inv_dist * gc.mass * dt * 0.1 * m;
            }
            p.x += p.vx * dt * 20.0;
            p.y += p.vy * dt * 20.0;
        }
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        for p in &self.particles {
            let shape = SHAPES[p.shape_idx];
            for (dx, dy) in shape {
                let x = (p.x + *dx as f32) as isize;
                let y = (p.y + *dy as f32) as isize;
                if x >= 0 && y >= 0 && (x as usize) < cols && (y as usize) < rows {
                    let idx = y as usize * cols + x as usize;
                    if idx < grid.len() {
                        let speed_v = (p.vx * p.vx + p.vy * p.vy).sqrt();
                        let t = (speed_v / 4.0).clamp(0.0, 1.0);
                        let (r, g, b) = resolve_color(self.palette, t);
                        let ch = match self.style {
                            Style::Flared if speed_v > 2.0 => '+',
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
}

impl crate::interface::app::screensaver::Screensaver for PulledBlocks {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if self.active {
            PulledBlocks::draw(self, grid, cols, rows);
        }
    }
}
