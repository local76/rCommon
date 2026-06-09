use crate::core::TerminalCell;
use crate::role::application::palette::query_current_palette;
use super::effect::FireEffect;

pub fn draw_fire(effect: &FireEffect, grid: &mut [TerminalCell], cols: usize, rows: usize) {
    const CHARS: &[char] = &[
        ' ', '.', ':', '-', '=', '+', '*', 'o', 's', 'x', 'z', '#', 'A', '@', '█'
    ];

    // Clear the grid to blank black cells first
    for cell in grid.iter_mut() {
        *cell = TerminalCell {
            ch: ' ',
            fg: (0, 0, 0),
            bg: (0, 0, 0),
            bold: false,
        };
    }

    // Find top candidates for lens flares (only highly excited stars, max 4)
    let mut flare_candidates: Vec<(usize, f32)> = effect.stars.iter()
        .enumerate()
        .filter(|(_, star)| star.excitation > 0.8)
        .map(|(idx, star)| (idx, star.excitation))
        .collect();
    flare_candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
    let allowed_flares: Vec<usize> = flare_candidates.iter()
        .take(4)
        .map(|&(idx, _)| idx)
        .collect();

    // 1. Draw background stars & lens flares (illuminated and excited by sparks)
    for (i, star) in effect.stars.iter().enumerate() {
        let sx = (star.x * cols as f32) as usize;
        let sy = (star.y * rows as f32) as usize;
        if sx < cols && sy < rows {
            // Only draw if there is no fire at this location
            if effect.fire_grid[sy * cols + sx] == 0 {
                // Base twinkle brightness
                let sparkle_base = ((effect.time_elapsed * 2.0 + star.phase).sin() + 1.0) * 0.5;
                let sparkle = (sparkle_base + star.excitation).min(2.0);
                let base_brightness = (sparkle_base * 120.0 + 40.0) as u8;

                let mut r = base_brightness;
                let mut g = base_brightness;
                let mut b = base_brightness.saturating_add(25);

                if star.excitation > 0.05 {
                    let blend = (star.excitation * 0.7).min(1.0);
                    r = (r as f32 * (1.0 - blend) + star.excited_color.0 as f32 * blend).min(255.0) as u8;
                    g = (g as f32 * (1.0 - blend) + star.excited_color.1 as f32 * blend).min(255.0) as u8;
                    b = (b as f32 * (1.0 - blend) + star.excited_color.2 as f32 * blend).min(255.0) as u8;
                }

                let final_brightness = sparkle * 0.4;

                let ch = if final_brightness > 0.8 {
                    '✹'
                } else if final_brightness > 0.5 {
                    '✦'
                } else {
                    star.ch
                };

                grid[sy * cols + sx] = TerminalCell {
                    ch,
                    fg: (r, g, b),
                    bg: (0, 0, 0),
                    bold: final_brightness > 0.6 || star.excitation > 0.3,
                };

                // Draw lens flares and starbursts on highly excited stars
                let is_excited = allowed_flares.contains(&i);
                if is_excited {
                    let flare_intensity = ((star.excitation - 0.8) / 0.7 + 0.5).min(1.5);
                    let flare_color = star.excited_color;

                    // Draw horizontal flare (cinematic anamorphic streak, longer)
                    let h_len = 12;
                    for dx in 1..h_len {
                        let alpha = (120.0f32 * flare_intensity).max(30.0f32) as u8;
                        let fade = alpha.saturating_sub((dx * (110 / h_len)) as u8);
                        if fade > 10 {
                            if sx + dx < cols {
                                let cell = &mut grid[sy * cols + (sx + dx)];
                                if effect.fire_grid[sy * cols + (sx + dx)] == 0 && (cell.ch == ' ' || cell.ch == '─') {
                                    cell.ch = '─';
                                    let fg_r = fade.saturating_add((flare_color.0 as f32 * 0.8) as u8);
                                    let fg_g = ((fade as f32 * 0.75) as u8).saturating_add((flare_color.1 as f32 * 0.8) as u8);
                                    let fg_b = (fade.saturating_add(45)).saturating_add((flare_color.2 as f32 * 0.8) as u8);
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx >= dx {
                                let cell = &mut grid[sy * cols + (sx - dx)];
                                if effect.fire_grid[sy * cols + (sx - dx)] == 0 && (cell.ch == ' ' || cell.ch == '─') {
                                    cell.ch = '─';
                                    let fg_r = fade.saturating_add((flare_color.0 as f32 * 0.8) as u8);
                                    let fg_g = ((fade as f32 * 0.75) as u8).saturating_add((flare_color.1 as f32 * 0.8) as u8);
                                    let fg_b = (fade.saturating_add(45)).saturating_add((flare_color.2 as f32 * 0.8) as u8);
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                        }
                    }

                    // Draw vertical flare
                    let v_len = 5;
                    for dy in 1..v_len {
                        let alpha = (90.0f32 * flare_intensity).max(20.0f32) as u8;
                        let fade = alpha.saturating_sub((dy * (80 / v_len)) as u8);
                        if fade > 10 {
                            if sy + dy < rows {
                                let cell = &mut grid[(sy + dy) * cols + sx];
                                if effect.fire_grid[(sy + dy) * cols + sx] == 0 && (cell.ch == ' ' || cell.ch == '│') {
                                    cell.ch = '│';
                                    let fg_r = fade.saturating_add((flare_color.0 as f32 * 0.8) as u8);
                                    let fg_g = ((fade as f32 * 0.75) as u8).saturating_add((flare_color.1 as f32 * 0.8) as u8);
                                    let fg_b = (fade.saturating_add(30)).saturating_add((flare_color.2 as f32 * 0.8) as u8);
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sy >= dy {
                                let cell = &mut grid[(sy - dy) * cols + sx];
                                if effect.fire_grid[(sy - dy) * cols + sx] == 0 && (cell.ch == ' ' || cell.ch == '│') {
                                    cell.ch = '│';
                                    let fg_r = fade.saturating_add((flare_color.0 as f32 * 0.8) as u8);
                                    let fg_g = ((fade as f32 * 0.75) as u8).saturating_add((flare_color.1 as f32 * 0.8) as u8);
                                    let fg_b = (fade.saturating_add(30)).saturating_add((flare_color.2 as f32 * 0.8) as u8);
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Render fire grid (overlays stars/flares where fire_val > 0)
    for y in 0..rows {
        for x in 0..cols {
            let mut fire_val = effect.fire_grid[y * cols + x] as usize;
            if fire_val > 0 {
                fire_val = fire_val.min(35);
                let char_idx = (fire_val * (CHARS.len() - 1)) / 35;
                let ch = CHARS[char_idx];
                let fg = effect.palette[fire_val];

                grid[y * cols + x] = TerminalCell {
                    ch,
                    fg,
                    bg: (0, 0, 0),
                    bold: fire_val > 14,
                };
            }
        }
    }

    // 3. Overlay rising sparks
    for spark in &effect.sparks {
        let sx = spark.x.round() as i32;
        let sy = spark.y.round() as i32;
        if sx >= 0 && sx < cols as i32 && sy >= 0 && sy < rows as i32 {
            let ux = sx as usize;
            let uy = sy as usize;
            let grid_idx = uy * cols + ux;

            let life_pct = spark.life / spark.max_life;
            let ch = if life_pct > 0.72 {
                '*'
            } else if life_pct > 0.32 {
                '+'
            } else {
                '.'
            };

            let color = if life_pct > 0.75 {
                let t = (life_pct - 0.75) / 0.25;
                (
                    255,
                    (180.0 + 75.0 * t) as u8,
                    (120.0 * t) as u8,
                )
            } else if life_pct > 0.35 {
                let t = (life_pct - 0.35) / 0.40;
                (
                    (180.0 + 75.0 * t) as u8,
                    (t * 180.0) as u8,
                    0,
                )
            } else {
                let t = life_pct / 0.35;
                (
                    (180.0 * t) as u8,
                    0,
                    0,
                )
            };

            let current = &mut grid[grid_idx];
            let current_fire_val = effect.fire_grid[grid_idx];
            if current_fire_val < 10 {
                current.ch = ch;
                current.fg = color;
                current.bold = life_pct > 0.45;
            }
        }
    }

    // 3.5. Overlay active volcanic globs (100% larger with core and envelope)
    for glob in &effect.volcanic_globs {
        let gx = glob.x.round() as i32;
        let gy = glob.y.round() as i32;
        
        let cells = [
            (gx, gy, '●', (255, 255, 200), true),      // Core
            (gx - 1, gy, 'o', (255, 130, 0), true),     // Left
            (gx + 1, gy, 'o', (255, 130, 0), true),     // Right
            (gx, gy - 1, 'o', (255, 130, 0), true),     // Top
            (gx, gy + 1, 'o', (255, 130, 0), true),     // Bottom
        ];

        for &(px, py, ch, fg, bold) in &cells {
            if px >= 0 && px < cols as i32 && py >= 0 && py < rows as i32 {
                let grid_idx = py as usize * cols + px as usize;
                grid[grid_idx] = TerminalCell {
                    ch,
                    fg,
                    bg: (0, 0, 0),
                    bold,
                };
            }
        }
    }

    // 4. Draw logo cells (styled with Windows Theme Accent color)
    for cell in &effect.logo_cells {
        let grid_idx = cell.y * cols + cell.x;
        let temp = cell.temp.min(1.0);

        // library 4.0: pull the accent per-frame from the canonical
        // ScreenPalette. Replaces the pre-4.0 `effect.theme_accent` field
        // so OS theme changes propagate without restarting the saver.
        let mut fg = query_current_palette().accent;
        if temp > 0.1 {
            let t = (temp - 0.1) / 0.9;
            fg.0 = (fg.0 as f32 * (1.0 - t) + 255.0 * t) as u8;
            fg.1 = (fg.1 as f32 * (1.0 - t) + 255.0 * t) as u8;
            fg.2 = (fg.2 as f32 * (1.0 - t) + 180.0 * t) as u8;
        }

        grid[grid_idx] = TerminalCell {
            ch: cell.ch,
            fg,
            bg: (0, 0, 0),
            bold: temp > 0.15,
        };
    }
}
