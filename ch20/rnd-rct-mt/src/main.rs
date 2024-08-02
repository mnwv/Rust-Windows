#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*, Win32::Foundation::*, 
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::WindowsAndMessaging::*,
    Win32::System::Threading::{QueueUserWorkItem, WORKER_THREAD_FLAGS, Sleep,},
};

#[macro_use] mod macros;

static CX_CLIENT: std::sync::RwLock<i32> = std::sync::RwLock::new(0);
static CY_CLIENT: std::sync::RwLock<i32> = std::sync::RwLock::new(0);

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let window_class = w!("RndRctMT");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: LoadIconW(HINSTANCE(std::ptr::null_mut()), IDI_APPLICATION).unwrap(),
            hCursor: LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW).unwrap(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: windows_strings::PCWSTR(std::ptr::null()),
            lpszClassName: window_class,
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            window_class,
            w!("RndRctMT"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        )?;

        let mut message = MSG::default();

        while GetMessageW(&mut message, None, 0, 0).into() {
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

#[allow(unreachable_code)]
unsafe extern "system" fn thread(param: *mut core::ffi::c_void) -> u32 {
    use rand::prelude::*;

    let hwnd: HWND = HWND(param);
    // 以下はHWNDが渡されていることの確認
    // let mut text: [u16; 100] = [0; 100];
    // let n = GetWindowTextW(hwnd, &mut text);
    // println!("GetWindowTextW() returns:{}", n);
    // for i in 0..n as usize {
    //     print!(" {:#06X}", text[i]);
    // }
    // println!();
    // // let parts = std::slice::from_raw_parts(text.as_ptr(), n as usize);
    // // let s = String::from_utf16(parts).unwrap();
    // let s = PWSTR(text.as_ptr().cast_mut()).to_string().unwrap();
    // println!("GetWindowTextW():{}", s);
    /*
        GetWindowTextW() returns:8
        0x0052 0x006E 0x0064 0x0052 0x0063 0x0074 0x004D 0x0054
        GetWindowTextW():RndRctMT
    */
    loop {
        let cx_client = *CX_CLIENT.read().unwrap();
        let cy_client = *CY_CLIENT.read().unwrap();
        if cx_client != 0 && cy_client != 0 {
            let x_left = random::<i32>() % cx_client;
            let x_right = random::<i32>() % cx_client;
            let y_top = random::<i32>() % cy_client;
            let y_bottom = random::<i32>() % cy_client;
            let red = random::<i32>() % 255;
            let green = random::<i32>() % 255;
            let blue = random::<i32>() % 255;

            let hdc = GetDC(hwnd);
            let brush = CreateSolidBrush(COLORREF(rgb!(red, green, blue)));
            SelectObject(hdc, brush);
            let _ = Rectangle(hdc,
                        std::cmp::min(x_left, x_right),
                        std::cmp::min(y_top, y_bottom),
                        std::cmp::max(x_left, x_right),
                        std::cmp::max(y_top, y_bottom)
                    );
            ReleaseDC(hwnd, hdc);
            let _ = DeleteObject(brush);
            Sleep(100);
        }
    }
    0
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                println!("WM_CREATE");
                let _ = QueueUserWorkItem(Some(thread), Some(window.0), WORKER_THREAD_FLAGS(0));
                LRESULT(0)
            }
            WM_SIZE => {
                println!("WM_SIZE");
                *CX_CLIENT.write().unwrap() = loword!(lparam.0) as i32;
                *CY_CLIENT.write().unwrap() = hiword!(lparam.0) as i32;
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}


#[allow(dead_code)]
fn type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

#[allow(dead_code)]
fn to_wide_chars(str: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}