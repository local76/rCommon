use std::time::Duration;
use crate::core::screensaver::Screensaver;
use crate::core::{LcgRng, TerminalCell};
use crate::platform::native::sys_info::get_system_info;
use crate::role::application::rgb::{RgbController, is_openrgb_enabled};
use crate::role::application::rgb::protocol::RgbColor;
use super::types::{Rocket, Particle, Star, FIREWORK_COLORS};

pub struct Bursts {
    rng: LcgRng,
    pub(crate) rockets: Vec<Rocket>,
    pub(crate) particles: Vec<Particle>,
    pub(crate) stars: Vec<Star>,
    pub(crate) skyline: Vec<usize>, // Height of building at each column
    pub(crate) skyline_windows: Vec<bool>, // Whether window is lit at grid cell (r * cols + c)
    pub(crate) time_elapsed: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    launch_rate_opt: u32,
    skyline_style_opt: u32,

    // Live system dynamics
    sys_refresh_timer: f32,
    mem_pressure: f32,
    cpu_load: f32,
    host_bias: f32,
    rgb: Option<RgbController>,
}

impl Default for Bursts {
    fn default() -> Self {
        Self::new()
    }
}

impl Bursts {
    pub fn new() -> Self {
        // Pre-4.1 HKEY_CURRENT_USER registry reads (LaunchRate, SkylineStyle)
        // collapsed to defaults for the inline migration. Re-added in 4.2.
        let launch_rate_opt: u32 = 1;
        let skyline_style_opt: u32 = 0;

        let sys = get_system_info();
        let host_bias = sys.hostname.chars().map(|c| c as u32).sum::<u32>() as f32 / 1000.0 % 1.0;

        Self {
            rng: LcgRng::new(7777),
            rockets: Vec::new(),
            particles: Vec::new(),
            stars: Vec::new(),
            skyline: Vec::new(),
            skyline_windows: Vec::new(),
            time_elapsed: 0.0,
            last_cols: 0,
            last_rows: 0,
            launch_rate_opt,
            skyline_style_opt,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: 0.4,
            host_bias,
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
        }
    }

    fn generate_skyline(&mut self, cols: usize, rows: usize) {
        self.skyline = vec![0; cols];
        self.skyline_windows = vec![false; cols * rows];

        if self.skyline_style_opt == 1 {
            return; // Empty sky
        }

        let mut c = 0;
        while c < cols {
            let building_w = self.rng.next_usize(6) + 3; // 3 to 8 cols wide
            let building_h = self.rng.next_usize(rows / 4) + 3; // building height

            for i in 0..building_w {
                if c + i < cols {
                    self.skyline[c + i] = building_h;
                    
                    // Windows in this building
                    for r in 0..building_h {
                        let gy = rows.saturating_sub(1).saturating_sub(r);
                        if self.rng.next_bool(0.12) {
                            self.skyline_windows[gy * cols + (c + i)] = true;
                        }
                    }
                }
            }
            c += building_w + self.rng.next_usize(2); // gap between buildings
        }
    }
}

impl Screensaver for Bursts {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let delta = dt.as_secs_f32();
        self.time_elapsed += delta;

        // Live refresh: more launches under load, host_bias for variety
        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (self.mem_pressure * 0.6 + 0.3).min(0.9);
            if self.host_bias > 0.65 { self.cpu_load = (self.cpu_load + 0.1).min(0.98); }
            self.sys_refresh_timer = 0.0;
        }

        // Initialize skyline if resized
        if cols != self.last_cols || rows != self.last_rows {
            self.generate_skyline(cols, rows);
            self.rockets.clear();
            self.particles.clear();
            
            // Create background stars
            let target_stars = (cols * rows / 20).clamp(10, 80);
            let mut stars = Vec::new();
            for i in 0..target_stars {
                stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch: if i % 7 == 0 { '✦' } else if i % 3 == 0 { '•' } else { '.' },
                    excitation: 0.0,
                    excited_color: (255, 255, 255),
                });
            }
            self.stars = stars;

            self.last_cols = cols;
            self.last_rows = rows;
        }

        // 1. Launch new rockets randomly
        // Live: higher CPU/mem pressure = more frequent/larger fireworks show
        let load_mult = 1.0 + self.cpu_load * 0.7 + self.mem_pressure * 0.3;
        let (base_max, base_chance) = match self.launch_rate_opt {
            0 => (2, 0.015),
            2 => (7, 0.09),
            _ => (4, 0.04),
        };
        let max_rockets = (base_max as f32 * load_mult).max(1.0) as usize;
        let chance = base_chance * load_mult;
        if self.rockets.len() < max_rockets && self.rng.next_bool(chance) {
            let start_x = self.rng.next_range(5.0, cols as f32 - 5.0);
            let start_y = rows as f32 - 1.0;
            let target_y = self.rng.next_range(3.0, rows as f32 * 0.55);
            let color = FIREWORK_COLORS[self.rng.next_usize(FIREWORK_COLORS.len())];
            
            // Aim towards the middle
            let target_x = self.rng.next_range(cols as f32 * 0.25, cols as f32 * 0.75);
            let dx = target_x - start_x;
            let dy = target_y - start_y;
            let time_to_peak = self.rng.next_range(1.2, 1.8);
            let vx = dx / time_to_peak;
            let vy = dy / time_to_peak;

            self.rockets.push(Rocket {
                x: start_x,
                y: start_y,
                vx,
                vy,
                target_y,
                color,
            });
        }

        // 2. Update rockets
        let mut exploded_rockets = Vec::new();
        for (i, rocket) in self.rockets.iter_mut().enumerate() {
            rocket.x += rocket.vx * delta;
            rocket.y += rocket.vy * delta;

            // Spawn smoke/trail particles
            if self.rng.next_bool(0.4) {
                self.particles.push(Particle {
                    x: rocket.x,
                    y: rocket.y,
                    vx: self.rng.next_range(-0.5, 0.5),
                    vy: self.rng.next_range(0.2, 1.0),
                    color: (100, 100, 100),
                    ch: '.',
                    life: 0.6,
                    max_life: 0.6,
                });
            }

            if rocket.y <= rocket.target_y {
                exploded_rockets.push(i);
            }
        }

        // Process explosions
        for idx in exploded_rockets.into_iter().rev() {
            let rocket = self.rockets.remove(idx);
            if let Some(ref r) = self.rgb {
                let color = RgbColor::new(rocket.color.0, rocket.color.1, rocket.color.2);
                r.flash(color, std::time::Duration::from_millis(300));
            }
            
            // Spawn explosion particles
            let num_particles = self.rng.next_usize(20) + 20;
            for _ in 0..num_particles {
                let angle = self.rng.next_range(0.0, std::f32::consts::TAU);
                let speed = self.rng.next_range(4.0, 16.0);
                
                let vx = angle.cos() * speed / 0.55;
                let vy = angle.sin() * speed;

                let ch = match self.rng.next_usize(4) {
                    0 => '*',
                    1 => '+',
                    2 => '•',
                    _ => '.',
                };
                let max_life = self.rng.next_range(0.8, 1.5);

                self.particles.push(Particle {
                    x: rocket.x,
                    y: rocket.y,
                    vx,
                    vy,
                    color: rocket.color,
                    ch,
                    life: max_life,
                    max_life,
                });
            }
        }

        // Decay star excitations
        for star in &mut self.stars {
            if star.excitation > 0.0 {
                star.excitation -= delta * 2.0;
                if star.excitation < 0.0 {
                    star.excitation = 0.0;
                }
            }
        }

        // 3. Update explosion particles and check star excitation
        let cols_f = cols as f32;
        let rows_f = rows as f32;
        for p in &mut self.particles {
            p.x += p.vx * delta;
            p.y += p.vy * delta;
            
            p.vy += 4.5 * delta; // Gravity
            p.vx *= 1.0 - 0.5 * delta; // Drag
            p.vy *= 1.0 - 0.5 * delta;

            p.life -= delta;

            // Excite background stars (ignore smoke particles)
            if p.color != (100, 100, 100) && p.life > 0.0 {
                for star in &mut self.stars {
                    let sx = star.x * cols_f;
                    let sy = star.y * rows_f;
                    let dx = p.x - sx;
                    let dy = (p.y - sy) * 2.0;
                    let dist_sq = dx*dx + dy*dy;
                    if dist_sq < 9.0 {
                        let dist = dist_sq.sqrt();
                        let force = (1.0 - dist / 3.0) * 1.5;
                        if force > star.excitation {
                            star.excitation = force;
                            star.excited_color = p.color;
                        }
                    }
                }
            }
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        self.draw_impl(grid, cols, rows);
    }
}
