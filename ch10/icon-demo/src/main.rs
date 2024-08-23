#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*, Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
};

#[macro_use] mod macros;

const WINDOW_CLASS: PCWSTR = w!("ICONDEMO");
static mut CX_CLIENT:i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut ICON: HICON = HICON(std::ptr::null_mut());
static mut CX_ICON: i32 = 0;
static mut CY_ICON: i32 = 0;

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: LoadIconW(instance, WINDOW_CLASS).unwrap(),
            hCursor: LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW).unwrap(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: PCWSTR(std::ptr::null()),
            lpszClassName: WINDOW_CLASS,
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            WINDOW_CLASS,
            w!("Icon Demo"),
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

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                println!("WM_CREATE");
                let p : *const CREATESTRUCTW = lparam.0 as *const CREATESTRUCTW;
                let instance = (*p).hInstance;
                ICON = LoadIconW(instance, WINDOW_CLASS).unwrap();
                CX_ICON = GetSystemMetrics(SM_CXICON);
                CY_ICON = GetSystemMetrics(SM_CYICON);
                println!("CX_ICON={} CY_ICON={}", CX_ICON, CY_ICON);
                let mut buf: [u16; 256] = [0; 256];
                GetWindowTextW(window, &mut buf);
                let s = from_wide_ptr(buf.as_ptr());
                let s = format!("{}  CX_ICON={} CY_ICON={}", s, CX_ICON, CY_ICON);
                let w = to_wide_chars(&s);
                SetWindowTextW(window, PCWSTR::from_raw(w.as_ptr())).unwrap();
                LRESULT(0)
            }
            WM_SIZE => {
                println!("WM_SIZE");
                CX_CLIENT = loword!(lparam.0) as i32;
                CY_CLIENT = hiword!(lparam.0) as i32;
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                let mut x = 0;
                let mut y = 0;
                while y < CY_CLIENT {
                    while x < CX_CLIENT {
                        DrawIcon(hdc, x, y, ICON).unwrap();
                        x += CX_ICON;
                    }
                    x = 0;
                    y += CY_ICON;
                }
                let _ = EndPaint(window, &mut ps);
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

#[allow(dead_code)]
fn from_wide_ptr(ptr: *const u16) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    unsafe {
        let len = (0..std::isize::MAX).position(|i| *ptr.offset(i) == 0).unwrap();
        let slice = std::slice::from_raw_parts(ptr, len);
        OsString::from_wide(slice).to_string_lossy().into_owned()
    }
}
