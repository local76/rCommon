use library::toolkit::sys_info::*;

#[test]
fn test_local_ip() {
    let ip = query_local_ip();
    println!("Local IP: {:?}", ip);
}

#[test]
fn test_sys_info_stubs() {
    let res = get_system_screen_resolution();
    assert!(res.0 > 0 && res.1 > 0);
    let dpi = get_console_window_dpi();
    assert!(dpi > 0);
}

#[test]
fn test_dashboard_info() {
    let info = get_dashboard_info();
    assert!(!info.os.is_empty());
}
