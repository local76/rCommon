use crate::core::TerminalCell;
use crate::role::application::palette::query_current_palette;
use super::state::Glyphs;

impl Glyphs {
    pub fn draw_impl(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        // library 4.0: pull the canonical accent color from ScreenPalette.
        let accent = query_current_palette().accent;

        for d in self.drops.iter() {
            let x = d.x as i32;
            if x < 0 || x as usize >= cols {
                continue;
            }
            for k in 0..d.length {
                let y = d.y as i32 - k as i32;
                if y < 0 || y as usize >= rows {
                    continue;
                }
                let idx = (y as usize) * cols + x as usize;
                if idx >= grid.len() {
                    break;
                }
                let pool = if !self.live_system_chars.is_empty() && k == 0 {
                    &self.live_system_chars
                } else {
                    &self.char_pool
                };
                let ch = pool[(d.char_rot + k) % pool.len().max(1)];
                let intensity = 1.0 - (k as f32 / d.length as f32);
                let g = ((accent.1 as f32) * (0.4 + 0.6 * intensity)) as u8;
                let b = ((accent.2 as f32) * (0.4 + 0.6 * intensity)) as u8;
                let r = ((accent.0 as f32) * (0.4 + 0.6 * intensity)) as u8;
                grid[idx] = TerminalCell {
                    ch,
                    fg: (r, g, b),
                    bg: (0, 0, 0),
                    bold: k == 0,
                };
            }
        }
    }
}
