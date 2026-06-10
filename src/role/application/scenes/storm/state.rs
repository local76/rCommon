use std::time::Duration;
use crate::core::screensaver::Screensaver;
use crate::core::{LcgRng, TerminalCell};
use crate::platform::native::sys_info::get_system_info;
use crate::role::application::palette::query_current_palette;
use crate::role::application::rgb::{RgbController, is_openrgb_enabled};
use crate::role::application::rgb::protocol::RgbColor;
use super::types::{LogoCell, Drop, Splash, Phase, BirdState, Animal, SceneryCell};
pub struct Storm {
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

impl Default for Storm {
    fn default() -> Self {
        Self::new()
    }
}

impl Storm {
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
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
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

    pub fn check_resize(&mut self, cols: usize, rows: usize) {
        if cols != self.last_cols || rows != self.last_rows {
            self.logo_cells.clear();
            self.splashes.clear();
            self.drops.clear();
            self.puddle = vec![0.0f32; cols];
            self.puddle_color = vec![(0u8, 0u8, 0u8); cols];

            // library 4.1: render the centered system logo from the live OS info
            // (replaces pre-4.1 `trance_core::logo_lines()` + `logo_dimensions()`).
            let logo_text = get_system_info().logo_text;
            let lines = render_logo_block(&logo_text, None);
            let logo_h = lines.len();
            let logo_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
            let logo_x = cols.saturating_sub(logo_w) / 2;
            let logo_y = rows.saturating_sub(logo_h) / 2;

            for (r_offset, line) in lines.iter().enumerate().take(logo_h) {
                for (c_offset, ch) in line.chars().enumerate() {
                    if ch != ' ' {
                        self.logo_cells.push(LogoCell {
                            x: logo_x + c_offset,
                            y: logo_y + r_offset,
                            ch,
                            active: true,
                            glow: 0.0,
                            water: 0.0,
                        });
                    }
                }
            }

            self.phase = Phase::Complete;
            self.phase_timer = 0.0;
            self.last_cols = cols;
            self.last_rows = rows;
            let (bg, mid, fg) = Self::generate_scenery(&mut self.rng, cols, rows);
            self.bg_cells = bg;
            self.mid_scenery = mid;
            self.fg_scenery = fg;

            // Populate all perch points (Big Tree branch + top of logo cells)
            let tree_x = 8;
            let mut perch_points = Vec::new();
            perch_points.push((tree_x + 2, rows.saturating_sub(5)));
            for cell in &self.logo_cells {
                let has_above = self.logo_cells.iter().any(|c| c.x == cell.x && c.y == cell.y - 1);
                if !has_above && cell.y > 0 {
                    perch_points.push((cell.x, cell.y - 1));
                }
            }
            self.perch_points = perch_points;

            // Choose starting perch point
            let p_idx = self.rng.next_usize(self.perch_points.len());
            self.bird_perch_x = self.perch_points[p_idx].0 as f32;
            self.bird_perch_y = self.perch_points[p_idx].1 as f32;
            self.bird_x = self.bird_perch_x;
            self.bird_y = self.bird_perch_y;
            self.bird_state = BirdState::Sitting;
            self.bird_timer = self.rng.next_range(5.0, 15.0);
            self.bird_wing_flap = false;
        }
    }

    pub fn update_drops(&mut self, delta: f32, cols: usize, rows: usize, speed_mult: f32) {
        // Adjust drop count (increased for heavy cold rain)
        let target_drops = match self.drop_count_opt {
            0 => (cols).clamp(20, 100),
            2 => (cols * 3).clamp(60, 400),
            _ => (cols * 2).clamp(40, 250),
        };

        if self.drops.len() < target_drops {
            while self.drops.len() < target_drops {
                let x = if self.phase == Phase::Building && self.rng.next_bool(0.6) {
                    let inactive: Vec<&LogoCell> = self.logo_cells.iter().filter(|c| !c.active).collect();
                    if !inactive.is_empty() {
                        let selected = inactive[self.rng.next_usize(inactive.len())];
                        selected.x as f32
                    } else {
                        self.rng.next_range(0.0, cols as f32)
                    }
                } else {
                    self.rng.next_range(0.0, cols as f32)
                };

                let is_bg = self.rng.next_bool(0.5); // 50% background rain
                let mut color = Self::cold_rain_color(&mut self.rng);
                if is_bg {
                    color = (
                        (color.0 as f32 * 0.35) as u8,
                        (color.1 as f32 * 0.35) as u8,
                        (color.2 as f32 * 0.35) as u8,
                    );
                }
                
                self.drops.push(Drop {
                    x,
                    y: -self.rng.next_range(1.0, rows as f32),
                    vy: self.rng.next_range(25.0, 45.0) * speed_mult * (if is_bg { 0.75 } else { 1.0 }),
                    color,
                    is_background: is_bg,
                });
            }
        } else if self.drops.len() > target_drops {
            self.drops.truncate(target_drops);
        }

        // Update drops position & collisions
        let mut drops = std::mem::take(&mut self.drops);
        for drop in &mut drops {
            drop.y += drop.vy * delta;

            // Wind drifts the drop horizontally
            drop.x += self.wind * delta;
            // Wrap horizontally around the screen columns
            if drop.x < 0.0 {
                drop.x += cols as f32;
            } else if drop.x >= cols as f32 {
                drop.x -= cols as f32;
            }

            let col = drop.x as usize;
            let row = drop.y as usize;

            if col < cols && row < rows {
                // Background drops do NOT collide with foreground elements
                if !drop.is_background {
                    // Check if we hit any logo cell (whether active or inactive)
                    let mut hit = false;
                    for cell in &mut self.logo_cells {
                        if cell.x == col && cell.y == row {
                            if !cell.active && self.phase == Phase::Building {
                                cell.active = true;
                                cell.glow = 1.0;
                            }
                            
                            if cell.active {
                                // Rain water piles up on the active OS/Kernel cells
                                cell.water = (cell.water + 0.45).min(2.5);
                                
                                // Spawn splash particles
                                for _ in 0..3 {
                                    self.splashes.push(Splash {
                                        x: col as f32,
                                        y: row as f32,
                                        vx: self.rng.next_range(-3.0, 3.0),
                                        vy: self.rng.next_range(-2.0, -0.5),
                                        life: 0.5,
                                        color: drop.color,
                                        is_background: false,
                                    });
                                }

                                // Reset drop
                                let is_bg = self.rng.next_bool(0.5);
                                let mut color = Self::cold_rain_color(&mut self.rng);
                                if is_bg {
                                    color = (
                                        (color.0 as f32 * 0.35) as u8,
                                        (color.1 as f32 * 0.35) as u8,
                                        (color.2 as f32 * 0.35) as u8,
                                    );
                                }
                                drop.is_background = is_bg;
                                drop.color = color;
                                drop.y = -self.rng.next_range(1.0, rows as f32);
                                drop.vy = self.rng.next_range(25.0, 45.0) * speed_mult * (if is_bg { 0.75 } else { 1.0 });
                                hit = true;
                                break;
                            }
                        }
                    }
                    if hit {
                        continue;
                    }
                }
            }

            // Reset drop if it falls off bottom
            if drop.y >= (rows as f32 - 1.0) && cols > 0 {
                let col = (drop.x as usize).min(cols - 1);
                
                // Foreground drops spawn floor splash particles and accumulate puddles
                if !drop.is_background {
                    for _ in 0..2 {
                        self.splashes.push(Splash {
                            x: col as f32,
                            y: (rows as f32 - 1.0),
                            vx: self.rng.next_range(-4.0, 4.0),
                            vy: self.rng.next_range(-3.0, -1.0),
                            life: self.rng.next_range(0.3, 0.6),
                            color: drop.color,
                            is_background: false,
                        });
                    }
                    
                    // Accumulate puddle on the floor
                    if col < self.puddle.len() {
                        self.puddle[col] = (self.puddle[col] + 0.38).min(3.0);
                        let p_col = self.puddle_color[col];
                        let drop_color = drop.color;
                        self.puddle_color[col] = (
                            (p_col.0 as f32 * 0.6 + drop_color.0 as f32 * 0.4) as u8,
                            (p_col.1 as f32 * 0.6 + drop_color.1 as f32 * 0.4) as u8,
                            (p_col.2 as f32 * 0.6 + drop_color.2 as f32 * 0.4) as u8,
                        );
                    }
                } else {
                    // Background splashes
                    for _ in 0..1 {
                        self.splashes.push(Splash {
                            x: col as f32,
                            y: (rows as f32 - 1.0),
                            vx: self.rng.next_range(-2.0, 2.0),
                            vy: self.rng.next_range(-1.5, -0.5),
                            life: self.rng.next_range(0.2, 0.4),
                            color: drop.color,
                            is_background: true,
                        });
                    }
                }

                let is_bg = self.rng.next_bool(0.5);
                let mut color = Self::cold_rain_color(&mut self.rng);
                if is_bg {
                    color = (
                        (color.0 as f32 * 0.35) as u8,
                        (color.1 as f32 * 0.35) as u8,
                        (color.2 as f32 * 0.35) as u8,
                    );
                }
                drop.is_background = is_bg;
                drop.color = color;
                drop.y = -self.rng.next_range(1.0, rows as f32);
                drop.vy = self.rng.next_range(25.0, 45.0) * speed_mult * (if is_bg { 0.75 } else { 1.0 });
            }
        }
        self.drops = drops;

        // Update splashes
        for s in &mut self.splashes {
            s.x += s.vx * delta;
            s.y += s.vy * delta;
            // Splashes are blown slightly by the wind
            s.vx += self.wind * delta * 0.25;
            s.vy += 9.8 * delta;
            s.life -= delta;
        }
        self.splashes.retain(|s| s.life > 0.0);

        // Decay logo cell water and glow
        for cell in &mut self.logo_cells {
            if cell.glow > 0.0 {
                cell.glow -= delta * 1.5;
            }
            if cell.water > 0.0 {
                cell.water -= delta * 0.45;
                if cell.water < 0.0 {
                    cell.water = 0.0;
                }
            }
        }

        // Decay/drain puddles on the ground
        for x in 0..cols {
            if x < self.puddle.len() && self.puddle[x] > 0.0 {
                self.puddle[x] -= delta * 0.28;
                if self.puddle[x] < 0.0 {
                    self.puddle[x] = 0.0;
                }
            }
        }
    }

    pub fn update_bird(&mut self, delta: f32, cols: usize, rows: usize) {
        match self.bird_state {
            BirdState::Sitting => {
                self.bird_x = self.bird_perch_x;
                self.bird_y = self.bird_perch_y;
                self.bird_timer -= delta;
                if self.bird_timer <= 0.0 {
                    self.bird_state = BirdState::Flying;
                    self.bird_timer = self.rng.next_range(6.0, 12.0);
                    self.bird_vx = self.rng.next_range(4.0, 8.0);
                    self.bird_vy = self.rng.next_range(-4.0, -1.5);
                    if self.rng.next_bool(0.15) && self.subtitle_timer <= 0.0 {
                        self.subtitle = "[Bird took off]".to_string();
                        self.subtitle_timer = 1.5;
                    }
                }
            }
            BirdState::Flying => {
                self.bird_timer -= delta;
                self.bird_x += self.bird_vx * delta;
                self.bird_y += self.bird_vy * delta;

                if self.rng.next_bool(0.25) {
                    self.bird_wing_flap = !self.bird_wing_flap;
                }

                if self.bird_x >= cols as f32 || self.bird_y < 0.0 || self.bird_timer <= 0.0 {
                    self.bird_state = BirdState::Dead;
                    self.bird_timer = self.rng.next_range(8.0, 18.0);
                }
            }
            BirdState::Scared => {
                self.bird_timer -= delta;
                if self.bird_timer <= 0.0 {
                    self.bird_state = BirdState::Flying;
                    self.bird_timer = 4.0;
                    
                    let current_x = self.bird_x;
                    let current_y = self.bird_y;
                    self.bird_vx = self.rng.next_range(9.0, 15.0);
                    self.bird_vy = self.rng.next_range(-6.0, -3.0);
                    
                    if (current_x - self.bird_perch_x).abs() < 1.0 && (current_y - self.bird_perch_y).abs() < 1.0 {
                        self.bird_x = self.bird_perch_x + 1.0;
                    }
                }
            }
            BirdState::Explores => {
                self.bird_timer -= delta;
                self.bird_x += self.bird_vx * delta;
                self.bird_y += self.bird_vy * delta;

                if self.rng.next_bool(0.18) {
                    self.bird_vx = self.rng.next_range(-25.0, 25.0);
                    self.bird_vy = self.rng.next_range(-20.0, 20.0);
                }

                if self.bird_x < 1.0 {
                    self.bird_x = 1.0;
                    self.bird_vx = -self.bird_vx * 0.8;
                } else if self.bird_x >= (cols as f32 - 1.0) {
                    self.bird_x = cols as f32 - 2.0;
                    self.bird_vx = -self.bird_vx * 0.8;
                }

                if self.bird_y < 1.0 {
                    self.bird_y = 1.0;
                    self.bird_vy = -self.bird_vy * 0.8;
                } else if self.bird_y >= (rows as f32 - 1.0) {
                    self.bird_y = rows as f32 - 2.0;
                    self.bird_vy = -self.bird_vy * 0.8;
                }

                if self.rng.next_bool(0.25) {
                    self.bird_wing_flap = !self.bird_wing_flap;
                }

                if self.rng.next_bool(0.6) {
                    self.splashes.push(Splash {
                        x: self.bird_x,
                        y: self.bird_y,
                        vx: self.rng.next_range(-4.0, 4.0),
                        vy: self.rng.next_range(-4.0, 4.0),
                        life: self.rng.next_range(0.3, 0.6),
                        color: if self.rng.next_bool(0.5) { (255, 235, 140) } else { (100, 220, 255) },
                        is_background: false,
                    });
                }

                if self.bird_timer <= 0.0 {
                    self.bird_state = BirdState::Dead;
                    self.bird_timer = 8.0;
                    self.subtitle = "[Subtitles: Bird Vaporized!]".to_string();
                    self.subtitle_timer = 2.0;
                    
                    for _ in 0..20 {
                        self.splashes.push(Splash {
                            x: self.bird_x,
                            y: self.bird_y,
                            vx: self.rng.next_range(-15.0, 15.0),
                            vy: self.rng.next_range(-12.0, -1.0),
                            life: self.rng.next_range(0.5, 1.2),
                            color: (50, 50, 55),
                            is_background: false,
                        });
                        self.splashes.push(Splash {
                            x: self.bird_x,
                            y: self.bird_y,
                            vx: self.rng.next_range(-18.0, 18.0),
                            vy: self.rng.next_range(-14.0, 2.0),
                            life: self.rng.next_range(0.4, 0.9),
                            color: (255, 235, 140),
                            is_background: false,
                        });
                    }
                }
            }
            BirdState::Dead => {
                self.bird_timer -= delta;
                if self.bird_timer <= 0.0 {
                    if !self.perch_points.is_empty() {
                        let p_idx = self.rng.next_usize(self.perch_points.len());
                        self.bird_perch_x = self.perch_points[p_idx].0 as f32;
                        self.bird_perch_y = self.perch_points[p_idx].1 as f32;
                    }
                    self.bird_state = BirdState::Sitting;
                    self.bird_timer = self.rng.next_range(5.0, 15.0);
                    self.bird_x = self.bird_perch_x;
                    self.bird_y = self.bird_perch_y;
                }
            }
        }
    }

    pub fn update_lightning(&mut self, delta: f32, cols: usize, rows: usize) {
        self.lightning_timer += delta;
        if self.lightning_flash > 0.0 {
            self.lightning_flash -= delta;
            if self.lightning_flash <= 0.0 {
                self.lightning_bolts.clear();
            }
        }

        if self.lightning_delay > 0.0 {
            self.lightning_delay -= delta;
            if self.lightning_delay <= 0.0 {
                self.lightning_flash = 0.18;
                if let Some(ref r) = self.rgb {
                    r.flash(RgbColor::WHITE, Duration::from_millis(180));
                }
                self.subtitle = "[CRACK! Lightning Strike]".to_string();
                self.subtitle_timer = 1.5;

                let target_type = self.rng.next_range(0.0, 6.0) as usize;
                let mut target_x = self.rng.next_range(5.0, cols.saturating_sub(5) as f32) as usize;
                let mut target_y = rows - 1;
                let mut hit_bird = false;
                let mut is_bg = false;

                match target_type {
                    0 => {
                        let active: Vec<&LogoCell> = self.logo_cells.iter().filter(|c| c.active).collect();
                        if !active.is_empty() {
                            let selected = active[self.rng.next_usize(active.len())];
                            target_x = selected.x;
                            target_y = selected.y;
                        }
                    }
                    1 => {
                        if !self.fg_scenery.is_empty() {
                            let selected = &self.fg_scenery[self.rng.next_usize(self.fg_scenery.len())];
                            target_x = selected.0;
                            target_y = selected.1;
                        }
                    }
                    2 => {
                        if !self.bg_cells.is_empty() {
                            let selected = &self.bg_cells[self.rng.next_usize(self.bg_cells.len())];
                            target_x = selected.0;
                            target_y = selected.1;
                            is_bg = true;
                        }
                    }
                    3 => {
                        if self.bird_state == BirdState::Sitting || self.bird_state == BirdState::Flying || self.bird_state == BirdState::Scared {
                            target_x = self.bird_x as usize;
                            target_y = self.bird_y as usize;
                            hit_bird = true;
                        }
                    }
                    4 => {
                        target_x = self.rng.next_range(0.0, cols as f32) as usize;
                        target_y = self.rng.next_range(rows as f32 * 0.2, rows as f32 * 0.6) as usize;
                        is_bg = true;
                    }
                    _ => {}
                }
                self.lightning_is_background = is_bg;

                let mut curr_x = self.rng.next_range(5.0, cols.saturating_sub(5) as f32) as usize;
                let mut bolts = Vec::new();
                let mut main_bolt = Vec::new();
                main_bolt.push((curr_x, 0));

                target_x = target_x.clamp(0, cols.saturating_sub(1));
                target_y = target_y.clamp(0, rows.saturating_sub(1));

                for y in 1..=target_y {
                    let diff = target_x as i32 - curr_x as i32;
                    let step = diff.signum();
                    let drift = if diff.abs() <= 1 {
                        self.rng.next_range(-2.0, 2.0) as i32
                    } else {
                        step + self.rng.next_range(-1.5, 1.5) as i32
                    };
                    curr_x = (curr_x as i32 + drift).clamp(0, cols as i32 - 1) as usize;
                    main_bolt.push((curr_x, y));

                    if y < target_y && self.rng.next_bool(0.15) && bolts.len() < 3 {
                        let mut branch = Vec::new();
                        let mut b_x = curr_x;
                        let b_direction = if self.rng.next_bool(0.5) { 1 } else { -1 };
                        for b_y in y..=(y + self.rng.next_range(4.0, 9.0) as usize).min(target_y) {
                            let b_drift = b_direction * (self.rng.next_range(0.0, 2.0) as i32) + self.rng.next_range(-1.0, 1.0) as i32;
                            b_x = (b_x as i32 + b_drift).clamp(0, cols as i32 - 1) as usize;
                            branch.push((b_x, b_y));
                        }
                        bolts.push(branch);
                    }
                }
                bolts.push(main_bolt);
                self.lightning_bolts = bolts;

                let is_lightning_bg = self.lightning_is_background;
                if hit_bird {
                    self.bird_state = BirdState::Explores;
                    self.bird_timer = 2.5;
                    self.bird_vx = self.rng.next_range(-20.0, 20.0);
                    self.bird_vy = self.rng.next_range(-15.0, -5.0);
                    self.subtitle = "[Subtitles: Bird electrified by lightning surge!]".to_string();
                    self.subtitle_timer = 2.0;

                    for _ in 0..10 {
                        self.splashes.push(Splash {
                            x: self.bird_x,
                            y: self.bird_y,
                            vx: self.rng.next_range(-10.0, 10.0),
                            vy: self.rng.next_range(-10.0, 10.0),
                            life: self.rng.next_range(0.4, 0.8),
                            color: (255, 255, 255),
                            is_background: false,
                        });
                    }
                } else {
                    let spark_count = if target_y == rows - 1 { 10 } else { 16 };
                    for _ in 0..spark_count {
                        self.splashes.push(Splash {
                            x: target_x as f32,
                            y: target_y as f32,
                            vx: self.rng.next_range(-14.0, 14.0),
                            vy: self.rng.next_range(-12.0, 1.0),
                            life: self.rng.next_range(0.4, 0.8),
                            color: (255, 235, 140),
                            is_background: is_lightning_bg,
                        });
                    }

                    let mut hit_logo_cell = false;
                    for cell in &mut self.logo_cells {
                        if cell.x == target_x && cell.y == target_y {
                            cell.glow = 1.0;
                            cell.water = (cell.water + 0.8).min(2.5);
                            hit_logo_cell = true;
                        }
                    }
                    if hit_logo_cell {
                        self.subtitle = "[Subtitles: System Surge Detected! *BZZZT*]".to_string();
                        self.subtitle_timer = 2.0;
                    }
                }

                if let Some(ref mut animal) = self.active_animal {
                    if animal.state != AnimalState::Startled && animal.state != AnimalState::WalkingOff {
                        if animal.animal_type == AnimalType::Bigfoot {
                            self.subtitle = "[Subtitles: Bigfoot watches the lightning calmly]".to_string();
                            self.subtitle_timer = 2.2;
                        } else {
                            animal.state = AnimalState::Startled;
                            animal.timer = 0.8;
                            self.subtitle = match animal.animal_type {
                                AnimalType::Deer => "[Subtitles: Deer startled by the blast!]".to_string(),
                                AnimalType::Bear => "[Subtitles: Bear startled! *Growls angrily*]".to_string(),
                                _ => "".to_string(),
                            };
                            self.subtitle_timer = 2.0;
                        }
                    }
                }

                if !hit_bird && (self.bird_state == BirdState::Sitting || self.bird_state == BirdState::Flying) {
                    self.bird_state = BirdState::Scared;
                    self.bird_timer = 0.6;
                }
            }
        }

        if self.lightning_timer > 7.0 && self.rng.next_bool(0.06) && self.lightning_delay <= 0.0 {
            self.lightning_timer = 0.0;
            self.lightning_delay = 0.8;
            self.subtitle = "[Distant thunder rumbling...]".to_string();
            self.subtitle_timer = 1.0;
        }
    }

    pub fn update_scenery_and_animals(&mut self, delta: f32, cols: usize, rows: usize) {
        self.animal_spawn_timer -= delta;
        if self.animal_spawn_timer <= 0.0 && self.active_animal.is_none() {
            self.animal_spawn_timer = self.rng.next_range(25.0, 50.0);
            let roll = self.rng.next_range(0.0, 1.0);
            let animal_type = if roll < 0.50 {
                AnimalType::Deer
            } else if roll < 0.85 {
                AnimalType::Bear
            } else {
                AnimalType::Bigfoot
            };

            let spawn_left = self.rng.next_bool(0.5);
            let base_speed = match animal_type {
                AnimalType::Deer => 4.5f32,
                AnimalType::Bear => 1.8f32,
                AnimalType::Bigfoot => 1.2f32,
            };

            let ay = if animal_type == AnimalType::Bigfoot {
                rows.saturating_sub(4) as f32
            } else {
                rows.saturating_sub(3) as f32
            };

            self.active_animal = Some(Animal {
                x: if spawn_left { -3.0 } else { cols as f32 + 3.0 },
                y: ay,
                vx: if spawn_left { base_speed } else { -base_speed },
                animal_type,
                state: AnimalState::Walking,
                timer: self.rng.next_range(5.0, 9.0),
                frame_toggle: false,
            });

            self.subtitle = match animal_type {
                AnimalType::Deer => "[A deer wanders out of the forest]".to_string(),
                AnimalType::Bear => "[A heavy brown bear walks out of the woods]".to_string(),
                AnimalType::Bigfoot => "[Unidentified creature rustling in the midground trees...]".to_string(),
            };
            self.subtitle_timer = 3.0;
        }

        if let Some(ref mut animal) = self.active_animal {
            animal.timer -= delta;
            
            if self.rng.next_bool(0.08) {
                animal.frame_toggle = !animal.frame_toggle;
            }

            match animal.state {
                AnimalState::Walking => {
                    animal.x += animal.vx * delta;
                    if animal.timer <= 0.0 {
                        if self.rng.next_bool(0.40) && animal.animal_type != AnimalType::Bigfoot {
                            animal.state = AnimalState::Idle;
                            animal.timer = self.rng.next_range(3.0, 6.0);
                            self.subtitle = match animal.animal_type {
                                AnimalType::Deer => "[Deer grazing on mossy ground]".to_string(),
                                AnimalType::Bear => "[Bear sitting down to rest]".to_string(),
                                _ => "".to_string(),
                            };
                            self.subtitle_timer = 2.0;
                        } else {
                            animal.timer = self.rng.next_range(4.0, 8.0);
                        }
                    }
                }
                AnimalState::Idle => {
                    if animal.timer <= 0.0 {
                        animal.state = AnimalState::Walking;
                        animal.timer = self.rng.next_range(4.0, 8.0);
                        self.subtitle = match animal.animal_type {
                            AnimalType::Deer => "[Deer walks on]".to_string(),
                            AnimalType::Bear => "[Bear lumbering forward]".to_string(),
                            _ => "".to_string(),
                        };
                        self.subtitle_timer = 1.8;
                    }
                }
                AnimalState::Startled => {
                    if animal.timer <= 0.0 {
                        animal.state = AnimalState::WalkingOff;
                        let run_speed = match animal.animal_type {
                            AnimalType::Deer => 11.0f32,
                            AnimalType::Bear => 6.0f32,
                            _ => 3.0f32,
                        };
                        animal.vx = animal.vx.signum() * run_speed;
                    }
                }
                AnimalState::WalkingOff => {
                    animal.x += animal.vx * delta;
                }
            }
        }

        if let Some(ref animal) = self.active_animal {
            if animal.x < -6.0 || animal.x > cols as f32 + 6.0 {
                self.active_animal = None;
            }
        }

        if self.subtitle_timer > 0.0 {
            self.subtitle_timer -= delta;
            if self.subtitle_timer <= 0.0 {
                self.subtitle.clear();
            }
        }

        if self.wind.abs() > 8.5 && self.rng.next_bool(0.01) && self.subtitle_timer <= 0.0 {
            self.subtitle = "Warning: Severe gale force wind gusts".to_string();
            self.subtitle_timer = 2.5;
        }
    }


impl Screensaver for Storm {
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

