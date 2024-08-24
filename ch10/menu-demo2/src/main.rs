#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*, Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::{GetModuleHandleW, },
    Win32::UI::WindowsAndMessaging::*,
    Win32::System::Diagnostics::Debug::MessageBeep,
};
use resource_id::*;
#[macro_use] mod macros;
mod menu;
mod resource_id;

const ID_TIMER: usize = 1;
const WINDOW_CLASS: PCWSTR = w!("MenuDemo2");
static mut SELECTION: usize = IDM_BKGND_WHITE;

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: LoadIconW(HINSTANCE(std::ptr::null_mut()), IDI_APPLICATION).unwrap(),
            hCursor: LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW).unwrap(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: WINDOW_CLASS,
            lpszClassName: WINDOW_CLASS,
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            WINDOW_CLASS,
            w!("Menu Demonstration #2"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            menu::create_menu()?,
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
            WM_COMMAND => {
                let menu = GetMenu(window);
                println!("wparam.0={}", wparam.0);
                let id:usize = loword!(wparam.0) as usize;
                println!("id={}", id);
                match id {
                    IDM_FILE_NEW..=IDM_FILE_SAVE_AS => {
                        let _ = MessageBeep(MB_OK);
                        LRESULT(0)
                    },
                    IDM_APP_EXIT => {
                        SendMessageW(window, WM_CLOSE, WPARAM(0), LPARAM(0));
                        LRESULT(0)
                    }
                    IDM_EDIT_UNDO..=IDM_EDIT_CLEAR => {
                        let _ = MessageBeep(MB_OK);
                        LRESULT(0)
                    },
                    IDM_BKGND_WHITE..=IDM_BKGND_BLACK => {
                        let colors: [GET_STOCK_OBJECT_FLAGS; 5]
                            = [ WHITE_BRUSH, LTGRAY_BRUSH, GRAY_BRUSH, DKGRAY_BRUSH, BLACK_BRUSH];
            
                        CheckMenuItem(menu, SELECTION as u32, MF_UNCHECKED.0);
                        SELECTION = loword!(wparam.0) as usize;
                        CheckMenuItem(menu, SELECTION as u32, MF_CHECKED.0);
                        let brush =
                            colors[(SELECTION - IDM_BKGND_WHITE) as usize];
                        let stock_object = GetStockObject(brush);
                        let r = SetClassLongPtrW(window, GCLP_HBRBACKGROUND, stock_object.0 as isize);
                        if r == 0 {
                            let n = GetLastError();
                            println!("GetLastError() returns:{:?}", n); // GetLastError() returns:WIN32_ERROR(1413)
                            let err = get_err_msg(n.0 as i32);
                            println!("err:{}", err);    // err:インデックスが無効です。
                        }
                        let x = GetClassLongPtrW(window, GCLP_HBRBACKGROUND);
                        println!("r={} x={}", r, x);
                        let _ = InvalidateRect(window, None, TRUE);
                        LRESULT(0)
                    },
                    IDM_TIMER_START => {
                        let ret = SetTimer(window, ID_TIMER, 1000, None);
                        if ret != 0 {
                            let _ = EnableMenuItem(menu, IDM_TIMER_START as u32, MF_GRAYED);
                            let _ = EnableMenuItem(menu, IDM_TIMER_STOP as u32, MF_ENABLED);
                        }
                        LRESULT(0)
                    }
                    IDM_TIMER_STOP => {
                        let _ = KillTimer(window, ID_TIMER);
                        let _ = EnableMenuItem(menu, IDM_TIMER_START as u32, MF_ENABLED);
                        let _ = EnableMenuItem(menu, IDM_TIMER_STOP as u32, MF_GRAYED);
                        LRESULT(0)
                    }
                    IDM_APP_HELP => {
                        MessageBoxW(window, w!("Help not yet implemented"),
                                    WINDOW_CLASS, MB_ICONEXCLAMATION | MB_OK);
                        LRESULT(0)
                    }
                    IDM_APP_ABOUT => {
                        MessageBoxW(window,
                                    w!("Menu Demonstration Program\r\n(c) WVMN, 2024"),
                                    WINDOW_CLASS, MB_ICONINFORMATION | MB_OK);
                        LRESULT(0)
                    }
                    _ => {
                        println!("WM_COMMAND DEFAULT");
                        DefWindowProcW(window, message, wparam, lparam)
                    }
                }
            }
            WM_TIMER => {
                let _ = MessageBeep(MB_OK);
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

fn get_err_msg(err_code: i32) -> String {
    use windows::Win32::{
        System::Diagnostics::Debug::{
            FormatMessageW, FORMAT_MESSAGE_ALLOCATE_BUFFER,
            FORMAT_MESSAGE_FROM_SYSTEM,
        },
        Foundation::{LocalFree, HLOCAL},
    };

    unsafe {
        let mut text: *mut u16 = std::ptr::null_mut();
        let n = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            None,
            err_code as u32,
            0,
            PWSTR(&mut text as *mut _ as *mut _),
            2048,
            None);
        if n > 0 {
            let parts = std::slice::from_raw_parts(text, n as usize);
            let s = String::from_utf16(parts).unwrap();
            LocalFree(HLOCAL(text as *mut core::ffi::c_void));
            return s;
        }
        "Failed:FormatMessageW()".to_string()
    }
}
