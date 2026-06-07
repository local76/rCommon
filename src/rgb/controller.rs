use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use std::io::Write;

use crate::rgb::protocol::{RgbColor, OpenRGBClient, find_restore_profile};

pub enum RgbCommand {
    SetColor(RgbColor),
    SetDeviceColor(u32, RgbColor), // u32 target device type
    Flash {
        color: RgbColor,
        duration: Duration,
    },
}

pub struct RgbController {
    tx: Option<Sender<RgbCommand>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Default for RgbController {
    fn default() -> Self {
        Self::new()
    }
}

impl RgbController {
    /// Starts the background OpenRGB thread and returns a controller handle.
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let thread_handle = thread::spawn(move || {
            run_rgb_thread(rx);
        });
        Self {
            tx: Some(tx),
            thread_handle: Some(thread_handle),
        }
    }

    /// Set all connected RGB devices to the specified color.
    pub fn set_color(&self, color: RgbColor) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(RgbCommand::SetColor(color));
        }
    }

    /// Set all connected RGB devices of a specific type to the specified color.
    pub fn set_device_color(&self, device_type: u32, color: RgbColor) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(RgbCommand::SetDeviceColor(device_type, color));
        }
    }

    /// Trigger a temporary flash of a color (e.g. for lightning strikes),
    /// which will smoothly fade back to the ambient/previous color.
    pub fn flash(&self, color: RgbColor, duration: Duration) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(RgbCommand::Flash { color, duration });
        }
    }
}

impl Drop for RgbController {
    fn drop(&mut self) {
        self.tx = None;
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

struct ActiveFlash {
    start_time: Instant,
    end_time: Instant,
    start_color: RgbColor,
    target_color: RgbColor,
}

fn lerp(start: u8, end: u8, t: f32) -> u8 {
    let s = start as f32;
    let e = end as f32;
    (s + (e - s) * t).clamp(0.0, 255.0) as u8
}

fn write_all_devices(client: &mut OpenRGBClient, color: RgbColor) -> Result<(), std::io::Error> {
    let targets: Vec<(u32, u16)> = client.devices.iter().map(|d| (d.index, d.num_leds)).collect();
    for (index, num_leds) in targets {
        let colors = vec![color; num_leds as usize];
        client.update_leds(index, &colors)?;
    }
    Ok(())
}

fn write_device_type(client: &mut OpenRGBClient, device_type: u32, color: RgbColor) -> Result<(), std::io::Error> {
    let targets: Vec<(u32, u16)> = client.devices.iter()
        .filter(|d| d.device_type == device_type)
        .map(|d| (d.index, d.num_leds))
        .collect();
    for (index, num_leds) in targets {
        let colors = vec![color; num_leds as usize];
        client.update_leds(index, &colors)?;
    }
    Ok(())
}

fn run_rgb_thread(rx: Receiver<RgbCommand>) {
    let mut client: Option<OpenRGBClient> = None;
    let mut last_connect_attempt = Instant::now() - Duration::from_secs(10);
    let mut current_color = RgbColor::BLACK;
    let mut active_flash: Option<ActiveFlash> = None;

    loop {
        if client.is_none() && last_connect_attempt.elapsed() > Duration::from_secs(5) {
            last_connect_attempt = Instant::now();
            if let Ok(c) = OpenRGBClient::connect() {
                client = Some(c);
                // Write current active color to devices immediately upon connection
                if active_flash.is_none() {
                    let _ = write_all_devices(client.as_mut().unwrap(), current_color);
                }
            }
        }

        let timeout = if active_flash.is_some() {
            Duration::from_millis(16) // Smooth lerping at ~60fps
        } else if client.is_none() {
            Duration::from_secs(2)
        } else {
            Duration::from_millis(100)
        };

        match rx.recv_timeout(timeout) {
            Ok(RgbCommand::SetColor(color)) => {
                current_color = color;
                if active_flash.is_none() {
                    if let Some(c) = &mut client {
                        if write_all_devices(c, color).is_err() {
                            client = None;
                        }
                    }
                }
            }
            Ok(RgbCommand::SetDeviceColor(device_type, color)) => {
                if active_flash.is_none() {
                    if let Some(c) = &mut client {
                        if write_device_type(c, device_type, color).is_err() {
                            client = None;
                        }
                    }
                }
            }
            Ok(RgbCommand::Flash { color, duration }) => {
                let now = Instant::now();
                active_flash = Some(ActiveFlash {
                    start_time: now,
                    end_time: now + duration,
                    start_color: current_color,
                    target_color: color,
                });
                if let Some(c) = &mut client {
                    if write_all_devices(c, color).is_err() {
                        client = None;
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some(flash) = &active_flash {
                    let now = Instant::now();
                    if now >= flash.end_time {
                        let orig = flash.start_color;
                        active_flash = None;
                        if let Some(c) = &mut client {
                            if write_all_devices(c, orig).is_err() {
                                client = None;
                            }
                        }
                    } else {
                        let total_dur = flash.end_time.duration_since(flash.start_time).as_secs_f32();
                        let elapsed = now.duration_since(flash.start_time).as_secs_f32();
                        let t = (elapsed / total_dur).clamp(0.0, 1.0);
                        let r = lerp(flash.target_color.r, flash.start_color.r, t);
                        let g = lerp(flash.target_color.g, flash.start_color.g, t);
                        let b = lerp(flash.target_color.b, flash.start_color.b, t);
                        let lerped = RgbColor { r, g, b };
                        if let Some(c) = &mut client {
                            if write_all_devices(c, lerped).is_err() {
                                client = None;
                            }
                        }
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                if let Some(mut c) = client {
                    // Always restore the initial colors of all devices first
                    let targets: Vec<(u32, Vec<RgbColor>)> = c.devices.iter()
                        .map(|d| (d.index, d.initial_colors.clone()))
                        .collect();
                    for (index, colors) in targets {
                        let _ = c.update_leds(index, &colors);
                    }

                    // Optionally also trigger the profile load (if one is configured/present)
                    if let Some(profile) = find_restore_profile() {
                        let name_bytes = profile.as_bytes();
                        let name_len = name_bytes.len() as u16;
                        let mut payload = Vec::new();
                        payload.extend_from_slice(&name_len.to_le_bytes());
                        payload.extend_from_slice(name_bytes);

                        let mut header = [0u8; 16];
                        header[0..4].copy_from_slice(b"ORGB");
                        header[4..8].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // Global device index
                        header[8..12].copy_from_slice(&150u32.to_le_bytes()); // LOAD_PROFILE ID
                        header[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());

                        let _ = c.stream.write_all(&header);
                        let _ = c.stream.write_all(&payload);
                    }

                    // Flush and sleep to allow OpenRGB to process the final packet
                    let _ = c.stream.flush();
                    std::thread::sleep(std::time::Duration::from_millis(150));
                }
                break;
            }
        }
    }
}
