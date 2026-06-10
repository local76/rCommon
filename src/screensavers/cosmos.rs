//! Consolidated cosmos screensaver effect module.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).


use crate::core::{LcgRng, TerminalCell, hsl_to_rgb, rgb_to_hsl};
use std::time::Duration;
use crate::core::screensaver::Screensaver;
#[cfg(feature = "sys-info")]
use crate::platform::native::sys_info::get_system_info;
#[cfg(feature = "rgb")]
use crate::toolkit::rgb_controller::{ is_openrgb_enabled, RgbController };
use crate::core::logo_block::render_logo_block;
use crate::core::screen_palette::query_current_palette;
#[cfg(feature = "rgb")]
use crate::toolkit::rgb_protocol::RgbColor;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UniverseState {
    Darkness,
    BigBang,
    Expansion,
    Accretion,
    Singularity,
    Collapse,
}

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub color: (u8, u8, u8),
    pub ch: char,
    pub history: Vec<(i32, i32)>,
}

pub struct GravityCenter {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub mass: f32,
    pub color: (u8, u8, u8),
    pub active: bool,
    pub is_black_hole: bool,
    pub birth_timer: f32,
}

pub struct LogoPixel {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub ch: char,
    pub exc: f32,
    pub active: bool,
}

pub fn to_screen(ux: f32, uy: f32, universe_cx: f32, universe_cy: f32, zoom: f32, cols: usize, rows: usize) -> (i32, i32) {
    let cx = cols as f32 / 2.0;
    let cy = rows as f32 / 2.0;
    let sx = cx + (ux - universe_cx) * zoom;
    let sy = cy + (uy - universe_cy) * zoom;
    (sx.round() as i32, sy.round() as i32)
}

pub fn to_universe(sx: f32, sy: f32, universe_cx: f32, universe_cy: f32, zoom: f32, cols: usize, rows: usize) -> (f32, f32) {
    let cx = cols as f32 / 2.0;
    let cy = rows as f32 / 2.0;
    let ux = universe_cx + (sx - cx) / zoom;
    let uy = universe_cy + (sy - cy) / zoom;
    (ux, uy)
}


pub struct Cosmos {
    pub(crate) rng: LcgRng,
    pub(crate) state: UniverseState,
    pub(crate) state_timer: f32,
    pub(crate) particles: Vec<Particle>,
    pub(crate) seeds: Vec<GravityCenter>,
    pub(crate) logo_pixels: Vec<LogoPixel>,
    pub(crate) time_elapsed: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,

    // Settings
    pub(crate) seed_density_opt: u32,
    pub(crate) sim_speed_opt: u32,

    // Live system dynamics
    pub(crate) sys_refresh_timer: f32,
    pub(crate) mem_pressure: f32,
    pub(crate) cpu_load: f32,
    pub(crate) host_bias: f32,
    pub(crate) universe_cx: f32,
    pub(crate) universe_cy: f32,
    pub(crate) spin_clockwise: bool,
    pub(crate) zoom: f32,
    pub(crate) grav_wave_timer: f32,
    pub(crate) grav_wave_cx: f32,
    pub(crate) grav_wave_cy: f32,
    pub(crate) rgb: Option<RgbController>,
    pub(crate) rgb_timer: f32,
    pub(crate) last_state: Option<UniverseState>,
}

impl Default for Cosmos {
    fn default() -> Self {
        Self::new()
    }
}

impl Cosmos {
    pub fn new() -> Self {
        // Pre-4.1 HKEY_CURRENT_USER registry reads (SeedDensity, SimSpeed)
        // collapsed to defaults for the inline migration. Re-added in 4.2.
        let seed_density_opt: u32 = 3;
        let sim_speed_opt: u32 = 1;

        let sys = get_system_info();
        let host_bias = sys.hostname.chars().map(|c| c as u32).sum::<u32>() as f32 / 1000.0 % 1.0;

        Self {
            rng: LcgRng::new(7777),
            state: UniverseState::Darkness,
            state_timer: 0.0,
            particles: Vec::new(),
            seeds: Vec::new(),
            logo_pixels: Vec::new(),
            time_elapsed: 0.0,
            last_cols: 0,
            last_rows: 0,
            seed_density_opt,
            sim_speed_opt,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: 0.4,
            host_bias,
            universe_cx: 0.0,
            universe_cy: 0.0,
            spin_clockwise: true,
            zoom: 0.75,
            grav_wave_timer: 0.0,
            grav_wave_cx: 0.0,
            grav_wave_cy: 0.0,
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
            rgb_timer: 0.0,
            last_state: None,
        }
    }
}

impl Screensaver for Cosmos {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        update_life(self, dt, cols, rows);
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        draw_life(self, grid, cols, rows);
    }
}

pub fn enter_state(eff: &mut Cosmos, cols: usize, rows: usize) {
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

pub fn update_life(eff: &mut Cosmos, dt: Duration, cols: usize, rows: usize) {
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
            update_expansion(eff, delta, cols, rows);
        }
        UniverseState::Accretion => {
            update_accretion(eff, delta, cols, rows);
        }
        UniverseState::Singularity => {
            update_singularity(eff, delta, cols, rows);
        }
        UniverseState::Collapse => {
            update_collapse(eff, delta, cols, rows);
        }
    }
}
pub fn update_expansion(eff: &mut Cosmos, delta: f32, _cols: usize, _rows: usize) {
    let dir = if eff.spin_clockwise { 1.0f32 } else { -1.0f32 };
    let progress = (eff.state_timer / 7.0).min(1.0);
    
    for p in &mut eff.particles {
        let dx = eff.universe_cx - p.x;
        let dy = eff.universe_cy - p.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.1);
        
        let pull = 10.0 * progress / (dist + 6.0);
        let tangent = 15.0 * progress / (dist.sqrt() + 2.0);
        
        p.vx += ((dx / dist) * pull + (dy / dist) * tangent * dir) * delta;
        p.vy += ((dy / dist) * pull * 0.45 - (dx / dist) * tangent * 0.45 * dir) * delta;

        p.vx *= 1.0 - (delta * 0.35);
        p.vy *= 1.0 - (delta * 0.35);
        p.x += p.vx * delta;
        p.y += p.vy * delta;

        let cx_i = p.x.round() as i32;
        let cy_i = p.y.round() as i32;
        p.history.push((cx_i, cy_i));
        if p.history.len() > 4 { p.history.remove(0); }
    }

    let progress = (eff.state_timer / 8.0).min(1.0);
    let k = 0.5 + progress * 5.5;
    let drag = 1.0 + progress * 2.0;
    
    for lp in &mut eff.logo_pixels {
        if lp.active {
            let dx = lp.origin_x - lp.x;
            let dy = lp.origin_y - lp.y;
            
            lp.vx += dx * k * delta;
            lp.vy += dy * k * delta;
            
            lp.vx *= 1.0 - (drag * delta);
            lp.vy *= 1.0 - (drag * delta);
            
            lp.x += lp.vx * delta;
            lp.y += lp.vy * delta;

            lp.exc = (lp.exc - 0.4 * delta).max(0.0);
        }
    }
}

pub fn update_accretion(eff: &mut Cosmos, delta: f32, cols: usize, rows: usize) {
    let dir = if eff.spin_clockwise { 1.0f32 } else { -1.0f32 };
    let seeds_len = eff.seeds.len();
    let cx = cols as f32 / 2.0;
    let cy = rows as f32 / 2.0;
    
    // Gravity centers drift
    for i in 0..seeds_len {
        if !eff.seeds[i].active { continue; }
        let mut sfx = 0.0f32;
        let mut sfy = 0.0f32;
        for j in 0..seeds_len {
            if i == j || !eff.seeds[j].active { continue; }
            let dx = eff.seeds[j].x - eff.seeds[i].x;
            let dy = eff.seeds[j].y - eff.seeds[i].y;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt().max(0.1);
            let force = (eff.seeds[j].mass * 12.0) / (dist_sq + 15.0);
            sfx += (dx / dist) * force;
            sfy += (dy / dist) * force;
        }
        let dx_c = cx - eff.seeds[i].x;
        let dy_c = cy - eff.seeds[i].y;
        let dist_c = (dx_c * dx_c + dy_c * dy_c).sqrt().max(0.1);
        sfx += (dx_c / dist_c) * 1.8;
        sfy += (dy_c / dist_c) * 0.9;

        let orbit_force = 3.5f32 * (1.0 - (eff.state_timer / 15.0).min(0.85));
        sfx += (dy_c / dist_c) * orbit_force * dir;
        sfy += (-dx_c / dist_c) * orbit_force * 0.45 * dir;
        for lp in &eff.logo_pixels {
            if !lp.active { continue; }
            let (lp_ux, lp_uy) = to_universe(lp.x, lp.y, eff.universe_cx, eff.universe_cy, eff.zoom, cols, rows);
            let dx = lp_ux - eff.seeds[i].x;
            let dy = lp_uy - eff.seeds[i].y;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt().max(0.1);
            let force = 0.50 / (dist_sq + 8.0);
            sfx += (dx / dist) * force;
            sfy += (dy / dist) * force;
        }

        eff.seeds[i].vx += sfx * delta;
        eff.seeds[i].vy += sfy * delta;
        eff.seeds[i].vx *= 1.0 - (delta * 0.25);
        eff.seeds[i].vy *= 1.0 - (delta * 0.25);

        eff.seeds[i].x += eff.seeds[i].vx * delta;
        eff.seeds[i].y += eff.seeds[i].vy * delta;
    }

    // Proximity shatters
    let mut seed_explosions = Vec::new();
    for seed in &mut eff.seeds {
        if !seed.active || seed.is_black_hole { continue; }
        
        for lp in &mut eff.logo_pixels {
            if !lp.active { continue; }
            
            let (lp_ux, lp_uy) = to_universe(lp.x, lp.y, eff.universe_cx, eff.universe_cy, eff.zoom, cols, rows);
            let dx = lp_ux - seed.x;
            let dy = lp_uy - seed.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < 2.0 {
                seed.active = false;
                lp.exc = 1.0;
                seed_explosions.push((seed.x, seed.y, seed.color, seed.vx, seed.vy));
                break;
            }
        }
    }

    for (sx, sy, color, _vx, _vy) in seed_explosions {
        let ox = sx - eff.universe_cx;
        let oy = sy - eff.universe_cy;
        let o_len = (ox * ox + oy * oy).sqrt().max(0.1);
        let dir_x = ox / o_len;
        let dir_y = oy / o_len;

        let count = 40;
        for _ in 0..count {
            let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
            let speed = eff.rng.next_range(60.0, 110.0);
            eff.particles.push(Particle {
                x: sx,
                y: sy,
                vx: (dir_x * 0.8 + angle.cos() * 0.2) * speed,
                vy: (dir_y * 0.8 + angle.sin() * 0.2) * speed * 0.45,
                mass: 0.6,
                color: if eff.rng.next_bool(0.3) { (255, 255, 255) } else { color },
                ch: if eff.rng.next_bool(0.4) { '*' } else if eff.rng.next_bool(0.5) { '+' } else { '·' },
                history: Vec::new(),
            });
        }
    }

    // Star merge check
    handle_seed_merges(eff, delta, dir, seeds_len);

    // Particles gravitate to active seeds
    for p in &mut eff.particles {
        let mut fx = 0.0f32;
        let mut fy = 0.0f32;
        for seed in &eff.seeds {
            if !seed.active { continue; }
            let dx = seed.x - p.x;
            let dy = seed.y - p.y;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt().max(0.1);
            
            let mass_multiplier = if seed.is_black_hole { 1.8 } else { 1.0 };
            let force = (seed.mass * 22.0 * mass_multiplier) / (dist_sq + 18.0);
            fx += (dx / dist) * force;
            fy += (dy / dist) * force;
        }

        for lp in &eff.logo_pixels {
            if !lp.active { continue; }
            let (lp_ux, lp_uy) = to_universe(lp.x, lp.y, eff.universe_cx, eff.universe_cy, eff.zoom, cols, rows);
            let dx = lp_ux - p.x;
            let dy = lp_uy - p.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < 256.0 {
                let dist = dist_sq.sqrt().max(0.1);
                let force = 0.45 / (dist_sq + 5.0);
                fx += (dx / dist) * force;
                fy += (dy / dist) * force;
            }
        }

        p.vx += (fx * delta) / p.mass;
        p.vy += (fy * delta) / p.mass;
        
        p.vx *= 1.0 - (delta * 0.40);
        p.vy *= 1.0 - (delta * 0.40);

        p.x += p.vx * delta;
        p.y += p.vy * delta;

        let cx_i = p.x.round() as i32;
        let cy_i = p.y.round() as i32;
        p.history.push((cx_i, cy_i));
        if p.history.len() > 4 { p.history.remove(0); }
    }

    // Accrete particles
    let mut new_sparks = Vec::new();
    let current_total_particles = eff.particles.len();
    let palette = query_current_palette();
    let accent = palette.accent;
    eff.particles.retain_mut(|p| {
        for lp in &eff.logo_pixels {
            if lp.active {
                let (lp_ux, lp_uy) = to_universe(lp.x, lp.y, eff.universe_cx, eff.universe_cy, eff.zoom, cols, rows);
                let dx = lp_ux - p.x;
                let dy = lp_uy - p.y;
                if dx * dx + dy * dy < 1.44 {
                    if current_total_particles + new_sparks.len() < 400 {
                        let ox = p.x - eff.universe_cx;
                        let oy = p.y - eff.universe_cy;
                        let o_len = (ox * ox + oy * oy).sqrt().max(0.1);
                        let dir_x = ox / o_len;
                        let dir_y = oy / o_len;

                        let spark_count = eff.rng.next_range(2.0, 4.0) as usize;
                        for _ in 0..spark_count {
                            let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                            let speed = eff.rng.next_range(50.0, 95.0);
                            new_sparks.push(Particle {
                                x: p.x,
                                y: p.y,
                                vx: (dir_x * 0.75 + angle.cos() * 0.25) * speed,
                                vy: (dir_y * 0.75 + angle.sin() * 0.25) * speed * 0.45,
                                mass: 0.5,
                                color: (accent.0.saturating_add(60), accent.1.saturating_add(60), 255),
                                ch: if eff.rng.next_bool(0.5) { '*' } else { '+' },
                                history: Vec::new(),
                            });
                        }
                    }
                    return false;
                }
            }
        }

        for seed in &mut eff.seeds {
            if seed.active {
                let dx = seed.x - p.x;
                let dy = seed.y - p.y;
                let dist_sq = dx * dx + dy * dy;
                if seed.is_black_hole {
                    if dist_sq < 2.25 {
                        if current_total_particles + new_sparks.len() < 350 {
                            let spark_count = eff.rng.next_range(2.0, 4.0) as usize;
                            for _ in 0..spark_count {
                                let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                                let speed = eff.rng.next_range(12.0, 24.0);
                                new_sparks.push(Particle {
                                    x: seed.x + angle.cos() * 1.6,
                                    y: seed.y + angle.sin() * 1.6 * 0.45,
                                    vx: angle.cos() * speed,
                                    vy: angle.sin() * speed * 0.45,
                                    mass: 0.5,
                                    color: (180, 100, 255),
                                    ch: if eff.rng.next_bool(0.5) { '+' } else { '·' },
                                    history: Vec::new(),
                                });
                            }
                        }
                        return false;
                    }
                } else {
                    if dist_sq < 1.44 {
                        seed.mass += 0.08;
                        if eff.rng.next_bool(0.4) && current_total_particles + new_sparks.len() < 350 {
                            let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                            let speed = eff.rng.next_range(8.0, 18.0);
                            new_sparks.push(Particle {
                                x: seed.x + angle.cos() * 1.3,
                                y: seed.y + angle.sin() * 1.3 * 0.45,
                                vx: angle.cos() * speed,
                                vy: angle.sin() * speed * 0.45,
                                mass: 0.4,
                                color: (255, 230, 150),
                                ch: '·',
                                history: Vec::new(),
                            });
                        }
                        return false;
                    }
                }
            }
        }
        true
    });
    eff.particles.extend(new_sparks);

    // Spawn orbital replenishment particles around black holes
    if eff.particles.len() < 250 {
        for seed in &eff.seeds {
            if seed.active && seed.is_black_hole && eff.rng.next_bool(0.12) {
                let dist = eff.rng.next_range(3.2, 7.5);
                let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                let px = seed.x + angle.cos() * dist;
                let py = seed.y + angle.sin() * dist * 0.45;
                
                let speed = (seed.mass * 12.0 / dist).sqrt();
                let tx = -angle.sin();
                let ty = angle.cos();
                
                let vx = tx * speed * dir;
                let vy = ty * speed * 0.45 * dir;
                
                let p_color = (
                    (seed.color.0 as i16 + eff.rng.next_range(-20.0, 20.0) as i16).clamp(0, 255) as u8,
                    (seed.color.1 as i16 + eff.rng.next_range(-20.0, 20.0) as i16).clamp(0, 255) as u8,
                    (seed.color.2 as i16 + eff.rng.next_range(-20.0, 20.0) as i16).clamp(0, 255) as u8,
                );
                eff.particles.push(Particle {
                    x: px,
                    y: py,
                    vx,
                    vy,
                    mass: eff.rng.next_range(0.4, 0.8),
                    color: p_color,
                    ch: if eff.rng.next_bool(0.5) { '·' } else { '.' },
                    history: Vec::new(),
                });
            }
        }
    }

    // Logo character drift
    handle_logo_character_drift(eff, delta, dir, cols, rows);

    // Stellar Ignition
    handle_nebular_stellar_ignition(eff, dir);
}
pub fn update_singularity(eff: &mut Cosmos, delta: f32, cols: usize, rows: usize) {
    let cols_f = cols as f32;
    let rows_f = rows as f32;
    let cx = cols_f / 2.0;
    let cy = rows_f / 2.0;
    let dir = if eff.spin_clockwise { 1.0f32 } else { -1.0f32 };

    if !eff.seeds.is_empty() {
        let seed = &mut eff.seeds[0];
        let dx_c = cx - seed.x;
        let dy_c = cy - seed.y;
        let dist_c = (dx_c * dx_c + dy_c * dy_c).sqrt().max(0.1);
        
        let mut sfx = (dx_c / dist_c) * 1.8;
        let mut sfy = (dy_c / dist_c) * 0.9;
        
        let orbit_force = 1.8f32;
        sfx += (dy_c / dist_c) * orbit_force * dir;
        sfy += (-dx_c / dist_c) * orbit_force * 0.45 * dir;
        
        seed.vx += sfx * delta;
        seed.vy += sfy * delta;
        seed.vx *= 1.0 - (delta * 0.15);
        seed.vy *= 1.0 - (delta * 0.15);
        
        seed.x += seed.vx * delta;
        seed.y += seed.vy * delta;
    }
    
    let (bh_x, bh_y) = if !eff.seeds.is_empty() {
        (eff.seeds[0].x, eff.seeds[0].y)
    } else {
        (cx, cy)
    };

    for p in &mut eff.particles {
        let dx = bh_x - p.x;
        let dy = bh_y - p.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.1);
        
        let pull = 110.0 / (dist + 2.0);
        let tangent = 45.0 / (dist.sqrt() + 1.0);

        p.vx += ((dx / dist) * pull + (dy / dist) * tangent * dir) * delta;
        p.vy += ((dy / dist) * pull * 0.45 - (dx / dist) * tangent * 0.45 * dir) * delta;

        p.vx *= 1.0 - (delta * 1.5);
        p.vy *= 1.0 - (delta * 1.5);

        p.x += p.vx * delta;
        p.y += p.vy * delta;

        let cx_i = p.x.round() as i32;
        let cy_i = p.y.round() as i32;
        p.history.push((cx_i, cy_i));
        if p.history.len() > 4 { p.history.remove(0); }
    }

    eff.particles.retain(|p| {
        let dx = bh_x - p.x;
        let dy = bh_y - p.y;
        dx * dx + dy * dy > 2.0
    });

    if eff.particles.len() < 200 && !eff.seeds.is_empty() && eff.rng.next_bool(0.15) {
        let seed = &eff.seeds[0];
        let dist = eff.rng.next_range(3.2, 7.5);
        let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
        let px = seed.x + angle.cos() * dist;
        let py = seed.y + angle.sin() * dist * 0.45;
        
        let speed = (seed.mass * 18.0 / dist).sqrt();
        let tx = -angle.sin();
        let ty = angle.cos();
        
        let vx = tx * speed * dir;
        let vy = ty * speed * 0.45 * dir;
        
        eff.particles.push(Particle {
            x: px,
            y: py,
            vx,
            vy,
            mass: eff.rng.next_range(0.4, 0.8),
            color: (160, 80, 255),
            ch: if eff.rng.next_bool(0.5) { '·' } else { '.' },
            history: Vec::new(),
        });
    }

    for lp in &mut eff.logo_pixels {
        if !lp.active { continue; }
        let (bh_sx, bh_sy) = to_screen(bh_x, bh_y, eff.universe_cx, eff.universe_cy, eff.zoom, cols, rows);
        let dx = bh_sx as f32 - lp.x;
        let dy = bh_sy as f32 - lp.y;
        let dist = (dx*dx + dy*dy).sqrt().max(0.1);
        
        lp.exc = (1.0 - dist / 12.0).clamp(0.0, 1.0);
        
        if dist > 1.6 {
            let pull = 18.0 / (dist + 2.0);
            let tangent = 12.0 / (dist.sqrt() + 1.0);
            
            lp.vx += ((dx / dist) * pull + (dy / dist) * tangent * dir) * 4.5 * delta;
            lp.vy += ((dy / dist) * pull - (dx / dist) * tangent * 0.45 * dir) * 4.5 * delta;
            
            lp.vx *= 1.0 - (1.5 * delta);
            lp.vy *= 1.0 - (1.5 * delta);
            
            lp.x += lp.vx * delta;
            lp.y += lp.vy * delta;
        } else {
            lp.active = false;
        }
    }
}

pub fn update_collapse(eff: &mut Cosmos, delta: f32, cols: usize, rows: usize) {
    let cols_f = cols as f32;
    let rows_f = rows as f32;
    let cx = cols_f / 2.0;
    let cy = rows_f / 2.0;

    if !eff.seeds.is_empty() {
        let seed = &mut eff.seeds[0];
        seed.x += (cx - seed.x) * 4.0 * delta;
        seed.y += (cy - seed.y) * 4.0 * delta;
    }

    let bh_x = if !eff.seeds.is_empty() { eff.seeds[0].x } else { cx };
    let bh_y = if !eff.seeds.is_empty() { eff.seeds[0].y } else { cy };
    let dir = if eff.spin_clockwise { 1.0f32 } else { -1.0f32 };
    
    for p in &mut eff.particles {
        let dx = bh_x - p.x;
        let dy = bh_y - p.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.1);
        
        let pull = (220.0 + eff.state_timer * 120.0) / (dist + 1.0);
        let tangent = (60.0 - eff.state_timer * 18.0).max(0.0) / (dist.sqrt() + 1.0);
        
        p.vx += ((dx / dist) * pull + (dy / dist) * tangent * dir) * delta;
        p.vy += ((dy / dist) * pull * 0.45 - (dx / dist) * tangent * 0.45 * dir) * delta;
        
        let drag = 2.5 + eff.state_timer * 2.0;
        p.vx *= 1.0 - (delta * drag);
        p.vy *= 1.0 - (delta * drag);
        
        p.x += p.vx * delta;
        p.y += p.vy * delta;
        
        let cx_i = p.x.round() as i32;
        let cy_i = p.y.round() as i32;
        p.history.push((cx_i, cy_i));
        if p.history.len() > 4 { p.history.remove(0); }
    }
    
    eff.particles.retain(|p| {
        let dx = bh_x - p.x;
        let dy = bh_y - p.y;
        dx * dx + dy * dy > 1.5
    });
    
    for lp in &mut eff.logo_pixels {
        if !lp.active { continue; }
        let (bh_sx, bh_sy) = to_screen(bh_x, bh_y, eff.universe_cx, eff.universe_cy, eff.zoom, cols, rows);
        let dx = bh_sx as f32 - lp.x;
        let dy = bh_sy as f32 - lp.y;
        let dist = (dx*dx + dy*dy).sqrt().max(0.1);
        
        lp.exc = (1.0 - dist / 8.0).clamp(0.0, 1.0);
        
        if dist > 1.2 {
            let pull = (45.0 + eff.state_timer * 25.0) / (dist + 1.0);
            let tangent = (16.0 - eff.state_timer * 4.0).max(0.0) / (dist.sqrt() + 1.0);
            
            lp.vx += ((dx / dist) * pull + (dy / dist) * tangent * dir) * 6.0 * delta;
            lp.vy += ((dy / dist) * pull - (dx / dist) * tangent * 0.45 * dir) * 6.0 * delta;
            
            let drag = 1.5 + eff.state_timer * 1.5;
            lp.vx *= 1.0 - (drag * delta);
            lp.vy *= 1.0 - (drag * delta);
            
            lp.x += lp.vx * delta;
            lp.y += lp.vy * delta;
        } else {
            lp.active = false;
        }
    }
}
pub fn handle_seed_merges(eff: &mut Cosmos, _delta: f32, _dir: f32, seeds_len: usize) {
    let mut spawn_sparks = Vec::new();
    for i in 0..seeds_len {
        if !eff.seeds[i].active { continue; }
        for j in (i+1)..seeds_len {
            if !eff.seeds[j].active { continue; }
            let dx = eff.seeds[j].x - eff.seeds[i].x;
            let dy = eff.seeds[j].y - eff.seeds[i].y;
            let merge_dist = 3.5 + (eff.seeds[i].mass + eff.seeds[j].mass) * 0.12;
            if dx*dx + dy*dy < merge_dist * merge_dist {
                // Merge j into i
                let m_i = eff.seeds[i].mass;
                let m_j = eff.seeds[j].mass;
                let new_mass = m_i + m_j;
                eff.seeds[i].x = (eff.seeds[i].x * m_i + eff.seeds[j].x * m_j) / new_mass;
                eff.seeds[i].y = (eff.seeds[i].y * m_i + eff.seeds[j].y * m_j) / new_mass;
                eff.seeds[i].vx = (eff.seeds[i].vx * m_i + eff.seeds[j].vx * m_j) / new_mass;
                eff.seeds[i].vy = (eff.seeds[i].vy * m_i + eff.seeds[j].vy * m_j) / new_mass;
                eff.seeds[i].mass = new_mass;
                eff.seeds[j].active = false;

                let was_i_bh = eff.seeds[i].is_black_hole;
                let was_j_bh = eff.seeds[j].is_black_hole;
                
                let merger_type = if was_i_bh && was_j_bh {
                    3
                } else if was_i_bh || was_j_bh {
                    2
                } else if new_mass >= 8.5 {
                    1
                } else {
                    0
                };
                
                let c_i = eff.seeds[i].color;
                let c_j = eff.seeds[j].color;
                let blended_color = (
                    (((c_i.0 as u16 + c_j.0 as u16) / 2) as u8),
                    (((c_i.1 as u16 + c_j.1 as u16) / 2) as u8),
                    (((c_i.2 as u16 + c_j.2 as u16) / 2) as u8),
                );

                spawn_sparks.push((eff.seeds[i].x, eff.seeds[i].y, merger_type, blended_color));

                if merger_type == 1 || merger_type == 2 || merger_type == 3 {
                    eff.seeds[i].is_black_hole = true;
                    eff.seeds[i].color = (130, 50, 240);
                    eff.seeds[i].birth_timer = 0.0;
                }
            }
        }
    }

    for (sx, sy, merger_type, color) in spawn_sparks {
        if merger_type == 3 {
            eff.grav_wave_timer = 1.2;
            eff.grav_wave_cx = sx;
            eff.grav_wave_cy = sy;
        }
        match merger_type {
            0 => {
                let count = 25;
                for _ in 0..count {
                    let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                    let speed = eff.rng.next_range(15.0, 32.0);
                    eff.particles.push(Particle {
                        x: sx,
                        y: sy,
                        vx: angle.cos() * speed,
                        vy: angle.sin() * speed * 0.45,
                        mass: 0.6,
                        color: (255, 235, 180),
                        ch: if eff.rng.next_bool(0.5) { '*' } else { '+' },
                        history: Vec::new(),
                    });
                }
            }
            1 => {
                for p in &mut eff.particles {
                    let dx = p.x - sx;
                    let dy = p.y - sy;
                    let dist_sq = dx * dx + dy * dy;
                    let dist = dist_sq.sqrt().max(0.1);
                    if dist < 22.0 {
                        let push = (22.0 - dist) * 5.0;
                        p.vx += (dx / dist) * push;
                        p.vy += (dy / dist) * push * 0.45;
                    }
                }
                for _ in 0..50 {
                    let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                    let speed = eff.rng.next_range(25.0, 48.0);
                    eff.particles.push(Particle {
                        x: sx,
                        y: sy,
                        vx: angle.cos() * speed,
                        vy: angle.sin() * speed * 0.45,
                        mass: 0.8,
                        color: (255, 120, 50),
                        ch: '░',
                        history: Vec::new(),
                    });
                }
                for _ in 0..25 {
                    let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                    let speed = eff.rng.next_range(15.0, 30.0);
                    eff.particles.push(Particle {
                        x: sx,
                        y: sy,
                        vx: angle.cos() * speed,
                        vy: angle.sin() * speed * 0.45,
                        mass: 0.5,
                        color: (255, 255, 255),
                        ch: '*',
                        history: Vec::new(),
                    });
                }
            }
            2 => {
                let count = 40;
                let flare_color = (
                    color.0.saturating_add(60),
                    color.1.saturating_add(60),
                    255
                );
                for _ in 0..count {
                    let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                    let speed = eff.rng.next_range(20.0, 40.0);
                    eff.particles.push(Particle {
                        x: sx,
                        y: sy,
                        vx: angle.cos() * speed,
                        vy: angle.sin() * speed * 0.45,
                        mass: 0.5,
                        color: flare_color,
                        ch: if eff.rng.next_bool(0.5) { '+' } else { '·' },
                        history: Vec::new(),
                    });
                }
            }
            3 => {
                for p in &mut eff.particles {
                    let dx = p.x - sx;
                    let dy = p.y - sy;
                    let dist_sq = dx * dx + dy * dy;
                    let dist = dist_sq.sqrt().max(0.1);
                    if dist < 32.0 {
                        let push = (32.0 - dist) * 7.5;
                        p.vx += (dx / dist) * push;
                        p.vy += (dy / dist) * push * 0.45;
                    }
                }
                for _ in 0..65 {
                    let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                    let speed = eff.rng.next_range(35.0, 65.0);
                    eff.particles.push(Particle {
                        x: sx,
                        y: sy,
                        vx: angle.cos() * speed,
                        vy: angle.sin() * speed * 0.45,
                        mass: 0.7,
                        color: (160, 80, 255),
                        ch: if eff.rng.next_bool(0.4) { '╬' } else if eff.rng.next_bool(0.5) { '═' } else { '─' },
                        history: Vec::new(),
                    });
                }
            }
            _ => {}
        }
    }
}

pub fn handle_logo_character_drift(eff: &mut Cosmos, delta: f32, dir: f32, cols: usize, rows: usize) {
    let palette = query_current_palette();
    let accent = palette.accent;
    let mut spawned_logo_fragments = Vec::new();

    for lp in &mut eff.logo_pixels {
        if !lp.active { continue; }
        lp.exc = (lp.exc - 1.2 * delta).max(0.0);
        for p in &eff.particles {
            let (p_sx, p_sy) = to_screen(p.x, p.y, eff.universe_cx, eff.universe_cy, eff.zoom, cols, rows);
            let dx = p_sx as f32 - lp.x;
            let dy = (p_sy as f32 - lp.y) * 2.0;
            if dx*dx + dy*dy < 4.0 {
                lp.exc = 1.0;
            }
        }

        let mut total_bh_weight = 0.0f32;
        let mut fx_bh = 0.0f32;
        let mut fy_bh = 0.0f32;

        for seed in &eff.seeds {
            if seed.active && seed.is_black_hole {
                let (bh_sx, bh_sy) = to_screen(seed.x, seed.y, eff.universe_cx, eff.universe_cy, eff.zoom, cols, rows);
                let dx = bh_sx as f32 - lp.x;
                let dy = bh_sy as f32 - lp.y;
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt().max(0.1);
                
                if dist < 12.0 {
                    lp.exc = 1.0;
                    
                    if dist > 1.8 {
                        let weight = 1.0 - (dist / 12.0);
                        total_bh_weight = total_bh_weight.max(weight);
                        
                        let pull = (seed.mass * 18.0) / (dist_sq + 6.0);
                        let tangent = (seed.mass * 12.0) / (dist.sqrt() + 2.0);
                        fx_bh += ((dx / dist) * pull + (dy / dist) * tangent * dir) * weight;
                        fy_bh += ((dy / dist) * pull - (dx / dist) * tangent * 0.45 * dir) * weight;
                    } else {
                        lp.active = false;
                        
                        for _ in 0..10 {
                            let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                            let speed = eff.rng.next_range(16.0, 32.0);
                            spawned_logo_fragments.push(Particle {
                                x: seed.x,
                                y: seed.y,
                                vx: angle.cos() * speed,
                                vy: angle.sin() * speed * 0.45,
                                mass: 0.5,
                                color: (
                                    (accent.0 as i16 + eff.rng.next_range(-20.0, 20.0) as i16).clamp(0, 255) as u8,
                                    (accent.1 as i16 + eff.rng.next_range(-20.0, 20.0) as i16).clamp(0, 255) as u8,
                                    (accent.2 as i16 + eff.rng.next_range(-20.0, 20.0) as i16).clamp(0, 255) as u8,
                                ),
                                ch: lp.ch,
                                history: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        let dx_spring = lp.origin_x - lp.x;
        let dy_spring = lp.origin_y - lp.y;
        let k = 5.0;
        
        let spring_weight = 1.0 - total_bh_weight;
        let fx_spring = dx_spring * k * spring_weight;
        let fy_spring = dy_spring * k * spring_weight;

        lp.vx += (fx_spring + fx_bh) * delta;
        lp.vy += (fy_spring + fy_bh) * delta;

        let drag = 2.0;
        lp.vx *= 1.0 - (drag * delta);
        lp.vy *= 1.0 - (drag * delta);
        
        lp.x += lp.vx * delta;
        lp.y += lp.vy * delta;
    }
    eff.particles.extend(spawned_logo_fragments);
}

pub fn handle_nebular_stellar_ignition(eff: &mut Cosmos, dir: f32) {
    if eff.state_timer > 1.0 && eff.particles.len() > 40 && eff.rng.next_bool(0.10) {
        let p_idx = eff.rng.next_range(0.0, eff.particles.len() as f32) as usize;
        let target_x = eff.particles[p_idx].x;
        let target_y = eff.particles[p_idx].y;
        
        let mut neighbors = Vec::new();
        for k in 0..eff.particles.len() {
            let dx = eff.particles[k].x - target_x;
            let dy = eff.particles[k].y - target_y;
            if dx * dx + dy * dy < 20.25 {
                neighbors.push(k);
            }
        }
        
        if neighbors.len() >= 12 {
            let mut sum_x = 0.0f32;
            let mut sum_y = 0.0f32;
            let mut sum_vx = 0.0f32;
            let mut sum_vy = 0.0f32;
            let mut sum_r = 0u32;
            let mut sum_g = 0u32;
            let mut sum_b = 0u32;
            
            for &idx in &neighbors {
                let p = &eff.particles[idx];
                sum_x += p.x;
                sum_y += p.y;
                sum_vx += p.vx;
                sum_vy += p.vy;
                sum_r += p.color.0 as u32;
                sum_g += p.color.1 as u32;
                sum_b += p.color.2 as u32;
            }
            
            let count_f = neighbors.len() as f32;
            let avg_x = sum_x / count_f;
            let avg_y = sum_y / count_f;
            let avg_vx = sum_vx / count_f;
            let avg_vy = sum_vy / count_f;
            let avg_color = (
                (sum_r / neighbors.len() as u32) as u8,
                (sum_g / neighbors.len() as u32) as u8,
                (sum_b / neighbors.len() as u32) as u8,
            );
            
            let dx = avg_x - eff.universe_cx;
            let dy = avg_y - eff.universe_cy;
            let dist = (dx * dx + dy * dy).sqrt().max(0.1);
            
            let tx = -dy / dist;
            let ty = dx / dist;
            
            let orbit_speed = (180.0 / dist).sqrt().clamp(4.0, 18.0);
            let orb_vx = tx * orbit_speed * dir;
            let orb_vy = ty * orbit_speed * 0.45 * dir;
            
            let new_vx = avg_vx * 0.2 + orb_vx * 0.8;
            let new_vy = avg_vy * 0.2 + orb_vy * 0.8;

            let new_star = GravityCenter {
                x: avg_x,
                y: avg_y,
                vx: new_vx,
                vy: new_vy,
                mass: (count_f * 0.35).clamp(1.5, 6.0),
                color: avg_color,
                active: true,
                is_black_hole: false,
                birth_timer: 0.0,
            };
            eff.seeds.push(new_star);
            
            let mut to_remove = vec![false; eff.particles.len()];
            for &idx in &neighbors {
                to_remove[idx] = true;
            }
            
            let mut i = 0;
            eff.particles.retain(|_| {
                let keep = !to_remove[i];
                i += 1;
                keep
            });
            
            let spark_color = (avg_color.0.saturating_add(80), avg_color.1.saturating_add(80), 255);
            for _ in 0..15 {
                let angle = eff.rng.next_range(0.0, std::f32::consts::TAU);
                let speed = eff.rng.next_range(12.0, 24.0);
                eff.particles.push(Particle {
                    x: avg_x,
                    y: avg_y,
                    vx: avg_vx + angle.cos() * speed,
                    vy: avg_vy + angle.sin() * speed * 0.45,
                    mass: 0.5,
                    color: spark_color,
                    ch: '+',
                    history: Vec::new(),
                });
            }
        }
    }
}




pub fn draw_life(effect: &Cosmos, grid: &mut [TerminalCell], cols: usize, rows: usize) {
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


pub fn draw_particles_and_trails(
    effect: &Cosmos,
    grid: &mut [TerminalCell],
    cols: usize,
    rows: usize,
    dim: f32,
) {
    if dim <= 0.001 {
        return;
    }
    // Trails
    for p in &effect.particles {
        let hist_len = p.history.len();
        for (k, &(hx, hy)) in p.history.iter().enumerate() {
            let (sx, sy) = to_screen(
                hx as f32,
                hy as f32,
                effect.universe_cx,
                effect.universe_cy,
                effect.zoom,
                cols,
                rows,
            );
            if sx >= 0 && sx < cols as i32 && sy >= 0 && sy < rows as i32 {
                let idx = sy as usize * cols + sx as usize;
                if grid[idx].ch == ' ' {
                    let t = (k + 1) as f32 / (hist_len + 1) as f32;
                    let intensity = t * 0.35 * dim;
                    let tr = (p.color.0 as f32 * intensity) as u8;
                    let tg = (p.color.1 as f32 * intensity) as u8;
                    let tb = (p.color.2 as f32 * intensity) as u8;
                    grid[idx] = TerminalCell {
                        ch: '·',
                        fg: (tr, tg, tb),
                        bg: (0, 0, 0),
                        bold: false,
                    };
                }
            }
        }
    }

    // Particle Core
    for p in &effect.particles {
        let (sx, sy) = to_screen(
            p.x,
            p.y,
            effect.universe_cx,
            effect.universe_cy,
            effect.zoom,
            cols,
            rows,
        );
        if sx >= 0 && sx < cols as i32 && sy >= 0 && sy < rows as i32 {
            let idx = sy as usize * cols + sx as usize;
            if grid[idx].ch == ' ' || grid[idx].ch == '·' {
                let tr = (p.color.0 as f32 * dim) as u8;
                let tg = (p.color.1 as f32 * dim) as u8;
                let tb = (p.color.2 as f32 * dim) as u8;
                grid[idx] = TerminalCell {
                    ch: p.ch,
                    fg: (tr, tg, tb),
                    bg: (0, 0, 0),
                    bold: dim > 0.35,
                };
            }
        }
    }
}
