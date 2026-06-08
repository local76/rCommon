//! OpenRGB SDK client protocol parsing and connection helpers.
//!
//! **Taxonomy Classification**: System Role (Purpose - Application Software).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub const OPENRGB_MAGIC: &[u8; 4] = b"ORGB";
pub const OPENRGB_DEFAULT_PORT: u16 = 6742;
pub const CMD_GET_CONTROLLER_COUNT: u32 = 0;
pub const CMD_GET_CONTROLLER_DATA: u32 = 1;
pub const CMD_GET_PROTOCOL_VERSION: u32 = 40;
pub const CMD_SET_CLIENT_NAME: u32 = 50;
pub const CMD_LOAD_PROFILE: u32 = 150;
pub const CMD_SET_LAYER_COLORS: u32 = 1050;
pub const HEADER_SIZE: usize = 16;
pub const MAX_PAYLOAD_SIZE: u32 = 10 * 1024 * 1024;
pub const MODE_FIXED_FIELDS_SIZE: usize = 36;
pub const ZONE_FIXED_FIELDS_SIZE: usize = 16;

/// Returns the human-readable name of an OpenRGB device type ID.
pub fn device_type_name(t: u32) -> &'static str {
    match t {
        0 => "Motherboard",
        1 => "DRAM (RAM)",
        2 => "GPU",
        3 => "Cooler",
        4 => "LED Strip",
        5 => "Keyboard",
        6 => "Mouse",
        7 => "Mousemat",
        8 => "Headset",
        12 => "Speaker",
        _ => "Other/Unknown",
    }
}

/// Configuration options for connecting to an OpenRGB server.
#[derive(Debug, Clone)]
pub struct OpenRGBConfig {
    pub server_host: String,
    pub server_port: u16,
    pub client_name: String,
    pub connection_timeout: Duration,
}

impl Default for OpenRGBConfig {
    fn default() -> Self {
        let exe_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
            .unwrap_or_else(|| "rApp".to_string());
        let exe_clean = exe_name.strip_suffix(".exe").unwrap_or(&exe_name).to_string();

        Self {
            server_host: "127.0.0.1".to_string(),
            server_port: OPENRGB_DEFAULT_PORT,
            client_name: exe_clean,
            connection_timeout: Duration::from_millis(500),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug, Clone)]
pub struct OpenRGBDevice {
    pub index: u32,
    pub device_type: u32,
    pub name: String,
    pub num_leds: u16,
    pub initial_colors: Vec<RgbColor>,
}

pub struct OpenRGBClient {
    pub stream: TcpStream,
    pub devices: Vec<OpenRGBDevice>,
}

impl OpenRGBClient {
    #[allow(dead_code)]
    pub fn connect() -> Result<Self, std::io::Error> {
        Self::connect_with_config(&OpenRGBConfig::default())
    }

    pub fn connect_with_config(config: &OpenRGBConfig) -> Result<Self, std::io::Error> {
        let addr = format!("{}:{}", config.server_host, config.server_port);
        let addrs: Vec<SocketAddr> = addr.to_socket_addrs()?.collect();
        if addrs.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Failed to resolve address"));
        }
        let mut stream = TcpStream::connect_timeout(&addrs[0], config.connection_timeout)?;
        stream.set_read_timeout(Some(config.connection_timeout))?;
        stream.set_write_timeout(Some(config.connection_timeout))?;

        // 1. Request Protocol Version (Command ID 40)
        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(OPENRGB_MAGIC);
        header[4..8].copy_from_slice(&0u32.to_le_bytes());
        header[8..12].copy_from_slice(&CMD_GET_PROTOCOL_VERSION.to_le_bytes());
        header[12..16].copy_from_slice(&4u32.to_le_bytes());
        stream.write_all(&header)?;
        stream.write_all(&1u32.to_le_bytes())?; // Negotiating protocol version 1

        let mut resp_header = [0u8; HEADER_SIZE];
        stream.read_exact(&mut resp_header)?;
        if &resp_header[0..4] != OPENRGB_MAGIC {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid magic"));
        }
        let resp_size = u32::from_le_bytes(resp_header[12..16].try_into().unwrap());
        if resp_size > MAX_PAYLOAD_SIZE {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Response payload too large"));
        }
        if resp_size == 4 {
            let mut resp_payload = [0u8; 4];
            stream.read_exact(&mut resp_payload)?;
        } else if resp_size > 0 {
            let mut temp = vec![0u8; resp_size as usize];
            stream.read_exact(&mut temp)?;
        }

        // 2. Set Client Name (Command ID 50)
        let mut name = config.client_name.clone();
        if !name.ends_with('\0') {
            name.push('\0');
        }
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len() as u16;
        let mut payload = Vec::new();
        payload.extend_from_slice(&name_len.to_le_bytes());
        payload.extend_from_slice(name_bytes);

        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(OPENRGB_MAGIC);
        header[4..8].copy_from_slice(&0u32.to_le_bytes());
        header[8..12].copy_from_slice(&CMD_SET_CLIENT_NAME.to_le_bytes());
        header[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        stream.write_all(&header)?;
        stream.write_all(&payload)?;

        // 3. Request Controller Count (Command ID 0)
        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(OPENRGB_MAGIC);
        header[4..8].copy_from_slice(&0u32.to_le_bytes());
        header[8..12].copy_from_slice(&CMD_GET_CONTROLLER_COUNT.to_le_bytes());
        header[12..16].copy_from_slice(&0u32.to_le_bytes());
        stream.write_all(&header)?;

        let mut resp_header = [0u8; HEADER_SIZE];
        stream.read_exact(&mut resp_header)?;
        let resp_size = u32::from_le_bytes(resp_header[12..16].try_into().unwrap());
        let count = if resp_size == 4 {
            let mut resp_payload = [0u8; 4];
            stream.read_exact(&mut resp_payload)?;
            u32::from_le_bytes(resp_payload)
        } else {
            0
        };

        // 4. Request Controller Data for each index (Command ID 1)
        let mut devices = Vec::new();
        for idx in 0..count {
            let mut header = [0u8; HEADER_SIZE];
            header[0..4].copy_from_slice(OPENRGB_MAGIC);
            header[4..8].copy_from_slice(&idx.to_le_bytes());
            header[8..12].copy_from_slice(&CMD_GET_CONTROLLER_DATA.to_le_bytes());
            header[12..16].copy_from_slice(&4u32.to_le_bytes());
            stream.write_all(&header)?;
            stream.write_all(&1u32.to_le_bytes())?;

            let mut resp_header = [0u8; HEADER_SIZE];
            stream.read_exact(&mut resp_header)?;
            let resp_size = u32::from_le_bytes(resp_header[12..16].try_into().unwrap());
            if resp_size > MAX_PAYLOAD_SIZE {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Device payload too large"));
            }
            let mut dev_payload = vec![0u8; resp_size as usize];
            stream.read_exact(&mut dev_payload)?;

            if let Ok(device) = parse_device_payload(idx, &dev_payload) {
                devices.push(device);
            }
        }

        Ok(Self { stream, devices })
    }

    pub fn update_leds(&mut self, device_index: u32, colors: &[RgbColor]) -> Result<(), std::io::Error> {
        let num_colors = colors.len() as u16;
        let mut payload = Vec::new();
        payload.extend_from_slice(&(4 + 2 + num_colors as u32 * 4).to_le_bytes());
        payload.extend_from_slice(&num_colors.to_le_bytes());
        for c in colors {
            payload.push(c.r);
            payload.push(c.g);
            payload.push(c.b);
            payload.push(0);
        }

        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(OPENRGB_MAGIC);
        header[4..8].copy_from_slice(&device_index.to_le_bytes());
        header[8..12].copy_from_slice(&CMD_SET_LAYER_COLORS.to_le_bytes());
        header[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());

        self.stream.write_all(&header)?;
        self.stream.write_all(&payload)?;
        Ok(())
    }
}

pub fn parse_device_payload(index: u32, data: &[u8]) -> crate::error::Result<OpenRGBDevice> {
    let mut cursor = 0;

    let read_u16 = |cur: &mut usize| -> crate::error::Result<u16> {
        if *cur + 2 > data.len() { return Err(crate::error::RcommonError::Rgb("EOF u16".to_string())); }
        let val = u16::from_le_bytes(data[*cur..*cur+2].try_into().unwrap());
        *cur += 2;
        Ok(val)
    };

    let read_u32 = |cur: &mut usize| -> crate::error::Result<u32> {
        if *cur + 4 > data.len() { return Err(crate::error::RcommonError::Rgb("EOF u32".to_string())); }
        let val = u32::from_le_bytes(data[*cur..*cur+4].try_into().unwrap());
        *cur += 4;
        Ok(val)
    };

    let read_string = |cur: &mut usize| -> crate::error::Result<String> {
        let len = read_u16(cur)? as usize;
        if len == 0 { return Ok(String::new()); }
        if *cur + len > data.len() { return Err(crate::error::RcommonError::Rgb("EOF String".to_string())); }
        let s_bytes = &data[*cur..*cur + len];
        *cur += len;
        let clean_len = if len > 0 && s_bytes[len - 1] == 0 { len - 1 } else { len };
        let s = String::from_utf8_lossy(&s_bytes[..clean_len]).into_owned();
        Ok(s)
    };

    let skip_bytes = |cur: &mut usize, n: usize| -> crate::error::Result<()> {
        if *cur + n > data.len() { return Err(crate::error::RcommonError::Rgb("EOF skip_bytes".to_string())); }
        *cur += n;
        Ok(())
    };

    let skip_string = |cur: &mut usize| -> crate::error::Result<()> {
        let len = read_u16(cur)? as usize;
        if *cur + len > data.len() { return Err(crate::error::RcommonError::Rgb("EOF skip_string".to_string())); }
        *cur += len;
        Ok(())
    };

    let _data_size = read_u32(&mut cursor)?;
    let device_type = read_u32(&mut cursor)?;
    let name = read_string(&mut cursor)?;

    // Skip vendor, description, version, serial, location
    skip_string(&mut cursor)?;
    skip_string(&mut cursor)?;
    skip_string(&mut cursor)?;
    skip_string(&mut cursor)?;
    skip_string(&mut cursor)?;

    let num_modes = read_u16(&mut cursor)?;
    skip_bytes(&mut cursor, 4)?; // _active_mode (i32)

    for _ in 0..num_modes {
        skip_string(&mut cursor)?; // _m_name
        skip_bytes(&mut cursor, MODE_FIXED_FIELDS_SIZE)?; // _m_value to _m_color_mode (9 fields * 4 bytes = 36 bytes)
        let colors_len = read_u16(&mut cursor)? as usize;
        skip_bytes(&mut cursor, colors_len * 4)?; // mode colors
    }

    let num_zones = read_u16(&mut cursor)?;
    for _ in 0..num_zones {
        skip_string(&mut cursor)?; // _z_name
        skip_bytes(&mut cursor, ZONE_FIXED_FIELDS_SIZE)?; // _z_type, _z_leds_min, _z_leds_max, _z_leds_count (4 fields * 4 bytes = 16 bytes)
        let matrix_len = read_u16(&mut cursor)? as usize;
        skip_bytes(&mut cursor, matrix_len)?; // zone matrix
    }

    let num_leds = read_u16(&mut cursor)?;
    for _ in 0..num_leds {
        skip_string(&mut cursor)?; // _l_name
        skip_bytes(&mut cursor, 4)?; // _l_value (u32)
    }

    let num_colors = read_u16(&mut cursor)?;
    let mut initial_colors = Vec::new();
    for _ in 0..num_colors {
        if cursor + 4 > data.len() { return Err(crate::error::RcommonError::Rgb("EOF Colors".to_string())); }
        let r = data[cursor];
        let g = data[cursor + 1];
        let b = data[cursor + 2];
        cursor += 4;
        initial_colors.push(RgbColor { r, g, b });
    }

    Ok(OpenRGBDevice {
        index,
        device_type,
        name,
        num_leds: num_colors,
        initial_colors,
    })
}

pub(crate) fn get_openrgb_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").ok()?;
        Some(std::path::PathBuf::from(appdata).join("OpenRGB"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            Some(std::path::PathBuf::from(xdg).join("OpenRGB"))
        } else {
            let home = std::env::var("HOME").ok()?;
            Some(std::path::PathBuf::from(home).join(".config").join("OpenRGB"))
        }
    }
}

pub(crate) fn find_restore_profile() -> Option<String> {
    #[cfg(all(feature = "reg", target_os = "windows"))]
    {
        if let Some(profile) = crate::reg::read_string(
            crate::reg::HKEY_CURRENT_USER,
            r#"Software\Windows-Screensavers\Settings"#,
            "OpenRGBRestoreProfile",
        ) {
            if !profile.is_empty() {
                return Some(profile);
            }
        }
    }

    // 2. Scan OpenRGB folder for profiles
    let dir = get_openrgb_dir()?;
    if !dir.exists() { return None; }

    // Check standard profiles
    for preferred in &["default", "restore", "normal"] {
        let p_path = dir.join(format!("{}.orp", preferred));
        if p_path.exists() {
            return Some(preferred.to_string());
        }
    }

    // Fall back to the first profile found
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "orp") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    return Some(stem.to_string());
                }
            }
        }
    }

    None
}
