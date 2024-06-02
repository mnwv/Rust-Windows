use windows_sys::{
    core::*, 
    Win32::UI::WindowsAndMessaging::*,
};

fn main() {
    unsafe {
        MessageBoxA(0, s!("Hello Rust!"), s!("HelloMsg"), MB_OK);
    }
}
