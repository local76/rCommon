use crate::core::TerminalCell;
use crate::core::logo_block::render_logo_block;
use crate::platform::native::sys_info::get_system_info;
use super::effect::Fireworks;
use super::types::ActiveExplosion;

impl Fireworks {
    pub fn draw_impl(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        // Collect active explosions to light up the logo and buildings
        let mut active_explosions = Vec::new();
        for p in &self.particles {
            if p.color != (100, 100, 100) {
                let pct = p.life / p.max_life;
                if pct > 0.4 {
                    active_explosions.push(ActiveExplosion {
                        x: p.x,
                        y: p.y,
                        radius: 11.0 * pct,
                        color: p.color,
                        intensity: pct,
                    });
                }
            }
        }

        // Find top candidates for lens flares (only highly excited stars, max 4)
        let mut flare_candidates: Vec<(usize, f32)> = self.stars.iter()
            .enumerate()
            .filter(|(_, star)| star.excitation > 0.8)
            .map(|(idx, star)| (idx, star.excitation))
            .collect();
        flare_candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        let allowed_flares: Vec<usize> = flare_candidates.iter()
            .take(4)
            .map(|&(idx, _)| idx)
            .collect();

        // Draw background stars (illuminated by active explosions and excited by sparks)
        for (i, star) in self.stars.iter().enumerate() {
            let sx = (star.x * cols as f32) as usize;
            let sy = (star.y * rows as f32) as usize;
            if sx < cols && sy < rows {
                // Only draw if skyward of the skyline profile
                let height_from_bottom = rows.saturating_sub(1).saturating_sub(sy);
                if height_from_bottom >= self.skyline[sx] {
                    // Check light from active explosions
                    let mut best_intensity = 0.0f32;
                    let mut lit_color = (0, 0, 0);

                    for exp in &active_explosions {
                        let dx = (sx as f32 - exp.x) * 0.55;
                        let dy = sy as f32 - exp.y;
                        let dist = (dx*dx + dy*dy).sqrt();
                        if dist < exp.radius {
                            let intensity = (1.0 - dist / exp.radius) * exp.intensity;
                            if intensity > best_intensity {
                                best_intensity = intensity;
                                lit_color = (
                                    (exp.color.0 as f32 * intensity) as u8,
                                    (exp.color.1 as f32 * intensity) as u8,
                                    (exp.color.2 as f32 * intensity) as u8,
                                );
                            }
                        }
                    }

                    // Base twinkle brightness
                    let sparkle_base = ((self.time_elapsed * 2.0 + star.phase).sin() + 1.0) * 0.5;
                    let sparkle = (sparkle_base + star.excitation).min(2.0);
                    let base_brightness = (sparkle_base * 120.0 + 40.0) as u8;

                    // Blend base color (dim white) with explosion/excited color
                    let mut r = base_brightness.saturating_add((lit_color.0 as f32 * 0.8) as u8);
                    let mut g = base_brightness.saturating_add((lit_color.1 as f32 * 0.8) as u8);
                    let mut b = (base_brightness.saturating_add(25)).saturating_add((lit_color.2 as f32 * 0.8) as u8);

                    if star.excitation > 0.05 {
                        let blend = (star.excitation * 0.7).min(1.0);
                        r = (r as f32 * (1.0 - blend) + star.excited_color.0 as f32 * blend).min(255.0) as u8;
                        g = (g as f32 * (1.0 - blend) + star.excited_color.1 as f32 * blend).min(255.0) as u8;
                        b = (b as f32 * (1.0 - blend) + star.excited_color.2 as f32 * blend).min(255.0) as u8;
                    }

                    let final_brightness = sparkle * 0.4 + best_intensity * 0.6;

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

                    // Draw lens flares and starbursts on highly illuminated/excited stars
                    let is_excited = allowed_flares.contains(&i);
                    if is_excited {
                        let flare_intensity = ((star.excitation - 0.8) / 0.7 + 0.5).min(1.5);
                        let flare_color = star.excited_color;

                        // Draw horizontal flare (cinematic anamorphic streak, longer)
                        let h_len = 12;
                        for dx in 1..h_len {
                            let alpha = (120.0 * flare_intensity).max(30.0) as u8;
                            let fade = alpha.saturating_sub((dx * (110 / h_len)) as u8);
                            if fade > 10 {
                                if sx + dx < cols {
                                    let cell = &mut grid[sy * cols + (sx + dx)];
                                    let h_test = rows.saturating_sub(1).saturating_sub(sy);
                                    if h_test >= self.skyline[sx + dx] && (cell.ch == ' ' || cell.ch == '─') {
                                        cell.ch = '─';
                                        let fg_r = fade.saturating_add((flare_color.0 as f32 * 0.8) as u8);
                                        let fg_g = ((fade as f32 * 0.75) as u8).saturating_add((flare_color.1 as f32 * 0.8) as u8);
                                        let fg_b = (fade.saturating_add(45)).saturating_add((flare_color.2 as f32 * 0.8) as u8);
                                        cell.fg = (fg_r, fg_g, fg_b);
                                    }
                                }
                                if sx >= dx {
                                    let cell = &mut grid[sy * cols + (sx - dx)];
                                    let h_test = rows.saturating_sub(1).saturating_sub(sy);
                                    if h_test >= self.skyline[sx - dx] && (cell.ch == ' ' || cell.ch == '─') {
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
                            let alpha = (90.0 * flare_intensity).max(20.0) as u8;
                            let fade = alpha.saturating_sub((dy * (80 / v_len)) as u8);
                            if fade > 10 {
                                if sy + dy < rows {
                                    let cell = &mut grid[(sy + dy) * cols + sx];
                                    let h_test = rows.saturating_sub(1).saturating_sub(sy + dy);
                                    if h_test >= self.skyline[sx] && (cell.ch == ' ' || cell.ch == '│') {
                                        cell.ch = '│';
                                        let fg_r = fade.saturating_add((flare_color.0 as f32 * 0.8) as u8);
                                        let fg_g = ((fade as f32 * 0.75) as u8).saturating_add((flare_color.1 as f32 * 0.8) as u8);
                                        let fg_b = (fade.saturating_add(30)).saturating_add((flare_color.2 as f32 * 0.8) as u8);
                                        cell.fg = (fg_r, fg_g, fg_b);
                                    }
                                }
                                if sy >= dy {
                                    let cell = &mut grid[(sy - dy) * cols + sx];
                                    let h_test = rows.saturating_sub(1).saturating_sub(sy - dy);
                                    if h_test >= self.skyline[sx] && (cell.ch == ' ' || cell.ch == '│') {
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

        // library 4.1: render the centered system logo from the live OS info
        // (replaces pre-4.1 `trance_core::logo_lines()` + `logo_dimensions()`).
        let logo_text = get_system_info().logo_text;
        let lines = render_logo_block(&logo_text, None);
        let logo_h = lines.len();
        let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let logo_x = cols.saturating_sub(logo_w) / 2;
        let logo_y = rows.saturating_sub(logo_h) / 2;

        for (r_offset, line) in lines.iter().enumerate().take(logo_h) {
            let gy = logo_y + r_offset;
            if gy >= rows { continue; }
            for (c_offset, ch) in line.chars().enumerate() {
                let gx = logo_x + c_offset;
                if gx >= cols { continue; }
                if ch != ' ' {
                    // Check light from active explosions
                    let mut best_intensity = 0.0f32;
                    let mut lit_color = (35, 15, 50); // dim default silhouette color

                    for exp in &active_explosions {
                        let dx = (gx as f32 - exp.x) * 0.55;
                        let dy = gy as f32 - exp.y;
                        let dist = (dx*dx + dy*dy).sqrt();
                        if dist < exp.radius {
                            let intensity = (1.0 - dist / exp.radius) * exp.intensity;
                            if intensity > best_intensity {
                                best_intensity = intensity;
                                lit_color = (
                                    (exp.color.0 as f32 * intensity + 35.0 * (1.0 - intensity)) as u8,
                                    (exp.color.1 as f32 * intensity + 15.0 * (1.0 - intensity)) as u8,
                                    (exp.color.2 as f32 * intensity + 50.0 * (1.0 - intensity)) as u8,
                                );
                            }
                        }
                    }

                    grid[gy * cols + gx] = TerminalCell {
                        ch,
                        fg: lit_color,
                        bg: (0, 0, 0),
                        bold: best_intensity > 0.1,
                    };
                }
            }
        }

        // 2. Draw rising rockets
        for rocket in &self.rockets {
            let cx = rocket.x as usize;
            let cy = rocket.y as usize;
            if cx < cols && cy < rows {
                grid[cy * cols + cx] = TerminalCell {
                    ch: '▲',
                    fg: (255, 255, 255),
                    bg: (0, 0, 0),
                    bold: true,
                };
            }
        }

        // 3. Draw explosion particles
        for p in &self.particles {
            let cx = p.x as usize;
            let cy = p.y as usize;
            if cx < cols && cy < rows {
                let pct = p.life / p.max_life;
                let color = (
                    (p.color.0 as f32 * pct) as u8,
                    (p.color.1 as f32 * pct) as u8,
                    (p.color.2 as f32 * pct) as u8,
                );
                
                // Only draw if skyward of the skyline profile (except smoke)
                let is_smoke = p.color == (100, 100, 100);
                let height_from_bottom = rows.saturating_sub(1).saturating_sub(cy);
                if height_from_bottom >= self.skyline[cx] || is_smoke {
                    // Only overwrite empty space or flares (except if smoke)
                    let current_ch = grid[cy * cols + cx].ch;
                    if is_smoke || current_ch == ' ' || current_ch == '─' || current_ch == '│' || current_ch == '/' || current_ch == '\\' {
                        grid[cy * cols + cx] = TerminalCell {
                            ch: p.ch,
                            fg: color,
                            bg: (0, 0, 0),
                            bold: pct > 0.5,
                        };
                    }
                }
            }
        }

        // 4. Draw city skyline (with building windows reacting to nearby explosions)
        for c in 0..cols {
            let building_h = self.skyline[c];
            for r in 0..building_h {
                let gy = rows.saturating_sub(1).saturating_sub(r);
                let idx = gy * cols + c;

                let mut ch = if self.skyline_windows[idx] {
                    '■'
                } else {
                    ' '
                };
                let mut fg = if self.skyline_windows[idx] {
                    (255, 220, 100)
                } else {
                    (0, 0, 0)
                };

                // Let windows dynamically reflect the color of nearby explosions
                let mut best_intensity = 0.0f32;
                let mut glow_color = (0, 0, 0);

                for exp in &active_explosions {
                    let dx = (c as f32 - exp.x) * 0.55;
                    let dy = gy as f32 - exp.y;
                    let dist = (dx*dx + dy*dy).sqrt();
                    let glow_radius = exp.radius * 2.2;
                    if dist < glow_radius {
                        let intensity = (1.0 - dist / glow_radius) * exp.intensity * 0.95;
                        if intensity > best_intensity {
                            best_intensity = intensity;
                            glow_color = exp.color;
                        }
                    }
                }

                if best_intensity > 0.05 {
                    if self.skyline_windows[idx] {
                        // Lit windows shift color based on the explosion's flash
                        fg = (
                            (fg.0 as f32 * (1.0 - best_intensity) + glow_color.0 as f32 * best_intensity).min(255.0) as u8,
                            (fg.1 as f32 * (1.0 - best_intensity) + glow_color.1 as f32 * best_intensity).min(255.0) as u8,
                            (fg.2 as f32 * (1.0 - best_intensity) + glow_color.2 as f32 * best_intensity).min(255.0) as u8,
                        );
                    } else {
                        // Unlit windows illuminate temporarily as the explosion flashes over them!
                        ch = '■';
                        fg = (
                            (glow_color.0 as f32 * best_intensity * 0.38) as u8,
                            (glow_color.1 as f32 * best_intensity * 0.38) as u8,
                            (glow_color.2 as f32 * best_intensity * 0.38) as u8,
                        );
                    }
                }

                grid[idx] = TerminalCell {
                    ch,
                    fg,
                    bg: (15, 15, 22), // deep dark skyline gray-blue
                    bold: best_intensity > 0.2,
                };
            }
        }

        // 5. Draw overlay cinematic lens flares and starbursts centered at active explosion origins
        let mut drawn_explosion_flares: Vec<(f32, f32)> = Vec::new();
        for p in &self.particles {
            if p.color != (100, 100, 100) {
                let pct = p.life / p.max_life;
                if pct > 0.85 {
                    let ex = p.x;
                    let ey = p.y;
                    
                    let mut too_close = false;
                    for &(dx, dy) in &drawn_explosion_flares {
                        let dist = ((ex - dx)*0.55).hypot(ey - dy);
                        if dist < 5.0 {
                            too_close = true;
                            break;
                        }
                    }
                    if too_close { continue; }
                    drawn_explosion_flares.push((ex, ey));

                    let sx = ex as usize;
                    let sy = ey as usize;
                    if sx < cols && sy < rows {
                        let flare_intensity = (pct - 0.85) / 0.15;
                        let (er, eg, eb) = p.color;

                        let center_idx = sy * cols + sx;
                        grid[center_idx] = TerminalCell {
                            ch: '✸',
                            fg: (255, 255, 255),
                            bg: grid[center_idx].bg,
                            bold: true,
                        };

                        // Draw horizontal streak
                        let h_len = 16;
                        for dx in 1..h_len {
                            let alpha = (160.0 * flare_intensity) as u8;
                            let fade = alpha.saturating_sub((dx * (150 / h_len)) as u8);
                            if fade > 15 {
                                let blend_color = |cell: &TerminalCell| -> (u8, u8, u8) {
                                    (
                                        cell.fg.0.saturating_add((er as f32 * (fade as f32 / 255.0)) as u8),
                                        cell.fg.1.saturating_add((eg as f32 * (fade as f32 / 255.0)) as u8),
                                        cell.fg.2.saturating_add((eb as f32 * (fade as f32 / 255.0) + 40.0 * (fade as f32 / 255.0)) as u8),
                                    )
                                };
                                if sx + dx < cols {
                                    let cell = &mut grid[sy * cols + (sx + dx)];
                                    cell.fg = blend_color(cell);
                                    if cell.ch == ' ' {
                                        cell.ch = '─';
                                    }
                                }
                                if sx >= dx {
                                    let cell = &mut grid[sy * cols + (sx - dx)];
                                    cell.fg = blend_color(cell);
                                    if cell.ch == ' ' {
                                        cell.ch = '─';
                                    }
                                }
                            }
                        }

                        // Draw vertical streak
                        let v_len = 6;
                        for dy in 1..v_len {
                            let alpha = (110.0 * flare_intensity) as u8;
                            let fade = alpha.saturating_sub((dy * (100 / v_len)) as u8);
                            if fade > 15 {
                                let blend_color = |cell: &TerminalCell| -> (u8, u8, u8) {
                                    (
                                        cell.fg.0.saturating_add((er as f32 * (fade as f32 / 255.0)) as u8),
                                        cell.fg.1.saturating_add((eg as f32 * (fade as f32 / 255.0)) as u8),
                                        cell.fg.2.saturating_add((eb as f32 * (fade as f32 / 255.0) + 20.0 * (fade as f32 / 255.0)) as u8),
                                    )
                                };
                                if sy + dy < rows {
                                    let cell = &mut grid[(sy + dy) * cols + sx];
                                    cell.fg = blend_color(cell);
                                    if cell.ch == ' ' {
                                        cell.ch = '│';
                                    }
                                }
                                if sy >= dy {
                                    let cell = &mut grid[(sy - dy) * cols + sx];
                                    cell.fg = blend_color(cell);
                                    if cell.ch == ' ' {
                                        cell.ch = '│';
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
