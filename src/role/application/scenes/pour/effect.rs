use std::time::Duration;
use crate::core::screensaver::Screensaver;
use crate::core::{LcgRng, TerminalCell};
use crate::platform::native::sys_info::get_system_info;
use crate::role::application::palette::query_current_palette;
use crate::role::application::rgb::RgbController;
use crate::role::application::rgb::protocol::RgbColor;
use super::types::{LogoCell, Drop, Splash, Phase, BirdState, Animal, SceneryCell};

pub struct Pour {
    pub(crate) rng: LcgRng,
    pub(crate) logo_cells: Vec<LogoCell>,
    pub(crate) drops: Vec<Drop>,
    pub(crate) splashes: Vec<Splash>,
    pub(crate) phase: Phase,
    pub(crate) phase_timer: f32,
    pub(crate) last_cols: usize,
    pub(crate) last_rows: usize,
    pub(crate) drop_count_opt: u32,
    pub(crate) assemble_speed_opt: u32,

    // Live system dynamics
    pub(crate) sys_refresh_timer: f32,
    pub(crate) mem_pressure: f32,
    pub(crate) cpu_load: f32,
    pub(crate) host_bias: f32,

    // Puddle accumulation
    pub(crate) puddle: Vec<f32>,
    pub(crate) puddle_color: Vec<(u8, u8, u8)>,

    // Wind dynamics
    pub(crate) wind: f32,

    // Lightning
    pub(crate) lightning_timer: f32,
    pub(crate) lightning_flash: f32,
    pub(crate) lightning_bolts: Vec<Vec<(usize, usize)>>,
    pub(crate) lightning_is_background: bool,
    pub(crate) lightning_delay: f32,

    // Scenery
    pub(crate) bg_cells: Vec<SceneryCell>,
    pub(crate) mid_scenery: Vec<SceneryCell>,
    pub(crate) fg_scenery: Vec<SceneryCell>,

    // Bird state
    pub(crate) bird_x: f32,
    pub(crate) bird_y: f32,
    pub(crate) bird_state: BirdState,
    pub(crate) bird_timer: f32,
    pub(crate) bird_wing_flap: bool,
    pub(crate) bird_vx: f32,
    pub(crate) bird_vy: f32,
    pub(crate) bird_perch_x: f32,
    pub(crate) bird_perch_y: f32,
    pub(crate) perch_points: Vec<(usize, usize)>,

    // Active Animal
    pub(crate) active_animal: Option<Animal>,
    pub(crate) animal_spawn_timer: f32,

    // Subtitles
    pub(crate) subtitle: String,
    pub(crate) subtitle_timer: f32,
    pub(crate) rgb: Option<RgbController>,
}

impl Pour {
    pub fn new() -> Self {
        // Pre-4.1 HKEY_CURRENT_USER registry reads (DropCount, AssembleSpeed)
        // collapsed to defaults for the inline migration. Re-added in 4.2.
        let drop_count_opt: u32 = 1;
        let assemble_speed_opt: u32 = 1;

        let sys = get_system_info();
        Self {
            rng: LcgRng::new(2468),
            logo_cells: Vec::new(),
            drops: Vec::new(),
            splashes: Vec::new(),
            phase: Phase::Building,
            phase_timer: 0.0,
            last_cols: 0,
            last_rows: 0,
            drop_count_opt,
            assemble_speed_opt,
            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            cpu_load: 0.4,
            host_bias: sys.hostname.chars().map(|c| c as u32).sum::<u32>() as f32 / 1000.0 % 1.0,
            puddle: Vec::new(),
            puddle_color: Vec::new(),
            wind: 0.0,
            lightning_timer: 0.0,
            lightning_flash: 0.0,
            lightning_bolts: Vec::new(),
            lightning_is_background: false,
            lightning_delay: 0.0,
            bg_cells: Vec::new(),
            mid_scenery: Vec::new(),
            fg_scenery: Vec::new(),
            bird_x: 0.0,
            bird_y: 0.0,
            bird_state: BirdState::Sitting,
            bird_timer: 0.0,
            bird_wing_flap: false,
            bird_vx: 0.0,
            bird_vy: 0.0,
            bird_perch_x: 0.0,
            bird_perch_y: 0.0,
            perch_points: Vec::new(),
            active_animal: None,
            animal_spawn_timer: 15.0,
            subtitle: String::new(),
            subtitle_timer: 0.0,
            rgb: Some(RgbController::new()),
        }
    }

    pub(crate) fn cold_rain_color(rng: &mut LcgRng) -> (u8, u8, u8) {
        let r = rng.next_range(0.0, 1.0);
        if r < 0.60 {
            let brightness = rng.next_range(60.0, 120.0);
            (
                (brightness * 0.8) as u8,
                (brightness * 0.9) as u8,
                brightness as u8,
            )
        } else if r < 0.90 {
            let b = rng.next_range(110.0, 180.0);
            let g = b * rng.next_range(0.6, 0.85);
            let r = g * rng.next_range(0.5, 0.7);
            (r as u8, g as u8, b as u8)
        } else {
            let val = rng.next_range(180.0, 230.0);
            (
                (val * 0.9) as u8,
                (val * 0.95) as u8,
                val as u8,
            )
        }
    }

    pub(crate) fn generate_scenery(rng: &mut LcgRng, cols: usize, rows: usize) -> (
        Vec<SceneryCell>,
        Vec<SceneryCell>,
        Vec<SceneryCell>,
    ) {
        let mut bg = Vec::new();
        let mut mid = Vec::new();
        let mut fg = Vec::new();
        if rows < 10 { return (bg, mid, fg); }

        let mountain_color = (18, 22, 28);
        let snow_color = (65, 75, 85);
        
        let mut mountain_heights = vec![0; cols];
        for (x, height) in mountain_heights.iter_mut().enumerate().take(cols) {
            let h = (rows as f32 * 0.20) + 
                    (x as f32 * 0.04).sin() * (rows as f32 * 0.08) + 
                    (x as f32 * 0.10).cos() * (rows as f32 * 0.03);
            *height = h.clamp(2.0, rows as f32 * 0.45) as usize;
        }

        for (x, &m_h) in mountain_heights.iter().enumerate().take(cols) {
            let peak_y = rows.saturating_sub(m_h + 3);
            
            for y in peak_y..rows.saturating_sub(2) {
                let ch = if y == peak_y {
                    if rng.next_bool(0.5) { '^' } else { '/' }
                } else if y == peak_y + 1 {
                    '.'
                } else {
                    ' '
                };
                
                let col = if y == peak_y { snow_color } else { mountain_color };
                if ch != ' ' {
                    bg.push((x, y, ch, col));
                }
            }
        }

        let mut bx = 4;
        while bx < cols - 4 {
            let base_y = rows.saturating_sub(3);
            let bg_h = rng.next_range(1.0, 3.0) as usize;
            let bg_tree_color = (rng.next_range(16.0, 24.0) as u8, rng.next_range(24.0, 32.0) as u8, rng.next_range(20.0, 28.0) as u8);
            let bg_trunk_color = (24, 20, 16);
            
            bg.push((bx, base_y, '|', bg_trunk_color));
            if bg_h > 1 {
                bg.push((bx, base_y - 1, '|', bg_trunk_color));
            }
            
            let foliage_top = base_y.saturating_sub(bg_h);
            bg.push((bx, foliage_top, '▲', bg_tree_color));
            if bg_h > 1 {
                bg.push((bx - 1, foliage_top + 1, '▲', bg_tree_color));
                bg.push((bx + 1, foliage_top + 1, '▲', bg_tree_color));
            }
            bx += rng.next_range(6.0, 14.0) as usize;
        }

        let mid_tree_color = (25, 38, 28);
        let mid_trunk_color = (32, 28, 24);
        
        let mut mx = 12;
        while mx < cols - 8 {
            let tree_h = rng.next_range(2.0, 3.5) as usize;
            let base_y = rows.saturating_sub(3);
            
            mid.push((mx, base_y, '║', mid_trunk_color));
            for h_offset in 1..=tree_h {
                let foliage_y = base_y.saturating_sub(h_offset);
                mid.push((mx, foliage_y, '▲', mid_tree_color));
                if h_offset > 1 {
                    if mx > 0 { mid.push((mx - 1, foliage_y, '▲', mid_tree_color)); }
                    if mx < cols - 1 { mid.push((mx + 1, foliage_y, '▲', mid_tree_color)); }
                }
            }
            mx += rng.next_range(8.0, 15.0) as usize;
        }

        let fg_tree_color = (35, 55, 40);
        let trunk_color = (48, 42, 36);
        
        let mut fx = cols.saturating_sub(22);
        while fx < cols - 3 {
            let tree_h = rng.next_range(2.0, 4.0) as usize;
            let base_y = rows.saturating_sub(3);
            
            fg.push((fx, base_y, '║', trunk_color));
            
            for h_offset in 1..=tree_h {
                let foliage_y = base_y.saturating_sub(h_offset);
                fg.push((fx, foliage_y, '▲', fg_tree_color));
                if h_offset > 1 {
                    if fx > 0 { fg.push((fx - 1, foliage_y, '▲', fg_tree_color)); }
                    if fx < cols - 1 { fg.push((fx + 1, foliage_y, '▲', fg_tree_color)); }
                }
            }
            fx += rng.next_range(7.0, 12.0) as usize;
        }

        let tree_x = 8;
        if cols > 20 {
            let base_y = rows.saturating_sub(3);
            let trunk_top = base_y.saturating_sub(4);
            for y in trunk_top..=base_y {
                fg.push((tree_x, y, '║', trunk_color));
            }
            let branch_y = base_y.saturating_sub(2);
            fg.push((tree_x + 1, branch_y, '═', trunk_color));
            fg.push((tree_x + 2, branch_y, '═', trunk_color));
            
            let foliage_base = base_y.saturating_sub(4);
            fg.push((tree_x, foliage_base - 2, '▲', fg_tree_color));
            for dx in -1..=1 {
                fg.push(((tree_x as i32 + dx) as usize, foliage_base - 1, '▲', fg_tree_color));
            }
            for dx in -2..=2 {
                fg.push(((tree_x as i32 + dx) as usize, foliage_base, '▲', fg_tree_color));
            }
        }

        (bg, mid, fg)
    }
}

impl Screensaver for Pour {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let delta = dt.as_secs_f32();
        self.phase_timer += delta;

        self.wind = (self.phase_timer * 0.35).sin() * 9.0 + (self.phase_timer * 1.5).cos() * 2.0;

        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 1.0 {
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
            self.cpu_load = (self.mem_pressure * 0.6 + 0.3).min(0.9);
            if self.host_bias > 0.6 { self.cpu_load = (self.cpu_load + 0.08).min(0.95); }
            self.sys_refresh_timer = 0.0;

            if let Some(ref r) = self.rgb {
                // library 4.0: pull from the cached ScreenPalette.
                let accent = query_current_palette().accent;
                r.set_color(RgbColor::new(accent.0 / 4, accent.1 / 4, accent.2 / 4));
            }
        }

        self.check_resize(cols, rows);

        let load_mult = 1.0 + self.cpu_load * 0.6 + self.mem_pressure * 0.3;
        let speed_mult = match self.assemble_speed_opt {
            0 => 0.6f32,
            2 => 1.6f32,
            _ => 1.0f32,
        } * load_mult;

        self.update_drops(delta, cols, rows, speed_mult);
        self.update_bird(delta, cols, rows);
        self.update_scenery_and_animals(delta, cols, rows);
        self.update_lightning(delta, cols, rows);
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        self.draw_impl(grid, cols, rows);
    }
}
