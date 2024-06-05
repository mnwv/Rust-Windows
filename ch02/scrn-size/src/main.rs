#![windows_subsystem = "windows"]

use windows_sys::Win32::UI::WindowsAndMessaging::*;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

macro_rules! to_wide_chars {
    ($str:expr) => {
        OsStr::new($str).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
    }
}

macro_rules! msgbox_with_format {
    ($caption:expr, $fmt:expr) => {
        let caption = to_wide_chars!($caption);
        let msg = to_wide_chars!($fmt);
        unsafe {
            MessageBoxW(0, msg.as_ptr(), caption.as_ptr(), MB_OK)
        }
    };
    ($caption:expr, $fmt:expr, $($arg:tt)*) => {
        let caption = to_wide_chars!($caption);
        let formatted_msg = std::fmt::format(format_args!($fmt, $($arg)*));
        let msg = to_wide_chars!(&formatted_msg);
        unsafe {
            MessageBoxW(0, msg.as_ptr(), caption.as_ptr(), MB_OK)
        }
   };
}

fn main() {
    msgbox_with_format!("ScrnSize", "スクリーン解像度を表示します。");
    let cx_screen = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let cy_screen = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    msgbox_with_format!("ScrnSize", "The screen is {} pixcels wide by {} pixcels high.",
                        cx_screen, cy_screen);
    msgbox_with_format!("スクリーン解像度", "幅: {} ピクセル、高さ: {} ピクセル",
                        cx_screen, cy_screen);
}
