use std::time::Duration;
use crate::core::{hsl_to_rgb, rgb_to_hsl};
use crate::core::logo_block::render_logo_block;
use crate::platform::native::sys_info::get_system_info;
use crate::role::application::palette::query_current_palette;
use crate::role::application::rgb::protocol::RgbColor;
use super::effect::LifeEffect;
use super::types::{UniverseState, Particle, GravityCenter, LogoPixel};
use super::update_expansion;
use super::update_collapse;

pub fn enter_state(eff: &mut LifeEffect, cols: usize, rows: usize) {
    let cols_f = cols as f32;
    let rows_f = rows as f32;
    let cx = cols_f / 2.0;
    let cy = rows_f / 2.0;

    // library 4.0: pull the accent hue from the canonical ScreenPalette.
    // The pre-4.0 code called `get_theme_accent()` + `rgb_to_hsl` here; now
    // we read `palette.accent` from the library-routed palette.
    let palette = query_current_palette();
    let (acc_h, _, _) = rgb_to_hsl(palette.accent.0, palette.accent.1, palette.accent.2);

    match eff.state {
        UniverseState::Darkness => {
            eff.particles.clear();
            eff.seeds.clear();
            for lp in &mut eff.logo_pixels {
                lp.active = false;
            }
            eff.universe_cx = cx;
            eff.universe_cy = cy;
            eff.zoom = 0.75;

            for _ in 0..20 {
                let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                let speed = eff.rng.next_range(2.0, 8.0);
                eff.particles.push(Particle {
                    x: eff.universe_cx,
                    y: eff.universe_cy,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed * 0.45,
                    mass: 0.3,
                    color: (100, 100, 100),
                    ch: '·',
                    history: Vec::new(),
                });
            }
        }
        UniverseState::BigBang => {
            eff.particles.clear();
            eff.seeds.clear();
            
            eff.spin_clockwise = eff.rng.next_bool(0.5);
            let dir = if eff.spin_clockwise { 1.0f32 } else { -1.0f32 };
            
            for lp in &mut eff.logo_pixels {
                lp.active = true;
                let rx = eff.rng.next_range(-2.5, 2.5);
                let ry = eff.rng.next_range(-1.2, 1.2);
                lp.x = eff.universe_cx + rx;
                lp.y = eff.universe_cy + ry;
                let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                let speed = eff.rng.next_range(20.0, 70.0);
                let swirl_speed = speed * 0.22;
                lp.vx = angle.cos() * speed - angle.sin() * swirl_speed * dir;
                lp.vy = angle.sin() * speed * 0.42 + angle.cos() * swirl_speed * 0.42 * dir;
                lp.exc = 1.0;
            }

            let num_particles = 120 + (eff.seed_density_opt * 45).min(350) as usize;
            
            for _ in 0..(num_particles * 2 / 3) {
                let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                let speed = eff.rng.next_range(15.0, 55.0);
                let swirl_speed = speed * 0.20;
                let vx = angle.cos() * speed - angle.sin() * swirl_speed * dir;
                let vy = angle.sin() * speed * 0.42 + angle.cos() * swirl_speed * 0.42 * dir;
                
                let p_hue = (acc_h + eff.rng.next_range(-40.0, 40.0)).rem_euclid(360.0);
                let color = hsl_to_rgb(p_hue, 0.90, 0.55);

                let rx = eff.rng.next_range(-3.5, 3.5);
                let ry = eff.rng.next_range(-1.6, 1.6);

                eff.particles.push(Particle {
                    x: eff.universe_cx + rx,
                    y: eff.universe_cy + ry,
                    vx,
                    vy,
                    mass: eff.rng.next_range(0.7, 1.3),
                    color,
                    ch: if eff.rng.next_bool(0.4) { '+' } else if eff.rng.next_bool(0.5) { '·' } else { '.' },
                    history: Vec::new(),
                });
            }

            for _ in 0..(num_particles / 3) {
                let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                let speed = eff.rng.next_range(65.0, 95.0);
                let swirl_speed = speed * 0.15;
                let vx = angle.cos() * speed - angle.sin() * swirl_speed * dir;
                let vy = angle.sin() * speed * 0.42 + angle.cos() * swirl_speed * 0.42 * dir;

                let color = if eff.rng.next_bool(0.5) {
                    (255, 220, 100)
                } else {
                    (100, 240, 255)
                };

                let rx = eff.rng.next_range(-2.0, 2.0);
                let ry = eff.rng.next_range(-0.9, 0.9);

                eff.particles.push(Particle {
                    x: eff.universe_cx + rx,
                    y: eff.universe_cy + ry,
                    vx,
                    vy,
                    mass: eff.rng.next_range(0.5, 0.8),
                    color,
                    ch: '*',
                    history: Vec::new(),
                });
            }
        }
        UniverseState::Expansion => {
            for lp in &mut eff.logo_pixels {
                lp.active = true;
            }
        }
        UniverseState::Accretion => {
            eff.seeds.clear();
            let num_seeds = eff.rng.next_range(4.0, 10.0) as usize;
            let total_mass = 30.0;
            let mass_per_seed = total_mass / num_seeds as f32;
            let dir = if eff.spin_clockwise { 1.0f32 } else { -1.0f32 };
            
            for i in 0..num_seeds {
                let angle = (i as f32 / num_seeds as f32) * std::f32::consts::TAU + eff.host_bias;
                let dist_r = eff.rng.next_range(cols_f * 0.12, cols_f * 0.38);
                let sx = eff.universe_cx + angle.cos() * dist_r;
                let sy = eff.universe_cy + angle.sin() * dist_r * 0.45;
                
                let tx = -angle.sin();
                let ty = angle.cos();
                let orbit_speed = (180.0 / dist_r).sqrt().clamp(3.0, 12.0);
                let vx = tx * orbit_speed * dir + eff.rng.next_range(-0.5, 0.5);
                let vy = ty * orbit_speed * 0.45 * dir + eff.rng.next_range(-0.25, 0.25);
                
                let seed_h = (acc_h + (i as f32 * (360.0 / num_seeds as f32))).rem_euclid(360.0);
                let seed_color = hsl_to_rgb(seed_h, 0.95, 0.55);
                eff.seeds.push(GravityCenter {
                    x: sx,
                    y: sy,
                    vx,
                    vy,
                    mass: mass_per_seed,
                    color: seed_color,
                    active: true,
                    is_black_hole: false,
                    birth_timer: 0.0,
                });

                for _ in 0..15 {
                    let spark_angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                    let spark_speed = eff.rng.next_range(8.0, 18.0);
                    eff.particles.push(Particle {
                        x: sx,
                        y: sy,
                        vx: vx + spark_angle.cos() * spark_speed,
                        vy: vy + spark_angle.sin() * spark_speed * 0.45,
                        mass: 0.4,
                        color: (255, 220, 130),
                        ch: '+',
                        history: Vec::new(),
                    });
                }
            }

            for lp in &mut eff.logo_pixels {
                lp.active = true;
            }
        }
        UniverseState::Singularity => {
            let mut last_seed = None;
            for seed in &eff.seeds {
                if seed.active {
                    last_seed = Some(seed);
                    break;
                }
            }
            
            let (sx, sy, smass, scolor, svx, svy, sbirth) = if let Some(s) = last_seed {
                (s.x, s.y, s.mass, s.color, s.vx, s.vy, if s.is_black_hole { 10.0 } else { 0.0 })
            } else {
                (cx, cy, 30.0, (130, 50, 240), 0.0, 0.0, 0.0)
            };
            
            eff.universe_cx = cx;
            eff.universe_cy = cy;
            
            eff.seeds.clear();
            eff.seeds.push(GravityCenter {
                x: sx,
                y: sy,
                vx: svx,
                vy: svy,
                mass: smass.max(30.0),
                color: scolor,
                active: true,
                is_black_hole: true,
                birth_timer: sbirth,
            });

            let flash_color = (130, 50, 240);
            for _ in 0..50 {
                let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                let speed = eff.rng.next_range(25.0, 50.0);
                eff.particles.push(Particle {
                    x: sx,
                    y: sy,
                    vx: svx + angle.cos() * speed,
                    vy: svy + angle.sin() * speed * 0.45,
                    mass: 0.5,
                    color: flash_color,
                    ch: '╬',
                    history: Vec::new(),
                });
            }
        }
        UniverseState::Collapse => {
            let bh_x = eff.universe_cx;
            let bh_y = eff.universe_cy;
            for i in 0..50 {
                let angle = (i as f32 / 50.0) * std::f32::consts::TAU;
                let dist = 35.0f32;
                let px = bh_x + angle.cos() * dist;
                let py = bh_y + angle.sin() * dist * 0.45;
                let speed = -45.0f32;
                eff.particles.push(Particle {
                    x: px,
                    y: py,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed * 0.45,
                    mass: 0.6,
                    color: (255, 255, 255),
                    ch: '░',
                    history: Vec::new(),
                });
            }
        }
    }
}

pub fn update_life(eff: &mut LifeEffect, dt: Duration, cols: usize, rows: usize) {
    let delta = dt.as_secs_f32().min(0.1) * (eff.sim_speed_opt.max(1) as f32);
    eff.time_elapsed += delta;
    eff.state_timer += delta;

    eff.rgb_timer += delta;
    if eff.rgb_timer >= 0.06 {
        eff.rgb_timer = 0.0;
        if let Some(ref r) = eff.rgb {
            if Some(eff.state) != eff.last_state {
                eff.last_state = Some(eff.state);
                match eff.state {
                    UniverseState::BigBang => {
                        r.flash(RgbColor::WHITE, Duration::from_millis(300));
                    }
                    UniverseState::Collapse => {
                        r.flash(RgbColor::new(140, 0, 255), Duration::from_millis(400));
                    }
                    _ => {}
                }
            }
            
            match eff.state {
                UniverseState::Darkness => {
                    r.set_color(RgbColor::new(0, 0, 15));
                }
                UniverseState::BigBang => {
                    let intensity = (1.0 - eff.state_timer / 1.6).clamp(0.0, 1.0);
                    let w = (intensity * 255.0) as u8;
                    r.set_color(RgbColor::new(w, w, w));
                }
                UniverseState::Expansion => {
                    let t = eff.state_timer * 1.5;
                    let get_expansion_color = |x: f32| -> RgbColor {
                        let dist = (x - t).abs();
                        let brightness = (-dist * dist * 3.0).exp().clamp(0.0, 1.0);
                        let c = (brightness * 200.0) as u8;
                        let b = (20.0 + brightness * 235.0).min(255.0) as u8;
                        RgbColor::new(0, c, b)
                    };
                    
                    r.set_device_color(5, get_expansion_color(0.5));
                    r.set_device_color(6, get_expansion_color(0.8));
                    r.set_device_color(12, get_expansion_color(0.1));
                    let c_internal = get_expansion_color(0.6);
                    r.set_device_color(0, c_internal);
                    r.set_device_color(1, c_internal);
                    r.set_device_color(2, c_internal);
                }
                UniverseState::Accretion => {
                    let t = eff.time_elapsed * 2.0;
                    let get_accretion_color = |phase_offset: f32| -> RgbColor {
                        let angle = t + phase_offset;
                        let red = (180.0 + angle.cos() * 75.0).clamp(0.0, 255.0) as u8;
                        let green = (60.0 + angle.sin() * 50.0).clamp(0.0, 255.0) as u8;
                        RgbColor::new(red, green, 0)
                    };
                    
                    r.set_device_color(5, get_accretion_color(0.0));
                    r.set_device_color(6, get_accretion_color(2.0));
                    r.set_device_color(12, get_accretion_color(4.0));
                    let c_internal = get_accretion_color(1.0);
                    r.set_device_color(0, c_internal);
                    r.set_device_color(1, c_internal);
                    r.set_device_color(2, c_internal);
                }
                UniverseState::Singularity => {
                    let pulse = ((eff.state_timer * 4.0).sin() * 0.3 + 0.7).clamp(0.0, 1.0);
                    let red = (pulse * 130.0) as u8;
                    let blue = (pulse * 255.0) as u8;
                    r.set_color(RgbColor::new(red, 0, blue));
                }
                UniverseState::Collapse => {
                    let progress = (eff.state_timer / 5.0).clamp(0.0, 1.0);
                    let scale = (1.0 - progress) * (1.0 - progress);
                    let red = (scale * 100.0) as u8;
                    let blue = (scale * 200.0) as u8;
                    r.set_color(RgbColor::new(red, 0, blue));
                }
            }
        }
    }

    for seed in &mut eff.seeds {
        if seed.active {
            seed.birth_timer += delta;
        }
    }

    if eff.grav_wave_timer > 0.0 {
        eff.grav_wave_timer = (eff.grav_wave_timer - delta).max(0.0);
    }

    eff.sys_refresh_timer += delta;
    if eff.sys_refresh_timer >= 1.0 {
        let sys = get_system_info();
        eff.mem_pressure = sys.mem_used_pct / 100.0;
        eff.cpu_load = (eff.mem_pressure * 0.6 + 0.3).min(0.9);
        eff.sys_refresh_timer = 0.0;
    }

    if cols != eff.last_cols || rows != eff.last_rows {
        eff.last_cols = cols;
        eff.last_rows = rows;
        eff.universe_cx = cols as f32 / 2.0;
        eff.universe_cy = rows as f32 / 2.0;
        eff.zoom = 0.75;

        // library 4.1: render the centered system logo from the live OS info
        // (replaces pre-4.1 `trance_core::logo_lines()` + `logo_dimensions()`).
        let logo_text = get_system_info().logo_text;
        let lines = render_logo_block(&logo_text, None);
        let logo_h = lines.len();
        let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let logo_x = cols.saturating_sub(logo_w) / 2;
        let logo_y = rows.saturating_sub(logo_h) / 2;

        eff.logo_pixels.clear();
        for (r_offset, line) in lines.iter().enumerate().take(logo_h) {
            let gy = logo_y + r_offset;
            for (c_offset, ch) in line.chars().enumerate() {
                let gx = logo_x + c_offset;
                if ch != ' ' {
                    eff.logo_pixels.push(LogoPixel {
                        x: gx as f32,
                        y: gy as f32,
                        vx: 0.0,
                        vy: 0.0,
                        origin_x: gx as f32,
                        origin_y: gy as f32,
                        ch,
                        exc: 0.0,
                        active: false,
                    });
                }
            }
        }

        eff.state = UniverseState::Darkness;
        eff.state_timer = 0.0;
        enter_state(eff, cols, rows);
    }

    let next_state = match eff.state {
        UniverseState::Darkness => {
            if eff.state_timer >= 2.0 + (eff.host_bias * 1.5) { Some(UniverseState::BigBang) } else { None }
        }
        UniverseState::BigBang => {
            if eff.state_timer >= 1.6 { Some(UniverseState::Expansion) } else { None }
        }
        UniverseState::Expansion => {
            let avg_vel = if eff.particles.is_empty() {
                0.0
            } else {
                eff.particles.iter().map(|p| (p.vx*p.vx + p.vy*p.vy).sqrt()).sum::<f32>() / eff.particles.len() as f32
            };
            let avg_logo_dist = if eff.logo_pixels.is_empty() {
                0.0
            } else {
                eff.logo_pixels.iter().map(|lp| ((lp.x - lp.origin_x)*(lp.x - lp.origin_x) + (lp.y - lp.origin_y)*(lp.y - lp.origin_y)).sqrt()).sum::<f32>() / eff.logo_pixels.len() as f32
            };
            if (avg_vel < 16.0 && avg_logo_dist < 0.65) || eff.state_timer >= 10.0 {
                Some(UniverseState::Accretion)
            } else {
                None
            }
        }
        UniverseState::Accretion => {
            let active_seeds = eff.seeds.iter().filter(|s| s.active).count();
            if active_seeds <= 1 {
                Some(UniverseState::Singularity)
            } else {
                None
            }
        }
        UniverseState::Singularity => {
            if eff.state_timer >= 6.0 {
                Some(UniverseState::Collapse)
            } else {
                None
            }
        }
        UniverseState::Collapse => {
            let active_logo_pixels = eff.logo_pixels.iter().filter(|lp| lp.active).count();
            if (eff.particles.is_empty() && active_logo_pixels == 0) || eff.state_timer >= 5.0 {
                Some(UniverseState::Darkness)
            } else {
                None
            }
        }
    };

    if let Some(ns) = next_state {
        eff.state = ns;
        eff.state_timer = 0.0;
        enter_state(eff, cols, rows);
    }

    match eff.state {
        UniverseState::Darkness => {
            for p in &mut eff.particles {
                p.vx *= 1.0 - (delta * 0.5);
                p.vy *= 1.0 - (delta * 0.5);
                p.x += p.vx * delta;
                p.y += p.vy * delta;
            }
        }
        UniverseState::BigBang => {
            for p in &mut eff.particles {
                p.vx *= 1.0 - (delta * 0.40);
                p.vy *= 1.0 - (delta * 0.40);
                p.x += p.vx * delta;
                p.y += p.vy * delta;

                let cx_i = p.x.round() as i32;
                let cy_i = p.y.round() as i32;
                p.history.push((cx_i, cy_i));
                if p.history.len() > 4 { p.history.remove(0); }
            }

            let cols_f = cols as f32;
            let rows_f = rows as f32;
            for lp in &mut eff.logo_pixels {
                if lp.active {
                    lp.vx *= 1.0 - (delta * 0.40);
                    lp.vy *= 1.0 - (delta * 0.40);
                    lp.x += lp.vx * delta;
                    lp.y += lp.vy * delta;
                    lp.x = lp.x.rem_euclid(cols_f);
                    lp.y = lp.y.rem_euclid(rows_f);
                    
                    lp.exc = (lp.exc - 0.2 * delta).max(0.6);
                }
            }
        }
        UniverseState::Expansion => {
            update_expansion::update_expansion(eff, delta, cols, rows);
        }
        UniverseState::Accretion => {
            update_expansion::update_accretion(eff, delta, cols, rows);
        }
        UniverseState::Singularity => {
            update_collapse::update_singularity(eff, delta, cols, rows);
        }
        UniverseState::Collapse => {
            update_collapse::update_collapse(eff, delta, cols, rows);
        }
    }
}
