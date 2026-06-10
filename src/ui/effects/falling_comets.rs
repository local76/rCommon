//! FallingComets — Verb (Falling) × Noun (Comets).
//!
//! Shooting-star streaks: long bright trails moving diagonally downward.
//! Default style: `Trailing`. Default palette: `Monochrome(White)`.
//!
//! Classification: Interface (TUI) + Role (Application).

use std::cell::RefCell;
use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Direction, Palette, Speed, Style, resolve_color};
use super::Particle;

/// Diagonal shooting-star streaks.
pub struct FallingComets {
    pub particles: Vec<Particle>,
    pub char_pool: Vec<char>,
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
    // Cached state mutated inside `draw`. RefCell lets `draw` stay `&self`
    // for the 4.0 unified ScreensaverEffect trait.
    last_drawn: RefCell<Vec<usize>>,
    last_cols: RefCell<usize>,
    last_rows: RefCell<usize>,
}

impl FallingComets {
    /// Construct with default style (`Trailing`) and palette (`Monochrome(White)`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(0xBEEF);
        let density_setting = Density::Normal;
        let count = ((((cols / 4).max(2)) as f32) * density_setting.multiplier()).max(2.0) as usize;
        let particles = (0..count).map(|_| Self::spawn(&mut rng, cols, rows)).collect();
        let char_pool: Vec<char> = "·*+x/\\-".chars().collect();
        Self {
            particles,
            char_pool,
            style: Style::Trailing,
            palette: Palette::WHITE,
            speed: Speed::Normal,
            direction: Direction::DiagonalDown,
            density_setting,
            active: true,
            focused: true,
            rng,
            last_drawn: RefCell::new(Vec::new()),
            last_cols: RefCell::new(cols),
            last_rows: RefCell::new(rows),
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

    fn spawn(rng: &mut LcgRng, cols: usize, rows: usize) -> Particle {
        Particle {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(0.0, (rows / 2) as f32),
            vx: rng.next_range(-2.0, 2.0),
            vy: rng.next_range(0.5, 2.0),
            ch: '*',
            color: (255, 255, 255),
            life: rng.next_range(1.0, 3.0),
        }
    }

    pub fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        if !self.active {
            return;
        }
        let dt = dt.as_secs_f32();
        let m = self.speed.multiplier();
        let (sx, sy) = self.direction.signs();
        for p in &mut self.particles {
            p.x += sx * p.vx.abs() * dt * 25.0 * m;
            p.y += sy * p.vy.abs() * dt * 25.0 * m;
            p.life -= dt;
            if p.life <= 0.0 || p.y > rows as f32 || p.y < 0.0 || p.x < 0.0 || p.x > cols as f32 {
                *p = Self::spawn(&mut self.rng, cols, rows);
            }
        }
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if !self.active {
            return;
        }
        {
            let lc = *self.last_cols.borrow();
            let lr = *self.last_rows.borrow();
            if cols != lc || rows != lr {
                self.last_drawn.borrow_mut().clear();
                *self.last_cols.borrow_mut() = cols;
                *self.last_rows.borrow_mut() = rows;
            }
        }
        for &idx in self.last_drawn.borrow().iter() {
            if idx < grid.len() {
                grid[idx] = TerminalCell::default();
            }
        }
        self.last_drawn.borrow_mut().clear();

        let trail_len = 8.0_f32;
        for p in &self.particles {
            let px = p.x;
            let py = p.y;
            let mag = (p.vx * p.vx + p.vy * p.vy).sqrt().max(0.1);
            let dx = p.vx / mag;
            let dy = p.vy / mag;
            for i in 0..(trail_len as usize) {
                let t = i as f32 / trail_len;
                let x = (px - dx * i as f32) as isize;
                let y = (py - dy * i as f32) as isize;
                if x >= 0 && y >= 0 && (x as usize) < cols && (y as usize) < rows {
                    let idx = y as usize * cols + x as usize;
                    if idx < grid.len() {
                        let intensity = (255.0 * (1.0 - t)) as u8;
                        let ch = if i == 0 {
                            match self.style {
                                Style::Flared => '+',
                                _ => '★',
                            }
                        } else {
                            self.char_pool[i % self.char_pool.len()]
                        };
                        let (r, g, b) = resolve_color(self.palette, t);
                        grid[idx] = TerminalCell {
                            ch,
                            fg: (
                                ((r as u16 * intensity as u16) >> 8) as u8,
                                ((g as u16 * intensity as u16) >> 8) as u8,
                                ((b as u16 * intensity as u16) >> 8) as u8,
                            ),
                            bg: (0, 0, 0),
                            bold: i < 2,
                        };
                        self.last_drawn.borrow_mut().push(idx);
                    }
                }
            }
        }
    }
}

impl crate::ui::screensaver_renderer::Screensaver for FallingComets {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        FallingComets::draw(self, grid, cols, rows);
    }
}
