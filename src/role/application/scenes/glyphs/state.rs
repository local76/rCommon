use std::time::Duration;
use crate::core::screensaver::Screensaver;
use crate::core::{LcgRng, TerminalCell};
use crate::platform::native::sys_info::get_system_info;
use crate::role::application::rgb::{RgbController, is_openrgb_enabled};
use crate::role::application::rgb::protocol::RgbColor;

use super::types::RainDrop;

pub struct Glyphs {
    rng: LcgRng,
    pub(super) drops: Vec<RainDrop>,
    pub(super) char_pool: Vec<char>,
    last_cols: usize,
    last_rows: usize,
    density_opt: u32,

    // Live system dynamics + data rain
    sys_refresh_timer: f32,
    mem_pressure: f32,
    pub(super) live_system_chars: Vec<char>,
    rgb: Option<RgbController>,
    rgb_timer: f32,
    time_elapsed: f32,
}

impl Default for Glyphs {
    fn default() -> Self {
        Self::new()
    }
}

impl Glyphs {
    pub fn new() -> Self {
        let char_pool_katakana = "ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ1234567890X:+-*<>|";
        let mut char_pool: Vec<char> = char_pool_katakana.chars().collect();
        let mut rng = LcgRng::new(0xCAFEBABEu64.wrapping_add(0xDEADBEEFu64));
        let sys = get_system_info();
        let live_system_chars: Vec<char> = sys
            .hostname
            .chars()
            .chain(sys.os.chars())
            .chain(sys.kernel.chars())
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();

        // Sparse starting set
        for _ in 0..256 {
            let idx = rng.next_usize(char_pool.len().max(1));
            if idx < char_pool.len() {
                char_pool.push(char_pool[idx]);
            }
        }

        Self {
            rng,
            drops: Vec::new(),
            char_pool,
            last_cols: 0,
            last_rows: 0,
            density_opt: 1,

            sys_refresh_timer: 0.0,
            mem_pressure: sys.mem_used_pct / 100.0,
            live_system_chars,
            rgb: if is_openrgb_enabled() { Some(RgbController::new()) } else { None },
            rgb_timer: 0.0,
            time_elapsed: 0.0,
        }
    }
}

impl Screensaver for Glyphs {
    fn update(&mut self, dt: Duration, cols: usize, rows: usize) {
        let delta = dt.as_secs_f32();
        self.time_elapsed += delta;

        // OpenRGB scrolling digital rain updates
        self.rgb_timer += delta;
        if self.rgb_timer >= 0.05 {
            self.rgb_timer = 0.0;
            if let Some(ref r) = self.rgb {
                let get_matrix_color = |x: f32, speed: f32| -> RgbColor {
                    let val = ((self.time_elapsed * speed - x * std::f32::consts::TAU).sin() * 127.0 + 128.0).clamp(0.0, 255.0) as u8;
                    let g = (val as f32 * (0.3 + self.mem_pressure * 0.7)) as u8;
                    RgbColor::new(0, g, 0)
                };

                r.set_device_color(5, get_matrix_color(0.5, 4.0));
                r.set_device_color(6, get_matrix_color(0.8, 4.0));
                r.set_device_color(12, get_matrix_color(0.1, 4.0));
                let c_internal = get_matrix_color(0.6, 4.0);
                r.set_device_color(0, c_internal);
                r.set_device_color(1, c_internal);
                r.set_device_color(2, c_internal);
            }
        }

        // System stats refresh every 2s
        self.sys_refresh_timer += delta;
        if self.sys_refresh_timer >= 2.0 {
            self.sys_refresh_timer = 0.0;
            let sys = get_system_info();
            self.mem_pressure = sys.mem_used_pct / 100.0;
        }

        if cols != self.last_cols || rows != self.last_rows || self.drops.is_empty() {
            self.last_cols = cols;
            self.last_rows = rows;
            self.drops.clear();
            let density = self.density_opt.max(1) as usize;
            for x in 0..cols {
                if self.rng.next_usize(density.max(1)) == 0 {
                    self.drops.push(RainDrop {
                        x: x as f32,
                        y: -self.rng.next_f32() * (rows as f32),
                        speed: self.rng.next_range(8.0, 30.0),
                        length: 4 + self.rng.next_usize(16),
                        char_rot: self.rng.next_usize(self.char_pool.len().max(1)),
                    });
                }
            }
        }

        for d in self.drops.iter_mut() {
            d.y += d.speed * delta;
            d.char_rot = (d.char_rot + 1) % self.char_pool.len().max(1);
            if d.y as i32 > rows as i32 + d.length as i32 {
                d.y = -(d.length as f32) - self.rng.next_f32() * 4.0;
                d.speed = self.rng.next_range(8.0, 30.0);
                d.length = 4 + self.rng.next_usize(16);
            }
        }
    }

    fn draw(&self, grid: &mut [TerminalCell], cols: usize, rows: usize) {
        self.draw_impl(grid, cols, rows);
    }
}
