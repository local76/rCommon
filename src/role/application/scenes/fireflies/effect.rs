use std::time::Duration;
use crate::core::screensaver::Screensaver;
use crate::core::{hsl_to_rgb, rgb_to_hsl, LcgRng, TerminalCell};
use crate::role::application::palette::query_current_palette;
use crate::role::application::rgb::RgbController;
use crate::role::application::rgb::protocol::RgbColor;
use super::types::{Firefly, Attractor, Star, KillSpark};

pub struct Fireflies {
    rng: LcgRng,
    pub(crate) fireflies: Vec<Firefly>,
    pub(crate) attractors: Vec<Attractor>,
    pub(crate) stars: Vec<Star>,
    pub(crate) kill_sparks: Vec<KillSpark>,
    pub(crate) time_elapsed: f32,
    last_cols: usize,
    last_rows: usize,
    pub(crate) logo_excitation: Vec<f32>,
    rgb: Option<RgbController>,
    rgb_timer: f32,
}

impl Fireflies {
    pub fn new() -> Self {
        let rng = LcgRng::new(1357);
        Self {
            rng,
            fireflies: Vec::new(),
            attractors: Vec::new(),
            stars: Vec::new(),
            kill_sparks: Vec::new(),
            time_elapsed: 0.0,
            last_cols: 0,
            last_rows: 0,
            logo_excitation: Vec::new(),
            rgb: Some(RgbController::new()),
            rgb_timer: 0.0,
        }
    }

    fn spawn_new_firefly(&mut self, cols: usize, rows: usize) {
        let size = self.rng.next_range(0.0, 4.0) as u8;
        let speed_mult = self.rng.next_range(0.7, 1.3);

        // library 4.0: pull from the canonical ScreenPalette.
        let accent = query_current_palette().accent;
        let (acc_h, _acc_s, _acc_l) = rgb_to_hsl(accent.0, accent.1, accent.2);
        let p = self.rng.next_f32();
        let h = if p < 0.4 {
            (acc_h + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
        } else if p < 0.7 {
            (acc_h + 120.0 + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
        } else {
            (acc_h - 120.0 + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
        };
        let color = hsl_to_rgb(h, 0.95, 0.60);

        // Spawn on the border of the screen to make it feel like they fly in
        let side = self.rng.next_usize(4);
        let (x, y) = match side {
            0 => (0.0, self.rng.next_range(0.0, rows as f32)), // Left
            1 => (cols as f32 - 1.0, self.rng.next_range(0.0, rows as f32)), // Right
            2 => (self.rng.next_range(0.0, cols as f32), 0.0), // Top
            _ => (self.rng.next_range(0.0, cols as f32), rows as f32 - 1.0), // Bottom
        };

        self.fireflies.push(Firefly {
            x,
            y,
            vx: self.rng.next_range(-3.0, 3.0),
            vy: self.rng.next_range(-3.0, 3.0),
            color,
            size,
            speed_mult,
            history: Vec::new(),
        });
    }
}

impl Screensaver for Fireflies {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let delta = dt.as_secs_f32().min(0.1);
        self.time_elapsed += delta;

        // OpenRGB drift updates
        self.rgb_timer += delta;
        if self.rgb_timer >= 0.15 {
            self.rgb_timer = 0.0;
            if let Some(ref r) = self.rgb {
                if self.fireflies.len() >= 4 {
                    // 5: Keyboard
                    let c0 = self.fireflies[0].color;
                    r.set_device_color(5, RgbColor::new(c0.0, c0.1, c0.2));

                    // 6: Mouse
                    let c1 = self.fireflies[1].color;
                    r.set_device_color(6, RgbColor::new(c1.0, c1.1, c1.2));

                    // 12: Speaker
                    let c2 = self.fireflies[2].color;
                    r.set_device_color(12, RgbColor::new(c2.0, c2.1, c2.2));

                    // 0, 1, 2: Motherboard, RAM, GPU
                    let c3 = self.fireflies[3].color;
                    let m_color = RgbColor::new(c3.0, c3.1, c3.2);
                    r.set_device_color(0, m_color);
                    r.set_device_color(1, m_color);
                    r.set_device_color(2, m_color);
                } else if !self.fireflies.is_empty() {
                    let c0 = self.fireflies[0].color;
                    r.set_color(RgbColor::new(c0.0, c0.1, c0.2));
                }
            }
        }

        // Initialize particles and attractors if grid size changes
        if cols != self.last_cols || rows != self.last_rows {
            self.last_cols = cols;
            self.last_rows = rows;

            // library 4.1: fixed-size logo excitation buffer (pre-4.1
            // `trance_core::logo_dimensions()` was a Windows file read).
            self.logo_excitation = vec![0.0; 80 * 12];

            // library 4.0: pull from the canonical ScreenPalette.
            let accent = query_current_palette().accent;
            let (acc_h, _acc_s, _acc_l) = rgb_to_hsl(accent.0, accent.1, accent.2);

            // Recreate fireflies
            self.fireflies.clear();
            self.kill_sparks.clear();
            let num_fireflies = ((cols * rows) / 45).clamp(30, 60);
            for _ in 0..num_fireflies {
                let size = self.rng.next_range(0.0, 4.0) as u8;
                let speed_mult = self.rng.next_range(0.7, 1.3);

                // Select a harmonious neon color using triadic accent color schemes
                let p = self.rng.next_f32();
                let h = if p < 0.4 {
                    (acc_h + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
                } else if p < 0.7 {
                    (acc_h + 120.0 + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
                } else {
                    (acc_h - 120.0 + self.rng.next_range(-15.0, 15.0)).rem_euclid(360.0)
                };
                let color = hsl_to_rgb(h, 0.95, 0.60);

                self.fireflies.push(Firefly {
                    x: self.rng.next_range(0.0, cols as f32),
                    y: self.rng.next_range(0.0, rows as f32),
                    vx: self.rng.next_range(-5.0, 5.0),
                    vy: self.rng.next_range(-5.0, 5.0),
                    color,
                    size,
                    speed_mult,
                    history: Vec::new(),
                });
            }

            // Recreate stars
            self.stars.clear();
            let target_stars = ((cols * rows) / 25).clamp(30, 120);
            for i in 0..target_stars {
                let ch = if i % 8 == 0 { '✦' } else if i % 3 == 0 { '+' } else { '.' };
                self.stars.push(Star {
                    x: self.rng.next_f32(),
                    y: self.rng.next_f32(),
                    phase: self.rng.next_f32() * std::f32::consts::TAU,
                    ch,
                    excitation: 0.0,
                });
            }

            // Recreate attractors
            self.attractors.clear();
            self.attractors.push(Attractor {
                x: cols as f32 / 2.0,
                y: rows as f32 / 2.0,
                color: accent,
                phase: 0.0,
                speed: 0.6,
            });
            self.attractors.push(Attractor {
                x: cols as f32 / 2.0,
                y: rows as f32 / 2.0,
                color: hsl_to_rgb((acc_h + 120.0).rem_euclid(360.0), 0.95, 0.60),
                phase: 2.0,
                speed: 0.45,
            });
            self.attractors.push(Attractor {
                x: cols as f32 / 2.0,
                y: rows as f32 / 2.0,
                color: hsl_to_rgb((acc_h - 120.0).rem_euclid(360.0), 0.95, 0.60),
                phase: 4.0,
                speed: 0.75,
            });
        }

        let cols_f = cols as f32;
        let rows_f = rows as f32;

        // 1. Move Attractors in smooth Lissajous orbits
        if self.attractors.len() >= 3 {
            let cx = cols_f / 2.0;
            let cy = rows_f / 2.0;

            // Attractor 0
            let t0 = self.time_elapsed * self.attractors[0].speed + self.attractors[0].phase;
            self.attractors[0].x = cx + t0.cos() * (cols_f * 0.35);
            self.attractors[0].y = cy + (t0 * 2.0).sin() * (rows_f * 0.30);

            // Attractor 1
            let t1 = self.time_elapsed * self.attractors[1].speed + self.attractors[1].phase;
            self.attractors[1].x = cx + (t1 * 1.5).sin() * (cols_f * 0.40);
            self.attractors[1].y = cy + t1.cos() * (rows_f * 0.35);

            // Attractor 2
            let t2 = self.time_elapsed * self.attractors[2].speed + self.attractors[2].phase;
            self.attractors[2].x = cx + t2.cos() * (cols_f * 0.28);
            self.attractors[2].y = cy + (t2 * 1.8).cos() * (rows_f * 0.28);
        }

        // 2. Slowly decay logo excitations
        for exc in &mut self.logo_excitation {
            *exc = (*exc - 1.8 * delta).max(0.0);
        }

        // 3. Update fireflies physics & chase/kill/flee steering
        let num_fireflies = self.fireflies.len();
        let mut dead_indices = Vec::new();
        let mut forces = vec![(0.0f32, 0.0f32); num_fireflies];

        for (i, force) in forces.iter_mut().enumerate() {
            let mut fx = 0.0f32;
            let mut fy = 0.0f32;

            // Pull towards orbit attractors
            for attr in &self.attractors {
                let dx = attr.x - self.fireflies[i].x;
                let dy = attr.y - self.fireflies[i].y;
                let dist_sq = dx * dx + dy * dy;
                let dist = dist_sq.sqrt().max(0.1);
                
                let pull = 45.0 / (dist_sq + 20.0);
                fx += (dx / dist) * pull;
                fy += (dy / dist) * pull;
            }

            // Faint pull towards screen center
            let cx = cols_f / 2.0;
            let cy = rows_f / 2.0;
            let dx = cx - self.fireflies[i].x;
            let dy = cy - self.fireflies[i].y;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt().max(0.1);
            let center_pull = 15.0 / (dist_sq + 60.0);
            fx += (dx / dist) * center_pull;
            fy += (dy / dist) * center_pull;

            // Flow wind fields
            let wind_x = (self.time_elapsed * 0.35 + self.fireflies[i].y * 0.08).cos() * 0.35;
            let wind_y = (self.time_elapsed * 0.45 + self.fireflies[i].x * 0.06).sin() * 0.25;
            fx += wind_x;
            fy += wind_y;

            // Faint random jitter
            let rx = self.rng.next_range(-0.5, 0.5);
            let ry = self.rng.next_range(-0.5, 0.5);
            fx += rx;
            fy += ry;

            // Predator-prey logic
            let mut closest_prey_dist = f32::MAX;
            let mut closest_predator_dist = f32::MAX;
            let mut prey_dx = 0.0;
            let mut prey_dy = 0.0;
            let mut pred_dx = 0.0;
            let mut pred_dy = 0.0;
            let mut prey_idx = None;

            for j in 0..num_fireflies {
                if i == j { continue; }
                let dx_j = self.fireflies[j].x - self.fireflies[i].x;
                let dy_j = self.fireflies[j].y - self.fireflies[i].y;
                let dist_sq_j = dx_j * dx_j + dy_j * dy_j;
                let dist_j = dist_sq_j.sqrt().max(0.1);

                if self.fireflies[j].size < self.fireflies[i].size {
                    // Larger fireflies chase smaller fireflies
                    if dist_j < closest_prey_dist {
                        closest_prey_dist = dist_j;
                        prey_dx = dx_j;
                        prey_dy = dy_j;
                        prey_idx = Some(j);
                    }
                } else if self.fireflies[j].size > self.fireflies[i].size {
                    // Smaller fireflies run away from larger fireflies
                    if dist_j < closest_predator_dist {
                        closest_predator_dist = dist_j;
                        pred_dx = dx_j;
                        pred_dy = dy_j;
                    }
                }
            }

            // Apply chase force
            if closest_prey_dist < f32::MAX {
                let force_chase = 55.0 / (closest_prey_dist + 4.5);
                fx += (prey_dx / closest_prey_dist) * force_chase;
                fy += (prey_dy / closest_prey_dist) * force_chase;

                // Mark prey for death if close enough
                if closest_prey_dist < 1.1 {
                    if let Some(idx) = prey_idx {
                        dead_indices.push(idx);
                    }
                }
            }

            // Apply flee force
            if closest_predator_dist < f32::MAX {
                let force_flee = 75.0 / (closest_predator_dist + 2.5);
                fx -= (pred_dx / closest_predator_dist) * force_flee;
                fy -= (pred_dy / closest_predator_dist) * force_flee;
            }

            *force = (fx, fy);
        }

        // Apply forces to velocity and position
        for (p, &(fx, fy)) in self.fireflies.iter_mut().zip(forces.iter()) {

            p.vx += fx * delta * 24.0 * p.speed_mult;
            p.vy += fy * delta * 24.0 * p.speed_mult;
            p.vx *= 1.0 - (delta * 1.8);
            p.vy *= 1.0 - (delta * 1.8);

            let speed = (p.vx * p.vx + p.vy * p.vy).sqrt();
            let max_speed = 36.0;
            if speed > max_speed {
                p.vx = (p.vx / speed) * max_speed;
                p.vy = (p.vy / speed) * max_speed;
            }

            p.x += p.vx * delta;
            p.y += p.vy * delta;

            // Wall bounces
            if p.x < 0.0 {
                p.x = 0.0;
                p.vx = -p.vx * 0.7;
            } else if p.x >= cols_f {
                p.x = cols_f - 1.0;
                p.vx = -p.vx * 0.7;
            }
            if p.y < 0.0 {
                p.y = 0.0;
                p.vy = -p.vy * 0.7;
            } else if p.y >= rows_f {
                p.y = rows_f - 1.0;
                p.vy = -p.vy * 0.7;
            }

            // Save coordinate history trail
            let cell_x = p.x.round() as i32;
            let cell_y = p.y.round() as i32;
            if p.history.is_empty() || p.history.last() != Some(&(cell_x, cell_y)) {
                p.history.push((cell_x, cell_y));
                if p.history.len() > 5 {
                    p.history.remove(0);
                }
            }
        }

        // Process dead fireflies (remove, trigger explosions, and respawn)
        if !dead_indices.is_empty() {
            dead_indices.sort_unstable();
            dead_indices.dedup();

            for &idx in dead_indices.iter().rev() {
                if idx < self.fireflies.len() {
                    let dead = self.fireflies.remove(idx);

                    // Spawn a colorful neon spark explosion burst
                    for _ in 0..12 {
                        let angle = self.rng.next_range(0.0, std::f32::consts::TAU);
                        let speed = self.rng.next_range(8.0, 22.0);
                        self.kill_sparks.push(KillSpark {
                            x: dead.x,
                            y: dead.y,
                            vx: angle.cos() * speed,
                            vy: angle.sin() * speed * 0.5,
                            color: dead.color,
                            life: self.rng.next_range(0.5, 1.2),
                        });
                    }

                    // Respawn a new firefly on the border to replace the population
                    self.spawn_new_firefly(cols, rows);
                }
            }
        }

        // Update kill sparks physics
        for spark in &mut self.kill_sparks {
            spark.x += spark.vx * delta;
            spark.y += spark.vy * delta;
            spark.life -= delta * 2.0;
        }
        self.kill_sparks.retain(|s| s.life > 0.0);

        // 3b. Decay star excitations and update them from fireflies
        for star in &mut self.stars {
            star.excitation = (star.excitation - 1.2 * delta).max(0.0);
        }
        for p in &self.fireflies {
            for star in &mut self.stars {
                let dx = p.x - star.x * cols_f;
                let dy = (p.y - star.y * rows_f) * 2.0;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < 9.0 {
                    let dist = dist_sq.sqrt();
                    let force = (1.0 - dist / 3.0).max(0.0) * 1.5;
                    star.excitation = star.excitation.max(force);
                }
            }
        }

        // 4. Update logo excitations from nearby fireflies
        // library 4.1: fixed 80x12 logo size (pre-4.1
        // `trance_core::logo_dimensions()` was a Windows file read).
        let logo_w: usize = 80;
        let logo_h: usize = 12;
        if logo_w > 0 && logo_h > 0 && self.logo_excitation.len() == logo_w * logo_h {
            let logo_x = cols.saturating_sub(logo_w) / 2;
            let logo_y = rows.saturating_sub(logo_h) / 2;

            for p in &self.fireflies {
                let px = p.x.round() as i32;
                let py = p.y.round() as i32;
                if px >= logo_x as i32 && px < (logo_x + logo_w) as i32 &&
                   py >= logo_y as i32 && py < (logo_y + logo_h) as i32 {
                    let lx = px as usize - logo_x;
                    let ly = py as usize - logo_y;
                    let l_idx = ly * logo_w + lx;
                    if l_idx < self.logo_excitation.len() {
                        self.logo_excitation[l_idx] = 1.0;
                    }
                }
            }
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        self.draw_impl(grid, cols, rows);
    }
}
