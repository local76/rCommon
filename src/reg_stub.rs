#![allow(dead_code, unused_variables)]
use std::io;

pub type HKEY = isize;
pub const HKEY_CLASSES_ROOT: HKEY = 0;
pub const HKEY_CURRENT_USER: HKEY = 1;
pub const HKEY_LOCAL_MACHINE: HKEY = 2;
pub const HKEY_USERS: HKEY = 3;

pub fn read_string(hive: HKEY, path: &str, key: &str) -> Option<String> {
    None
}

pub fn write_string(hive: HKEY, path: &str, key: &str, val: &str) -> io::Result<()> {
    Ok(())
}

pub fn read_u32(hive: HKEY, path: &str, key: &str) -> Option<u32> {
    None
}

pub fn write_u32(hive: HKEY, path: &str, key: &str, val: u32) -> io::Result<()> {
    Ok(())
}

pub fn read_bool_str(hive: HKEY, path: &str, key: &str) -> bool {
    false
}

pub fn write_bool_str(hive: HKEY, path: &str, key: &str, val: bool) -> io::Result<()> {
    Ok(())
}

pub fn list_values(hive: HKEY, path: &str) -> Option<Vec<(String, String)>> {
    None
}

pub fn read_binary(hive: HKEY, path: &str, key: &str) -> Option<Vec<u8>> {
    None
}

pub fn write_binary(hive: HKEY, path: &str, key: &str, val: &[u8]) -> io::Result<()> {
    Ok(())
}

pub fn delete_value(hive: HKEY, path: &str, key: &str) -> io::Result<()> {
    Ok(())
}
