use std::time::Duration;
use crate::core::screensaver::Screensaver;
use crate::core::{LcgRng, TerminalCell};
use crate::platform::native::sys_info::get_system_info;
use crate::role::application::rgb::{RgbController, is_openrgb_enabled};

use super::types::{UniverseState, Particle, GravityCenter, LogoPixel};
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


impl Screensaver for Cosmos {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        self::core::update_life(self, dt, cols, rows);
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        super::render::draw_life(self, grid, cols, rows);
    }
}
