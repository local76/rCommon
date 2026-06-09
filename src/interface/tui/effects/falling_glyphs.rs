//! FallingGlyphs — Verb (Falling) × Noun (Glyphs).
//!
//! Matrix-style falling characters with bright heads and fading trails.
//! Default style: `Trailing`. Default palette: `Monochrome(Green)`.
//!
//! Classification: Interface (TUI) + Role (Application).

use std::cell::RefCell;
use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Direction, Palette, Speed, Style, resolve_color};
use super::RainDrop;

/// Falling glyph streams with character trails.
pub struct FallingGlyphs {
    pub drops: Vec<RainDrop>,
    pub char_pool: Vec<char>,
    pub density: f32,
    /// Render treatment.
    pub style: Style,
    /// Color source.
    pub palette: Palette,
    /// Speed preset.
    pub speed: Speed,
    /// Motion direction.
    pub direction: Direction,
    /// Particle density.
    pub density_setting: Density,
    /// First-class active flag for focus/tab UIs.
    // 4.0: `active`/`focused` are reserved fields, currently inert
    // (ScreensaverState is a supertrait with default-true impls). They
    // are kept in the struct for future focus/active tracking without
    // a breaking 4.x schema change.
    #[allow(dead_code)]
    active: bool,
    #[allow(dead_code)]
    focused: bool,
    rng: LcgRng,
    // Cached state mutated inside `draw`. Wrapped in RefCell so `draw` can
    // remain `&self` (the 4.0 unified ScreensaverEffect trait requirement,
    // shared with trance-scenes GDI renderers).
    last_drawn: RefCell<Vec<usize>>,
    last_cols: RefCell<usize>,
    last_rows: RefCell<usize>,
}

impl FallingGlyphs {
    /// Construct with the default style (`Trailing`) and palette (`Monochrome(Green)`).
    pub fn new(cols: usize, rows: usize, density: f32) -> Self {
        let mut rng = LcgRng::new(0xC0FFEE);
        let mut drops = Vec::new();
        let num_drops = ((cols as f32) * density).max(1.0) as usize;

        for _ in 0..num_drops {
            drops.push(RainDrop {
                x: rng.next_usize(cols) as f32,
                y: rng.next_usize(rows) as f32,
                speed: rng.next_range(0.5, 2.5),
                length: rng.next_usize(8) + 3,
            });
        }

        let char_pool: Vec<char> = "ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾙﾚﾛﾜﾝ0123456789".chars().collect();

        Self {
            drops,
            char_pool,
            density,
            style: Style::Trailing,
            palette: Palette::GREEN,
            speed: Speed::Normal,
            direction: Direction::Down,
            density_setting: Density::Normal,
            active: true,
            focused: true,
            rng,
            last_drawn: RefCell::new(Vec::new()),
            last_cols: RefCell::new(cols),
            last_rows: RefCell::new(rows),
        }
    }

    /// Set the render style (builder).
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the color palette (builder).
    pub fn with_palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    /// Set the speed preset (builder).
    pub fn with_speed(mut self, speed: Speed) -> Self {
        self.speed = speed;
        self
    }

    /// Set the motion direction (builder).
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Set the particle density (builder).
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
            drop.x += sx * drop.speed * dt * 20.0 * m;
            drop.y += sy * drop.speed * dt * 20.0 * m;
            if drop.y > rows as f32 + drop.length as f32 {
                drop.y = - (drop.length as f32);
                drop.x = self.rng.next_usize(cols) as f32;
                drop.speed = self.rng.next_range(0.5, 2.5);
            }
        }
    }

    /// Render the falling glyphs into the provided grid.
    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if !self.active {
            return;
        }

        {
            let last_cols = *self.last_cols.borrow();
            let last_rows = *self.last_rows.borrow();
            if cols != last_cols || rows != last_rows {
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

        for drop in &self.drops {
            for i in 0..drop.length {
                let y = (drop.y - i as f32) as isize;
                if y >= 0 && y < rows as isize {
                    let x = drop.x as usize;
                    if x < cols {
                        let idx = (y as usize) * cols + x;
                        if idx < grid.len() {
                            let ch = if i == 0 {
                                '█'
                            } else {
                                let char_idx = (x * 17 + (y as usize + (drop.y * 0.1) as usize) * 31) % self.char_pool.len();
                                self.char_pool[char_idx]
                            };
                            let intensity = match self.style {
                                Style::Solid => 255,
                                Style::Trailing => 255 - (i as u8 * 20).min(200),
                                Style::Flared if i == 0 => 255,
                                Style::Flared => 255 - (i as u8 * 30).min(220),
                            };
                            // Heat-map t = 0 at head, climbs to 1 at tail
                            let t = (i as f32 / drop.length.max(1) as f32).clamp(0.0, 1.0);
                            let (r, g, b) = resolve_color(self.palette, t);
                            grid[idx] = TerminalCell {
                                ch,
                                fg: (
                                    ((r as u16 * intensity as u16) >> 8) as u8,
                                    ((g as u16 * intensity as u16) >> 8) as u8,
                                    ((b as u16 * intensity as u16) >> 8) as u8,
                                ),
                                bg: (0, 0, 0),
                                bold: i < 3,
                            };
                            self.last_drawn.borrow_mut().push(idx);
                        }
                    }
                }
            }
        }
    }
}

impl crate::interface::tui::screensaver::Screensaver for FallingGlyphs {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows, self.density);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        FallingGlyphs::draw(self, grid, cols, rows);
    }
}
