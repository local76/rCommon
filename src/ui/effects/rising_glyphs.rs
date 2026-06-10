//! RisingGlyphs — Verb (Rising) × Noun (Glyphs).
//!
//! Characters rising from a heat source at the bottom, fading as they ascend.
//! Like text drifting up off a fire. Default style: `Trailing`. Default palette: `Heat`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Palette, Speed, Style, resolve_color};

#[derive(Clone, Copy, Debug)]
pub struct RisingGlyph {
    pub x: f32,
    pub y: f32,
    pub ch: char,
    pub life: f32,
    pub max_life: f32,
}

/// Heat-source glyphs rising and fading.
pub struct RisingGlyphs {
    pub glyphs: Vec<RisingGlyph>,
    pub char_pool: Vec<char>,
    pub style: Style,
    pub palette: Palette,
    pub speed: Speed,
    pub density_setting: Density,
    // 4.0: reserved for future focus/active tracking
    #[allow(dead_code)]
    active: bool,
    #[allow(dead_code)]
    focused: bool,
    rng: LcgRng,
}

impl RisingGlyphs {
    /// Construct with default style (`Trailing`) and palette (`Heat`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(0xF1AE);
        let density_setting = Density::Normal;
        let count = ((((cols * rows / 4) as f32) * density_setting.multiplier()).max(20.0)) as usize;
        let char_pool: Vec<char> = "·*+~^°".chars().collect();
        let glyphs = (0..count).map(|_| Self::spawn(&mut rng, cols, rows, &char_pool)).collect();
        Self {
            glyphs,
            char_pool,
            style: Style::Trailing,
            palette: Palette::HEAT,
            speed: Speed::Normal,
            density_setting,
            active: true,
            focused: true,
            rng,
        }
    }

    fn spawn(rng: &mut LcgRng, cols: usize, rows: usize, pool: &[char]) -> RisingGlyph {
        let max_life = rng.next_range(1.5, 3.5);
        RisingGlyph {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range((rows as f32) * 0.7, rows as f32),
            ch: pool[rng.next_usize(pool.len())],
            life: max_life,
            max_life,
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

    pub fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        if !self.active {
            return;
        }
        let dt = dt.as_secs_f32();
        let m = self.speed.multiplier();
        for g in &mut self.glyphs {
            g.y -= dt * 6.0 * m;
            g.life -= dt;
            // Add slight horizontal drift
            if g.life <= 0.0 || g.y < 0.0 {
                *g = Self::spawn(&mut self.rng, cols, rows, &self.char_pool);
            }
        }
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
                    // t = 0 (hot, bottom) -> 1 (cool, top)
                    let t = (1.0 - g.life / g.max_life).clamp(0.0, 1.0);
                    let (r, g_c, b) = resolve_color(self.palette, t);
                    let intensity = match self.style {
                        Style::Trailing => (255.0 * (1.0 - t * 0.7)) as u8,
                        Style::Solid => 200,
                        Style::Flared => 255,
                    };
                    grid[idx] = TerminalCell {
                        ch: g.ch,
                        fg: (
                            ((r as u16 * intensity as u16) >> 8) as u8,
                            ((g_c as u16 * intensity as u16) >> 8) as u8,
                            ((b as u16 * intensity as u16) >> 8) as u8,
                        ),
                        bg: (0, 0, 0),
                        bold: t < 0.4,
                    };
                }
            }
        }
    }
}

impl crate::ui::screensaver_renderer::Screensaver for RisingGlyphs {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        RisingGlyphs::draw(self, grid, cols, rows);
    }
}
