use crate::core::TerminalCell;
use crate::role::application::palette::query_current_palette;
use super::effect::Unstable;
use super::types::{Phase, ExplosionType};

impl Unstable {
    pub fn draw_impl(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        // library 4.0: pull the accent per-frame from the canonical
        // ScreenPalette.
        let accent = query_current_palette().accent;

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

        // 2. Draw background stars and their cinematic lens flares/starbursts
        for (i, star) in self.stars.iter().enumerate() {
            let sx = (star.x * cols as f32) as usize;
            let sy = (star.y * rows as f32) as usize;

            if sx < cols && sy < rows {
                // Sparkle value is augmented by excitation!
                let sparkle_base = ((self.phase_timer * 2.0 + star.phase).sin() + 1.0) * 0.5;
                let sparkle = (sparkle_base + star.excitation).min(2.0);
                
                let mut r = (50.0 + sparkle * 80.0) as u8;
                let mut g = (50.0 + sparkle * 80.0) as u8;
                let mut b = (65.0 + sparkle * 75.0) as u8;

                // Blend with accent color when excited
                if star.excitation > 0.1 {
                    let blend = (star.excitation * 0.5).min(1.0);
                    r = (r as f32 * (1.0 - blend) + accent.0 as f32 * blend).min(255.0) as u8;
                    g = (g as f32 * (1.0 - blend) + accent.1 as f32 * blend).min(255.0) as u8;
                    b = (b as f32 * (1.0 - blend) + accent.2 as f32 * blend).min(255.0) as u8;
                }

                let ch = if sparkle > 1.2 {
                    '✹'
                } else if sparkle > 0.6 {
                    '✦'
                } else {
                    star.ch
                };

                grid[sy * cols + sx] = TerminalCell {
                    ch,
                    fg: (r, g, b),
                    bg: grid[sy * cols + sx].bg,
                    bold: sparkle > 0.6 || star.excitation > 0.3,
                };

                // 2b. Draw lens flares and starbursts on highly excited stars
                let is_excited = allowed_flares.contains(&i);
                if is_excited {
                    let flare_intensity = ((star.excitation - 0.8) / 0.7 + 0.5).min(1.5);
                    
                    // Draw horizontal flare (cinematic anamorphic streak, longer if excited)
                    let h_len = 12;
                    for dx in 1..h_len {
                        let alpha = (120.0 * flare_intensity).max(30.0) as u8;
                        let fade = alpha.saturating_sub((dx * (110 / h_len)) as u8);

                        if fade > 10 {
                            if sx + dx < cols {
                                let cell = &mut grid[sy * cols + (sx + dx)];
                                if cell.ch == ' ' || cell.ch == '─' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.75) as u8;
                                    let mut fg_b = fade.saturating_add(45);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '─';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx >= dx {
                                let cell = &mut grid[sy * cols + (sx - dx)];
                                if cell.ch == ' ' || cell.ch == '─' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.75) as u8;
                                    let mut fg_b = fade.saturating_add(45);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '─';
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
                                if cell.ch == ' ' || cell.ch == '│' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.75) as u8;
                                    let mut fg_b = fade.saturating_add(30);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '│';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sy >= dy {
                                let cell = &mut grid[(sy - dy) * cols + sx];
                                if cell.ch == ' ' || cell.ch == '│' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.75) as u8;
                                    let mut fg_b = fade.saturating_add(30);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '│';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                        }
                    }

                    // Draw diagonal starburst spikes
                    let d_len = 3;
                    for d in 1..=d_len {
                        let alpha = (70.0 * flare_intensity).max(15.0) as u8;
                        let fade = alpha.saturating_sub((d * (60 / d_len)) as u8);
                        if fade > 10 {
                            if sx + d < cols && sy >= d {
                                let cell = &mut grid[(sy - d) * cols + (sx + d)];
                                if cell.ch == ' ' || cell.ch == '/' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.65) as u8;
                                    let mut fg_b = fade.saturating_add(20);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '/';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx >= d && sy + d < rows {
                                let cell = &mut grid[(sy + d) * cols + (sx - d)];
                                if cell.ch == ' ' || cell.ch == '/' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.65) as u8;
                                    let mut fg_b = fade.saturating_add(20);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '/';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx >= d && sy >= d {
                                let cell = &mut grid[(sy - d) * cols + (sx - d)];
                                if cell.ch == ' ' || cell.ch == '\\' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.65) as u8;
                                    let mut fg_b = fade.saturating_add(20);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '\\';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                            if sx + d < cols && sy + d < rows {
                                let cell = &mut grid[(sy + d) * cols + (sx + d)];
                                if cell.ch == ' ' || cell.ch == '\\' {
                                    let mut fg_r = fade;
                                    let mut fg_g = (fade as f32 * 0.65) as u8;
                                    let mut fg_b = fade.saturating_add(20);
                                    
                                    fg_r = (fg_r as f32 * 0.5 + accent.0 as f32 * 0.5).min(255.0) as u8;
                                    fg_g = (fg_g as f32 * 0.5 + accent.1 as f32 * 0.5).min(255.0) as u8;
                                    fg_b = (fg_b as f32 * 0.5 + accent.2 as f32 * 0.5).min(255.0) as u8;
                                    
                                    cell.ch = '\\';
                                    cell.fg = (fg_r, fg_g, fg_b);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Draw particles + special side effects per explosion type
        let center_x = cols as f32 / 2.0;
        let center_y = rows as f32 / 2.0;
        let max_possible_dist = (center_x*center_x + center_y*center_y).sqrt().max(1.0);

        // Special side effects (visual flair only in Chaos phase)
        if self.phase == Phase::Chaos {
            match self.explosion_type {
                ExplosionType::Shockwave => {
                    // Expanding shock ring (deterministic pattern using phase_timer)
                    let ring_radius = ((self.phase_timer * 28.0) % (max_possible_dist * 1.2)) as i32;
                    let ring_thickness = 2;
                    for r in (ring_radius - ring_thickness)..=(ring_radius + ring_thickness) {
                        if r < 2 { continue; }
                        for angle_step in 0..36 {
                            let angle = (angle_step as f32) * 10.0 * std::f32::consts::PI / 180.0;
                            let rx = (center_x + r as f32 * angle.cos()).round() as i32;
                            let ry = (center_y + r as f32 * angle.sin() * 0.48).round() as i32; // aspect
                            if rx >= 0 && rx < cols as i32 && ry >= 0 && ry < rows as i32 {
                                let idx = (ry as usize) * cols + (rx as usize);
                                let cell = &mut grid[idx];
                                if cell.ch == ' ' || cell.ch == '.' || cell.ch == '•' {
                                    // Deterministic choice
                                    let use_block = ((r + angle_step) % 3) == 0;
                                    cell.ch = if use_block { '▓' } else { '░' };
                                    let intensity = (180.0 + (r as f32 - ring_radius as f32).abs() * 20.0).min(255.0) as u8;
                                    cell.fg = (intensity, (intensity as f32 * 0.7) as u8, intensity.saturating_sub(30));
                                    cell.bold = true;
                                }
                            }
                        }
                    }
                }
                ExplosionType::Entropy => {
                    // Data rot: deterministically corrupt background cells near unsnapped particles (no &mut rng in &self draw)
                    for p in &self.particles {
                        if !p.snapped {
                            let px = p.x.round() as i32;
                            let py = p.y.round() as i32;
                            // Use phase_timer + position for deterministic "randomness"
                            let seed = ((self.phase_timer * 17.0 + px as f32 * 0.7 + py as f32) as i32) % 17;
                            if seed % 7 < 2 {
                                for d in 0..3 {
                                    let ox = (seed + d) % 7 - 3;
                                    let oy = (seed * 3 + d) % 5 - 2;
                                    let rx = px + ox;
                                    let ry = py + oy;
                                    if rx >= 0 && rx < cols as i32 && ry >= 0 && ry < rows as i32 {
                                        let idx = (ry as usize) * cols + (rx as usize);
                                        let cell = &mut grid[idx];
                                        if cell.ch == ' ' || cell.ch == '.' {
                                            cell.ch = ['░', '▒', '▓', '?', '#'][((seed + d) as usize) % 5];
                                            cell.fg = (80, 60, 40);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ExplosionType::Resonance => {
                    // Resonance hum: modulate brightness of nearby static grid cells
                    let hum = ((self.phase_timer * 25.0).sin() * 0.5 + 0.5).min(1.0);
                    for p in &self.particles {
                        if p.snapped {
                            let px = p.x.round() as i32;
                            let py = p.y.round() as i32;
                            if px >= 0 && px < cols as i32 && py >= 0 && py < rows as i32 {
                                let idx = (py as usize) * cols + (px as usize);
                                let cell = &mut grid[idx];
                                if cell.ch != ' ' {
                                    let boost = (hum * 40.0) as u8;
                                    cell.fg = (
                                        cell.fg.0.saturating_add(boost),
                                        cell.fg.1.saturating_add(boost / 2),
                                        cell.fg.2.saturating_add(boost / 3),
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let is_aberration = match self.phase {
            Phase::Exploding => self.phase_timer < 0.6,
            Phase::SnapBack => true,
            Phase::Assembled => (self.phase_timer % 4.0) < 0.18,
            _ => false,
        };

        let shift = if is_aberration {
            match self.phase {
                Phase::Exploding => (((0.6 - self.phase_timer) * 5.0) as i32).max(1),
                Phase::Assembled if (self.phase_timer * 20.0) as i32 % 2 == 0 => 2,
                _ => 1,
            }
        } else {
            0
        };

        for p in &self.particles {
            let px = p.x.round() as i32;
            let py = p.y.round() as i32;

            if px >= 0 && px < cols as i32 && py >= 0 && py < rows as i32 {
                let dx = p.x - center_x;
                let dy = p.y - center_y;
                let dist = (dx*dx + dy*dy).sqrt();

                let color = if self.phase == Phase::Assembled {
                    let glow_factor = p.glow.min(1.5);
                    if glow_factor > 1.0 {
                        let extra = ((glow_factor - 1.0) * 2.0 * 255.0).min(255.0) as u8;
                        let r = (extra.max(accent.0)).max(160);
                        let g = (extra.max(accent.1)).max(160);
                        let b = (extra.max(accent.2)).max(160);
                        (r, g, b)
                    } else {
                        let r = (accent.0 as f32 * (0.6 + 0.4 * glow_factor)).min(255.0) as u8;
                        let g = (accent.1 as f32 * (0.6 + 0.4 * glow_factor)).min(255.0) as u8;
                        let b = (accent.2 as f32 * (0.6 + 0.4 * glow_factor)).min(255.0) as u8;
                        (r, g, b)
                    }
                } else {
                    let ratio = (dist / max_possible_dist).min(1.0);
                    let r = (255.0 * ratio + (accent.0 as f32) * (1.0 - ratio)) as u8;
                    let g = (110.0 * ratio + (accent.1 as f32) * (1.0 - ratio)) as u8;
                    let b = ((accent.2 as f32) * (1.0 - ratio)) as u8;
                    (r, g, b)
                };

                let idx = py as usize * cols + px as usize;

                // Draw chromatic splits (red left, blue right)
                if shift > 0 {
                    let rx = px - shift;
                    if rx >= 0 && rx < cols as i32 {
                        let r_idx = py as usize * cols + rx as usize;
                        grid[r_idx] = TerminalCell {
                            ch: p.ch,
                            fg: (230, 10, 50),
                            bg: grid[r_idx].bg,
                            bold: false,
                        };
                    }
                    let bx = px + shift;
                    if bx >= 0 && bx < cols as i32 {
                        let b_idx = py as usize * cols + bx as usize;
                        grid[b_idx] = TerminalCell {
                            ch: p.ch,
                            fg: (0, 120, 255),
                            bg: grid[b_idx].bg,
                            bold: false,
                        };
                    }
                }

                // Main particle (shifts to neon green during RGB glitch splits)
                grid[idx] = TerminalCell {
                    ch: p.ch,
                    fg: if shift > 0 { (10, 230, 80) } else { color },
                    bg: grid[idx].bg,
                    bold: self.phase == Phase::Assembled || p.glow > 0.8,
                };
            }
        }
    }
}
