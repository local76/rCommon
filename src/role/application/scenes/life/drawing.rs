use crate::core::TerminalCell;
use crate::role::application::palette::query_current_palette;
use super::effect::LifeEffect;
use super::types::{UniverseState, to_screen};
use super::draw_particles::draw_particles_and_trails;

pub fn draw_life(effect: &LifeEffect, grid: &mut [TerminalCell], cols: usize, rows: usize) {
    if cols == 0 || rows == 0 {
        return;
    }

    // library 4.0: pull the canonical ScreenPalette. We use `palette.accent`
    // for the per-particle color math (the `Particle.color` field uses the
    // accent-derived ramp that `update_*` populates).
    let palette = query_current_palette();
    let accent = palette.accent;

    // 1. Clear grid
    for cell in grid.iter_mut() {
        *cell = TerminalCell {
            ch: ' ',
            fg: (0, 0, 0),
            bg: (0, 0, 0),
            bold: false,
        };
    }

    match effect.state {
        UniverseState::Darkness => {
            // Draw remaining crunch ashes
            draw_particles_and_trails(effect, grid, cols, rows, 1.0);
        }
        UniverseState::BigBang => {
            // Draw radiating expansion particles
            draw_particles_and_trails(effect, grid, cols, rows, 1.0);

            // Draw active logo pixels blasting out
            for lp in &effect.logo_pixels {
                if !lp.active { continue; }
                let lx = lp.x.round() as i32;
                let ly = lp.y.round() as i32;
                if lx >= 0 && lx < cols as i32 && ly >= 0 && ly < rows as i32 {
                    let idx = ly as usize * cols + lx as usize;
                    let t = lp.exc;
                    let r = ((accent.0 as f32 * 0.28) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let g = ((accent.1 as f32 * 0.28) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let b = ((accent.2 as f32 * 0.28) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let fg = (r, g, b);
                    let bold = t > 0.35;
                    grid[idx] = TerminalCell {
                        ch: lp.ch,
                        fg,
                        bg: (0, 0, 0),
                        bold,
                    };
                }
            }

            // Draw central explosion shockwave flash
            if effect.state_timer < 1.6 {
                let max_t = 1.6;
                let progress = effect.state_timer / max_t;
                let r_universe = effect.state_timer * 26.0;
                let r_scaled = (r_universe * effect.zoom) as i32;
                let (ucx_screen, ucy_screen) = to_screen(effect.universe_cx, effect.universe_cy, effect.universe_cx, effect.universe_cy, effect.zoom, cols, rows);
                
                for dy in -r_scaled..=r_scaled {
                    for dx in -r_scaled*2..=r_scaled*2 {
                        let rx_f = (dx as f32 / 2.0) / effect.zoom;
                        let ry_f = (dy as f32) / effect.zoom;
                        let dist = (rx_f * rx_f + ry_f * ry_f).sqrt();
                        
                        let shell_thickness = 4.0;
                        if dist < r_universe && dist > (r_universe - shell_thickness).max(0.0) {
                            let gx = ucx_screen + dx;
                            let gy = ucy_screen + dy;
                            if gx >= 0 && gx < cols as i32 && gy >= 0 && gy < rows as i32 {
                                let idx = gy as usize * cols + gx as usize;
                                let shell_rel = (dist - (r_universe - shell_thickness)) / shell_thickness;
                                
                                let color = if progress < 0.25 {
                                    (255, 255, 255)
                                } else if shell_rel > 0.8 {
                                    (100, 240, 255)
                                } else if shell_rel > 0.4 {
                                    (255, 140, 50)
                                } else {
                                    (180, 50, 220)
                                };

                                let ch = if progress < 0.3 {
                                    if shell_rel > 0.5 { '█' } else { '▓' }
                                } else if progress < 0.6 {
                                    if shell_rel > 0.5 { '▓' } else { '░' }
                                } else if progress < 0.9 {
                                    if shell_rel > 0.6 { '░' } else { '·' }
                                } else {
                                    if shell_rel > 0.8 { '·' } else { ' ' }
                                };

                                if ch != ' ' {
                                    grid[idx] = TerminalCell {
                                        ch,
                                        fg: color,
                                        bg: (0, 0, 0),
                                        bold: progress < 0.7,
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
        UniverseState::Expansion | UniverseState::Accretion => {
            // Draw stars and trails
            draw_particles_and_trails(effect, grid, cols, rows, 1.0);

            // Draw gravity seeds
            for seed in &effect.seeds {
                if !seed.active { continue; }
                let (sx, sy) = to_screen(seed.x, seed.y, effect.universe_cx, effect.universe_cy, effect.zoom, cols, rows);
                if seed.is_black_hole {
                    // Draw black hole accretion disk
                    let r_universe = 5.0f32;
                    let r_scaled = (r_universe * effect.zoom) as i32;
                    let fade_in = (seed.birth_timer / 2.0).min(1.0);
                    for dy in -r_scaled..=r_scaled {
                        for dx in -r_scaled*2..=r_scaled*2 {
                            let rx_f = (dx as f32 / 2.0) / effect.zoom;
                            let ry_f = (dy as f32) / effect.zoom;
                            let dist = (rx_f * rx_f + ry_f * ry_f).sqrt();
                            let gx = sx + dx;
                            let gy = sy + dy;
                            if gx >= 0 && gx < cols as i32 && gy >= 0 && gy < rows as i32 {
                                let idx = gy as usize * cols + gx as usize;
                                if dist < 1.3 * fade_in {
                                    grid[idx] = TerminalCell {
                                        ch: ' ',
                                        fg: (0, 0, 0),
                                        bg: (0, 0, 0),
                                        bold: false,
                                    };
                                } else if dist < 4.8 * fade_in {
                                    let angle = ry_f.atan2(rx_f);
                                    let dir = if effect.spin_clockwise { 1.0 } else { -1.0 };
                                    let wave = (angle - effect.time_elapsed * 8.0 * dir + dist * 1.2).sin();
                                    if wave > -0.3 {
                                        let (fg, ch) = if dist < 2.2 * fade_in {
                                            (((180.0 * fade_in) as u8, (240.0 * fade_in) as u8, (255.0 * fade_in) as u8), '╬')
                                        } else if dist < 3.5 * fade_in {
                                            (((seed.color.0 as f32 * fade_in) as u8, (seed.color.1 as f32 * fade_in) as u8, (seed.color.2 as f32 * fade_in) as u8), if wave > 0.3 { '═' } else { '─' })
                                        } else {
                                            (
                                                (
                                                    ((seed.color.0.saturating_sub(50)) as f32 * fade_in) as u8,
                                                    ((seed.color.1.saturating_sub(20)) as f32 * fade_in) as u8,
                                                    ((seed.color.2.saturating_sub(80)) as f32 * fade_in) as u8
                                                ),
                                                if wave > 0.4 { '~' } else { '·' }
                                            )
                                        };
                                        
                                        grid[idx] = TerminalCell {
                                            ch,
                                            fg,
                                            bg: (0, 0, 0),
                                            bold: dist < 3.2 * fade_in,
                                        };
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Draw normal star
                    let r_universe = 2.0f32;
                    let r_scaled = (r_universe * effect.zoom) as i32;
                    let fade_in = (seed.birth_timer / 1.5).min(1.0);
                    if fade_in > 0.01 {
                        for dy in -r_scaled..=r_scaled {
                            for dx in -r_scaled*2..=r_scaled*2 {
                                let rx_f = (dx as f32 / 2.0) / effect.zoom;
                                let ry_f = (dy as f32) / effect.zoom;
                                let dist = rx_f * rx_f + ry_f * ry_f;
                                let r_univ_sq = r_universe * r_universe;
                                if dist <= r_univ_sq {
                                    let gx = sx + dx;
                                    let gy = sy + dy;
                                    if gx >= 0 && gx < cols as i32 && gy >= 0 && gy < rows as i32 {
                                        let idx = gy as usize * cols + gx as usize;
                                        let intensity = ((1.0 - dist / r_univ_sq) * fade_in).clamp(0.0, 1.0);
                                        let gr = (seed.color.0 as f32 * intensity * 0.45) as u8;
                                        let gg = (seed.color.1 as f32 * intensity * 0.45) as u8;
                                        let gb = (seed.color.2 as f32 * intensity * 0.45) as u8;

                                        if grid[idx].ch == ' ' || grid[idx].ch == '·' || grid[idx].ch == '.' {
                                            grid[idx] = TerminalCell {
                                                ch: if dist < 1.0 * fade_in { '❂' } else { '·' },
                                                fg: (gr, gg, gb),
                                                bg: (0, 0, 0),
                                                bold: dist < 1.0 * fade_in,
                                            };
                                        }
                                    }
                                }
                            }
                        }
                        if sx >= 0 && sx < cols as i32 && sy >= 0 && sy < rows as i32 {
                            grid[sy as usize * cols + sx as usize] = TerminalCell {
                                ch: if fade_in > 0.8 { '☼' } else { '·' },
                                fg: if fade_in > 0.8 { (255, 255, 255) } else {
                                    ((seed.color.0 as f32 * fade_in) as u8,
                                     (seed.color.1 as f32 * fade_in) as u8,
                                     (seed.color.2 as f32 * fade_in) as u8)
                                },
                                bg: (0, 0, 0),
                                bold: fade_in > 0.8,
                            };
                        }
                    }
                }
            }

            // Draw logo pixels
            for lp in &effect.logo_pixels {
                if !lp.active { continue; }
                let lx = lp.x.round() as i32;
                let ly = lp.y.round() as i32;
                if lx >= 0 && lx < cols as i32 && ly >= 0 && ly < rows as i32 {
                    let idx = ly as usize * cols + lx as usize;
                    let t = lp.exc;
                    let r = ((accent.0 as f32 * 0.28) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let g = ((accent.1 as f32 * 0.28) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let b = ((accent.2 as f32 * 0.28) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let fg = (r, g, b);
                    let bold = t > 0.35;
                    grid[idx] = TerminalCell {
                        ch: lp.ch,
                        fg,
                        bg: (0, 0, 0),
                        bold,
                    };
                }
            }
        }
        UniverseState::Singularity => {
            // Draw particles spiraling in
            draw_particles_and_trails(effect, grid, cols, rows, 1.0);

            // Draw collapsing logo characters
            for lp in &effect.logo_pixels {
                if !lp.active { continue; }
                let lx = lp.x.round() as i32;
                let ly = lp.y.round() as i32;
                if lx >= 0 && lx < cols as i32 && ly >= 0 && ly < rows as i32 {
                    let idx = ly as usize * cols + lx as usize;
                    let t = lp.exc;
                    let r = ((accent.0 as f32 * 0.20) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let g = ((accent.1 as f32 * 0.20) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let b = ((accent.2 as f32 * 0.20) * (1.0 - t) + 255.0 * t).clamp(0.0, 255.0) as u8;
                    let fg = (r, g, b);
                    let bold = t > 0.35;
                    grid[idx] = TerminalCell {
                        ch: lp.ch,
                        fg,
                        bg: (0, 0, 0),
                        bold,
                    };
                }
            }

            // Draw central black hole singularity and accretion disk
            let bh_x_univ = if !effect.seeds.is_empty() { effect.seeds[0].x } else { effect.universe_cx };
            let bh_y_univ = if !effect.seeds.is_empty() { effect.seeds[0].y } else { effect.universe_cy };
            let (bh_x, bh_y) = to_screen(bh_x_univ, bh_y_univ, effect.universe_cx, effect.universe_cy, effect.zoom, cols, rows);
            let r_universe = 5.0f32;
            let r_scaled = (r_universe * effect.zoom) as i32;
            let fade_in = if !effect.seeds.is_empty() {
                (effect.seeds[0].birth_timer / 1.5).min(1.0)
            } else {
                1.0
            };
            for dy in -r_scaled..=r_scaled {
                for dx in -r_scaled*2..=r_scaled*2 {
                    let rx_f = (dx as f32 / 2.0) / effect.zoom;
                    let ry_f = (dy as f32) / effect.zoom;
                    let dist = (rx_f * rx_f + ry_f * ry_f).sqrt();
                    let gx = bh_x + dx;
                    let gy = bh_y + dy;
                    if gx >= 0 && gx < cols as i32 && gy >= 0 && gy < rows as i32 {
                        let idx = gy as usize * cols + gx as usize;
                        if dist < 1.3 * fade_in {
                            grid[idx] = TerminalCell {
                                ch: ' ',
                                fg: (0, 0, 0),
                                bg: (0, 0, 0),
                                bold: false,
                            };
                        } else if dist < 4.8 * fade_in {
                            let angle = ry_f.atan2(rx_f);
                            let dir = if effect.spin_clockwise { 1.0 } else { -1.0 };
                            let wave = (angle - effect.time_elapsed * 10.0 * dir + dist * 1.2).sin();
                            if wave > -0.3 {
                                let (fg, ch) = if dist < 2.2 * fade_in {
                                    (((180.0 * fade_in) as u8, (240.0 * fade_in) as u8, (255.0 * fade_in) as u8), '╬')
                                } else if dist < 3.5 * fade_in {
                                    (((120.0 * fade_in) as u8, (50.0 * fade_in) as u8, (240.0 * fade_in) as u8), if wave > 0.3 { '═' } else { '─' })
                                } else {
                                    (((70.0 * fade_in) as u8, (20.0 * fade_in) as u8, (140.0 * fade_in) as u8), if wave > 0.4 { '~' } else { '·' })
                                };
                                
                                grid[idx] = TerminalCell {
                                    ch,
                                    fg,
                                    bg: (0, 0, 0),
                                    bold: dist < 3.2 * fade_in,
                                };
                            }
                        }
                    }
                }
            }
        }
        UniverseState::Collapse => {
            let collapse_dim = (1.0 - effect.state_timer / 3.33).max(0.0);
            
            // Draw collapsing particles and trails
            draw_particles_and_trails(effect, grid, cols, rows, collapse_dim);

            // Draw collapsing logo characters
            for lp in &effect.logo_pixels {
                if !lp.active { continue; }
                let lx = lp.x.round() as i32;
                let ly = lp.y.round() as i32;
                if lx >= 0 && lx < cols as i32 && ly >= 0 && ly < rows as i32 {
                    let idx = ly as usize * cols + lx as usize;
                    let t = lp.exc;
                    let r = (((accent.0 as f32 * 0.15) * (1.0 - t) + 255.0 * t) * collapse_dim).clamp(0.0, 255.0) as u8;
                    let g = (((accent.1 as f32 * 0.15) * (1.0 - t) + 255.0 * t) * collapse_dim).clamp(0.0, 255.0) as u8;
                    let b = (((accent.2 as f32 * 0.15) * (1.0 - t) + 255.0 * t) * collapse_dim).clamp(0.0, 255.0) as u8;
                    let fg = (r, g, b);
                    let bold = t > 0.35 && collapse_dim > 0.35;
                    grid[idx] = TerminalCell {
                        ch: lp.ch,
                        fg,
                        bg: (0, 0, 0),
                        bold,
                    };
                }
            }

            // Draw the shrinking/collapsing black hole accretion disk
            let bh_x_univ = if !effect.seeds.is_empty() { effect.seeds[0].x } else { effect.universe_cx };
            let bh_y_univ = if !effect.seeds.is_empty() { effect.seeds[0].y } else { effect.universe_cy };
            let (bh_x, bh_y) = to_screen(bh_x_univ, bh_y_univ, effect.universe_cx, effect.universe_cy, effect.zoom, cols, rows);
            
            let r_universe = (5.0 - effect.state_timer * 1.5).max(0.0);
            if r_universe > 0.1 {
                let r_scaled = (r_universe * effect.zoom) as i32;
                for dy in -r_scaled..=r_scaled {
                    for dx in -r_scaled*2..=r_scaled*2 {
                        let rx_f = (dx as f32 / 2.0) / effect.zoom;
                        let ry_f = (dy as f32) / effect.zoom;
                        let dist = (rx_f * rx_f + ry_f * ry_f).sqrt();
                        let gx = bh_x + dx;
                        let gy = bh_y + dy;
                        if gx >= 0 && gx < cols as i32 && gy >= 0 && gy < rows as i32 {
                            let idx = gy as usize * cols + gx as usize;
                            if dist < 1.3 {
                                grid[idx] = TerminalCell {
                                    ch: ' ',
                                    fg: (0, 0, 0),
                                    bg: (0, 0, 0),
                                    bold: false,
                                };
                            } else if dist < r_universe {
                                let angle = ry_f.atan2(rx_f);
                                let dir = if effect.spin_clockwise { 1.0 } else { -1.0 };
                                let wave = (angle - effect.time_elapsed * 10.0 * dir + dist * 1.2).sin();
                                if wave > -0.3 {
                                    let (fg, ch) = if dist < 2.2 {
                                        ((180, 240, 255), '╬')
                                    } else if dist < 3.5 {
                                        ((120, 50, 240), if wave > 0.3 { '═' } else { '─' })
                                    } else {
                                        ((70, 20, 140), if wave > 0.4 { '~' } else { '·' })
                                    };
                                    
                                    grid[idx] = TerminalCell {
                                        ch,
                                        fg,
                                        bg: (0, 0, 0),
                                        bold: dist < 3.2,
                                    };
                                }
                            }
                        }
                    }
                }
            }

            // Singularity pulsing dot at the collapse center
            let (ucx_i, ucy_i) = to_screen(bh_x_univ, bh_y_univ, effect.universe_cx, effect.universe_cy, effect.zoom, cols, rows);
            if ucx_i >= 0 && ucx_i < cols as i32 && ucy_i >= 0 && ucy_i < rows as i32 {
                let idx = ucy_i as usize * cols + ucx_i as usize;
                let pulse = (effect.time_elapsed * 18.0).sin();
                let ch = if pulse > 0.0 { '█' } else { '☼' };
                grid[idx] = TerminalCell {
                    ch,
                    fg: (255, 255, 255),
                    bg: (0, 0, 0),
                    bold: true,
                };
            }
        }
    }

    // Draw gravitational wave spacetime ripples on top of the grid
    if effect.grav_wave_timer > 0.0 {
        let age = 1.2 - effect.grav_wave_timer;
        let r_universe = age * 50.0;
        let r_screen = r_universe * effect.zoom;
        let (gw_sx, gw_sy) = to_screen(effect.grav_wave_cx, effect.grav_wave_cy, effect.universe_cx, effect.universe_cy, effect.zoom, cols, rows);
        
        for y in 0..rows {
            for x in 0..cols {
                let dx = x as f32 - gw_sx as f32;
                let dy = (y as f32 - gw_sy as f32) * 2.0; // aspect ratio correction
                let dist = (dx * dx + dy * dy).sqrt();
                
                let thickness = 2.0f32;
                if (dist - r_screen).abs() < thickness {
                    let idx = y * cols + x;
                    let intensity = (1.0 - (dist - r_screen).abs() / thickness).clamp(0.0, 1.0) * (effect.grav_wave_timer / 1.2);
                    
                    let r = (160.0 * intensity) as u8;
                    let g = (80.0 * intensity + 100.0 * (1.0 - intensity)) as u8;
                    let b = 255;
                    
                    if grid[idx].ch == ' ' {
                        if intensity > 0.4 {
                            grid[idx] = TerminalCell {
                                ch: if intensity > 0.8 { '≈' } else if intensity > 0.6 { '~' } else { '·' },
                                fg: (r, g, b),
                                bg: (0, 0, 0),
                                bold: intensity > 0.7,
                            };
                        }
                    } else {
                        grid[idx].fg = (
                            (grid[idx].fg.0 as f32 * (1.0 - intensity) + r as f32 * intensity).clamp(0.0, 255.0) as u8,
                            (grid[idx].fg.1 as f32 * (1.0 - intensity) + g as f32 * intensity).clamp(0.0, 255.0) as u8,
                            (grid[idx].fg.2 as f32 * (1.0 - intensity) + b as f32 * intensity).clamp(0.0, 255.0) as u8,
                        );
                        if intensity > 0.5 {
                            grid[idx].bold = true;
                            if grid[idx].ch == '═' || grid[idx].ch == '─' {
                                grid[idx].ch = '≈';
                            } else if grid[idx].ch == '☼' || grid[idx].ch == '❂' {
                                grid[idx].ch = '╬';
                            }
                        }
                    }
                }
            }
        }
    }
}
