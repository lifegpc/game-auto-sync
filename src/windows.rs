use winapi::um::wincon::GetConsoleWindow;
use winapi::um::winuser::{ShowWindow, SW_HIDE, SW_SHOW};

fn console_show_window(n_cmd_show: i32) -> bool {
    let h_wnd = unsafe { GetConsoleWindow() };
    if h_wnd.is_null() {
        println!("Failed to get console window.");
        return false;
    }
    unsafe { ShowWindow(h_wnd, n_cmd_show) != 0 }
}

pub fn show_window() -> bool {
    console_show_window(SW_SHOW)
}

pub fn hide_window() -> bool {
    console_show_window(SW_HIDE)
}
