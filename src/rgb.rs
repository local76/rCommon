pub mod protocol;
pub mod controller;

pub use protocol::{RgbColor, OpenRGBDevice, parse_device_payload};
pub use controller::{RgbCommand, RgbController};
