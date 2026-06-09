use std::time::Duration;
use crate::core::screensaver::Screensaver;
use crate::core::{LcgRng, TerminalCell};
use crate::core::logo_block::render_logo_block;
use crate::platform::native::sys_info::get_system_info;
use crate::role::application::rgb::RgbController;
use super::types::{Particle, Star, Phase, ExplosionType};

pub struct Unstable {
    pub(crate) rng: LcgRng,
    pub(crate) particles: Vec<Particle>,
    pub(crate) stars: Vec<Star>,
    pub(crate) phase: Phase,
    pub(crate) phase_timer: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    pub(crate) explosion_type: ExplosionType,
    pub(crate) black_hole_burst_triggered: bool,
    pub(crate) particle_limit_opt: u32,
    pub(crate) explosion_freq_opt: u32,

    // Live system dynamics
    pub(crate) sys_refresh_timer: f32,
    pub(crate) mem_pressure: f32,
    pub(crate) cpu_load: f32,
    pub(crate) host_bias: f32,
    pub(crate) rgb: Option<RgbController>,
    pub(crate) rgb_timer: f32,
    pub(crate) last_phase: Option<Phase>,
    pub(crate) time_elapsed: f32,
}

impl Unstable {
    pub fn new() -> Self {
        // Pre-4.1 HKEY_CURRENT_USER registry reads (ParticleLimit, ExplosionFreq)
        // collapsed to defaults for the inline migration. Re-added in 4.2.
        let particle_limit_opt: u32 = 1;
        let explosion_freq_opt: u32 = 1;

        let sys = get_system_info();
        let host_bias = sys.hostname.chars().map(|c| c as u32).sum::<u32>() as f32 / 1000.0 % 1.0;

        Self {
            rng: LcgRng::new(8888),
            particles: Vec::new(),
            stars: Vec::new(),
            phase: Phase::Assembled,
            phase_timer: 0.0,
            last_cols: 0,
            last_rows: 0,
            explosion_type: ExplosionType::Supernova,
            black_hole_burst_triggered: false,
            particle_limit_opt,
            explosion_freq_opt,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: 0.4,
            host_bias,
            rgb: Some(RgbController::new()),
            rgb_timer: 0.0,
            last_phase: None,
            time_elapsed: 0.0,
        }
    }
}

impl Screensaver for Unstable {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let delta = dt.as_secs_f32();
        self.phase_timer += delta;
        self.time_elapsed += delta;

        // OpenRGB unstable phase-based updates
        self.rgb_timer += delta;
        if self.rgb_timer >= 0.08 {
            self.rgb_timer = 0.0;
            if let Some(ref r) = self.rgb {
                use crate::role::application::rgb::protocol::RgbColor;
                // library 4.0: pull from the canonical ScreenPalette.
                let accent = crate::role::application::palette::query_current_palette().accent;
                
                if Some(self.phase) != self.last_phase {
                    self.last_phase = Some(self.phase);
                    if self.phase == Phase::Exploding {
                        let (flash_c, dur_ms) = match self.explosion_type {
                            ExplosionType::Supernova => (RgbColor::new(255, 255, 220), 500),
                            ExplosionType::BlackHole => (RgbColor::new(120, 0, 255), 600),
                            ExplosionType::Vortex => (RgbColor::new(0, 180, 255), 450),
                            ExplosionType::GlitchWave => (RgbColor::new(255, 0, 0), 300),
                            ExplosionType::Shockwave => (RgbColor::new(0, 255, 180), 400),
                            ExplosionType::Entropy => (RgbColor::new(255, 128, 0), 500),
                            ExplosionType::Resonance => (RgbColor::new(255, 0, 128), 450),
                        };
                        r.flash(flash_c, Duration::from_millis(dur_ms));
                    }
                }
                
                // Set baseline color in non-exploding phases
                if self.phase != Phase::Exploding {
                    match self.phase {
                        Phase::Assembled => {
                            r.set_color(RgbColor::new(accent.0 / 2, accent.1 / 2, accent.2 / 2));
                        }
                        Phase::Chaos => {
                            // Glitchy orange/red flicker
                            let r_val = self.rng.next_range(100.0, 220.0) as u8;
                            let g_val = self.rng.next_range(20.0, 100.0) as u8;
                            r.set_color(RgbColor::new(r_val, g_val, 0));
                        }
                        Phase::SnapBack => {
                            // Smoothly blend from red/orange back to theme accent color
                            let progress = (self.phase_timer / 1.5).clamp(0.0, 1.0);
                            let red = (accent.0 as f32 * progress + 150.0 * (1.0 - progress)) as u8;
                            let green = (accent.1 as f32 * progress + 50.0 * (1.0 - progress)) as u8;
                            let blue = (accent.2 as f32 * progress) as u8;
                            r.set_color(RgbColor::new(red / 2, green / 2, blue / 2));
                        }
                        Phase::Exploding => {}
                    }
                }
            }
        }

        // Live: high system load = more chaos/explosions, host_bias unique instability
        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (self.mem_pressure * 0.6 + 0.3).min(0.9);
            if self.host_bias > 0.6 { self.cpu_load = (self.cpu_load + 0.1).min(0.98); }
            self.sys_refresh_timer = 0.0;
        }

        // Reinitialize if screen size changed
        if cols != self.last_cols || rows != self.last_rows {
            self.particles.clear();
            self.stars.clear();
            // library 4.1: render the centered system logo from the live OS info
            // (replaces pre-4.1 `trance_core::logo_lines()`).
            let logo_text = get_system_info().logo_text;
            let lines = render_logo_block(&logo_text, None);
            let logo_h = lines.len();
            let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
            let logo_x = cols.saturating_sub(logo_w) / 2;
            let logo_y = rows.saturating_sub(logo_h) / 2;

            for (r_offset, line) in lines.iter().enumerate().take(logo_h) {
                for (c_offset, ch) in line.chars().enumerate() {
                    if ch != ' ' {
                        if self.particle_limit_opt == 0 && self.rng.next_bool(0.5) {
                            continue;
                        }
                        let hx = (logo_x + c_offset) as f32;
                        let hy = (logo_y + r_offset) as f32;
                        self.particles.push(Particle {
                            home_x: hx,
                            home_y: hy,
                            x: hx,
                            y: hy,
                            vx: 0.0,
                            vy: 0.0,
                            ch,
                            orig_ch: ch,
                            glow: 0.0,
                            snapped: true,
                        });
                    }
                }
            }

            // Create background stars
            let target_stars = (cols * rows / 16).clamp(20, 100);
            for i in 0..target_stars {
                self.stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch: if i % 8 == 0 { '✦' } else if i % 3 == 0 { '•' } else { '.' },
                    excitation: 0.0,
                });
            }

            self.phase = Phase::Assembled;
            self.phase_timer = 0.0;
            self.last_cols = cols;
            self.last_rows = rows;
        }

        // Decay star excitations
        for star in &mut self.stars {
            if star.excitation > 0.0 {
                star.excitation -= delta * 2.5;
                if star.excitation < 0.0 {
                    star.excitation = 0.0;
                }
            }
        }

        // Particle-star proximity interaction: unsnapped particles excite nearby stars
        let cols_f = cols as f32;
        let rows_f = rows as f32;
        let star_excite_mult = match self.explosion_type {
            ExplosionType::Shockwave => 2.2,
            ExplosionType::Entropy => 1.8,
            ExplosionType::Resonance => 1.4,
            ExplosionType::BlackHole => 0.7,
            _ => 1.5,
        };
        for p in &self.particles {
            if !p.snapped {
                for star in &mut self.stars {
                    let sx = star.x * cols_f;
                    let sy = star.y * rows_f;
                    let dx = p.x - sx;
                    let dy = (p.y - sy) * 2.0;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < 9.0 {
                        let dist = dist_sq.sqrt();
                        let force = (1.0 - dist / 3.0) * 1.5 * star_excite_mult;
                        star.excitation = star.excitation.max(force);
                    }
                }
            }
        }

        // Particle dynamics update based on phase
        match self.phase {
            Phase::Assembled => {
                self.update_assembled(delta);
            }
            Phase::Exploding => {
                self.update_exploding(cols, rows);
            }
            Phase::Chaos => {
                self.update_chaos(delta, cols, rows);
            }
            Phase::SnapBack => {
                self.update_snapback(delta);
            }
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        self.draw_impl(grid, cols, rows);
    }
}
