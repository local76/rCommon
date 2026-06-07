use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use rcommon::rgb::parse_device_payload;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("===================================================");
    println!("   OpenRGB Connection & Hardware Check");
    println!("===================================================\n");

    let addr = SocketAddr::from(([127, 0, 0, 1], 6742));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(800))?;
    stream.set_read_timeout(Some(Duration::from_millis(800)))?;
    stream.set_write_timeout(Some(Duration::from_millis(800)))?;

    println!("[1/4] Negotiating protocol version...");
    let mut header = [0u8; 16];
    header[0..4].copy_from_slice(b"ORGB");
    header[4..8].copy_from_slice(&0u32.to_le_bytes());
    header[8..12].copy_from_slice(&40u32.to_le_bytes());
    header[12..16].copy_from_slice(&4u32.to_le_bytes());
    stream.write_all(&header)?;
    stream.write_all(&1u32.to_le_bytes())?;

    let mut resp_header = [0u8; 16];
    stream.read_exact(&mut resp_header)?;
    let resp_size = u32::from_le_bytes(resp_header[12..16].try_into().unwrap());
    if resp_size == 4 {
        let mut resp_payload = [0u8; 4];
        stream.read_exact(&mut resp_payload)?;
        println!("      Negotiated Version: {}", u32::from_le_bytes(resp_payload));
    } else if resp_size > 0 {
        let mut temp = vec![0u8; resp_size as usize];
        stream.read_exact(&mut temp)?;
    }

    println!("[2/4] Registering client name...");
    let name = "rIdleCheck\0";
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len() as u16;
    let mut payload = Vec::new();
    payload.extend_from_slice(&name_len.to_le_bytes());
    payload.extend_from_slice(name_bytes);

    let mut header = [0u8; 16];
    header[0..4].copy_from_slice(b"ORGB");
    header[4..8].copy_from_slice(&0u32.to_le_bytes());
    header[8..12].copy_from_slice(&50u32.to_le_bytes());
    header[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    stream.write_all(&header)?;
    stream.write_all(&payload)?;

    println!("[3/4] Requesting device count...");
    let mut header = [0u8; 16];
    header[0..4].copy_from_slice(b"ORGB");
    header[4..8].copy_from_slice(&0u32.to_le_bytes());
    header[8..12].copy_from_slice(&0u32.to_le_bytes());
    header[12..16].copy_from_slice(&0u32.to_le_bytes());
    stream.write_all(&header)?;

    let mut resp_header = [0u8; 16];
    stream.read_exact(&mut resp_header)?;
    let resp_size = u32::from_le_bytes(resp_header[12..16].try_into().unwrap());
    let count = if resp_size == 4 {
        let mut resp_payload = [0u8; 4];
        stream.read_exact(&mut resp_payload)?;
        u32::from_le_bytes(resp_payload)
    } else {
        0
    };
    println!("      Found {} RGB device(s) connected to OpenRGB.", count);

    let mut devices = Vec::new();
    for idx in 0..count {
        let mut header = [0u8; 16];
        header[0..4].copy_from_slice(b"ORGB");
        header[4..8].copy_from_slice(&idx.to_le_bytes());
        header[8..12].copy_from_slice(&1u32.to_le_bytes());
        header[12..16].copy_from_slice(&4u32.to_le_bytes());
        stream.write_all(&header)?;
        stream.write_all(&1u32.to_le_bytes())?;

        let mut resp_header = [0u8; 16];
        stream.read_exact(&mut resp_header)?;
        let resp_size = u32::from_le_bytes(resp_header[12..16].try_into().unwrap());
        let mut dev_payload = vec![0u8; resp_size as usize];
        stream.read_exact(&mut dev_payload)?;

        match parse_device_payload(idx, &dev_payload) {
            Ok(device) => {
                let type_str = match device.device_type {
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
                };
                println!("        -> [{}] {} (Type: {}, LEDs: {})", device.index, device.name, type_str, device.num_leds);
                devices.push(device);
            }
            Err(e) => {
                println!("        -> Error parsing device [{}]: {}", idx, e);
            }
        }
    }

    if count > 0 {
        println!("\n[4/4] Flashing all devices white for 1 second...");
        for dev in &devices {
            println!("        Flashing device {} (index {})...", dev.name, dev.index);
            let num_colors = dev.num_leds;
            let mut payload = Vec::new();
            payload.extend_from_slice(&(4 + 2 + num_colors as u32 * 4).to_le_bytes());
            payload.extend_from_slice(&num_colors.to_le_bytes());
            for _ in 0..num_colors {
                payload.push(255); // R
                payload.push(255); // G
                payload.push(255); // B
                payload.push(0);   // padding
            }

            let mut header = [0u8; 16];
            header[0..4].copy_from_slice(b"ORGB");
            header[4..8].copy_from_slice(&dev.index.to_le_bytes());
            header[8..12].copy_from_slice(&1050u32.to_le_bytes());
            header[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());

            stream.write_all(&header)?;
            stream.write_all(&payload)?;
        }

        println!("      Sleeping for 1 second...");
        std::thread::sleep(Duration::from_secs(1));

        println!("      Restoring initial colors...");
        for dev in &devices {
            println!("        Restoring device {} (index {})...", dev.name, dev.index);
            let num_colors = dev.num_leds;
            let mut payload = Vec::new();
            payload.extend_from_slice(&(4 + 2 + num_colors as u32 * 4).to_le_bytes());
            payload.extend_from_slice(&num_colors.to_le_bytes());
            for color in &dev.initial_colors {
                payload.push(color.r);
                payload.push(color.g);
                payload.push(color.b);
                payload.push(0);
            }

            let mut header = [0u8; 16];
            header[0..4].copy_from_slice(b"ORGB");
            header[4..8].copy_from_slice(&dev.index.to_le_bytes());
            header[8..12].copy_from_slice(&1050u32.to_le_bytes());
            header[12..16].copy_from_slice(&(payload.len() as u32).to_le_bytes());

            stream.write_all(&header)?;
            stream.write_all(&payload)?;
        }
    }

    println!("\nVerification completed successfully!");
    Ok(())
}


