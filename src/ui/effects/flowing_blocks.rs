//! FlowingBlocks — Verb (Flowing) × Noun (Blocks).
//!
//! Tetris-style block shapes drifting horizontally across the screen.
//! Default style: `Solid`. Default palette: `Accent`.
//!
//! Classification: Interface (TUI) + Role (Application).

use crate::core::{LcgRng, TerminalCell};
use super::dimensions::{Density, Direction, Palette, Speed, Style, resolve_color};

/// Tetromino shape definitions, indexed into SHAPES.
pub const SHAPES: &[&[(i8, i8)]] = &[
    // I
    &[(-1, 0), (0, 0), (1, 0), (2, 0)],
    // O
    &[(-1, -1), (0, -1), (-1, 0), (0, 0)],
    // T
    &[(-1, 0), (0, 0), (1, 0), (0, -1)],
    // L
    &[(-1, 0), (0, 0), (1, 0), (1, -1)],
    // S
    &[(-1, 0), (0, 0), (0, -1), (1, -1)],
    // Z
    &[(-1, -1), (0, -1), (0, 0), (1, 0)],
    // single 2x2
    &[(-1, -1), (0, -1), (-1, 0), (0, 0)],
];

#[derive(Clone, Copy, Debug)]
pub struct FlowingBlock {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub shape_idx: usize,
    pub ch: char,
}

/// Tetromino-style blocks flowing horizontally.
pub struct FlowingBlocks {
    pub blocks: Vec<FlowingBlock>,
    pub shape: &'static [&'static [(i8, i8)]],
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
    #[allow(dead_code)]
    rng: LcgRng,
}

impl FlowingBlocks {
    /// Construct with default style (`Solid`) and palette (`Accent`).
    pub fn new(cols: usize, rows: usize) -> Self {
        let mut rng = LcgRng::new(0xB10C);
        let density_setting = Density::Normal;
        let count = ((((cols / 6).max(2)) as f32) * density_setting.multiplier()).max(2.0) as usize;
        let pool = ['█', '▓', '▒', '■'];
        let blocks = (0..count).map(|_| FlowingBlock {
            x: rng.next_range(0.0, cols as f32),
            y: rng.next_range(1.0, (rows - 1) as f32),
            vx: rng.next_range(0.4, 1.2) * if rng.next_bool(0.5) { 1.0 } else { -1.0 },
            shape_idx: rng.next_usize(SHAPES.len()),
            ch: pool[rng.next_usize(pool.len())],
        }).collect();
        Self {
            blocks,
            shape: SHAPES,
            style: Style::Solid,
            palette: Palette::ACCENT,
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

    pub fn update(&mut self, dt: std::time::Duration, cols: usize, _rows: usize) {
        if !self.active {
            return;
        }
        let dt = dt.as_secs_f32();
        let m = self.speed.multiplier();
        for b in &mut self.blocks {
            b.x += b.vx * dt * 8.0 * m;
            let shape = self.shape[b.shape_idx];
            let max_dx = shape.iter().map(|(dx, _)| *dx as f32).fold(0.0_f32, f32::max);
            let min_dx = shape.iter().map(|(dx, _)| *dx as f32).fold(0.0_f32, f32::min);
            if b.x + max_dx >= cols as f32 {
                b.x = -max_dx;
            } else if b.x + min_dx < 0.0 {
                b.x = cols as f32 - max_dx;
            }
        }
    }

    pub fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        if !self.active {
            return;
        }
        for b in &self.blocks {
            let shape = self.shape[b.shape_idx];
            for (dx, dy) in shape {
                let x = (b.x + *dx as f32) as isize;
                let y = (b.y + *dy as f32) as isize;
                if x >= 0 && y >= 0 && (x as usize) < cols && (y as usize) < rows {
                    let idx = y as usize * cols + x as usize;
                    if idx < grid.len() {
                        let t = (b.x / cols.max(1) as f32).clamp(0.0, 1.0);
                        let (r, g, b_c) = resolve_color(self.palette, t);
                        let ch = match self.style {
                            Style::Flared if *dx == 0 && *dy == 0 => '◆',
                            _ => b.ch,
                        };
                        grid[idx] = TerminalCell {
                            ch,
                            fg: (r, g, b_c),
                            bg: (0, 0, 0),
                            bold: true,
                        };
                    }
                }
            }
        }
    }
}

impl crate::interface::app::screensaver::Screensaver for FlowingBlocks {
    fn init(&mut self, cols: usize, rows: usize) {
        *self = Self::new(cols, rows);
    }
    fn update(&mut self, dt: std::time::Duration, cols: usize, rows: usize) {
        self.update(dt, cols, rows);
    }
    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        FlowingBlocks::draw(self, grid, cols, rows);
    }
}
