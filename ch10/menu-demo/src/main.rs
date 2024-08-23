#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*, Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
    Win32::System::Diagnostics::Debug::MessageBeep,
};

#[macro_use] mod macros;
mod resource;

const WINDOW_CLASS: PCWSTR = w!("MENUDEMO");
static mut CX_CLIENT:i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut SELECTION: i32 = resource::IDM_BKGND_WHITE;
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
            w!("Menu Demonstration"),
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
            WM_COMMAND => {
                let menu = GetMenu(window);
                println!("wparam.0={}", wparam.0);
                let id:i32 = loword!(wparam.0) as i32;
                println!("id={}", id);
                match id {
                    resource::IDM_FILE_NEW..=resource::IDM_FILE_SAVE_AS => {
                        let _ = MessageBeep(MESSAGEBOX_STYLE(0));
                        LRESULT(0)
                    },
                    resource::IDM_APP_EXIT => {
                        SendMessageW(window, WM_CLOSE, WPARAM(0), LPARAM(0));
                        LRESULT(0)
                    }
                    resource::IDM_EDIT_UNDO..=resource::IDM_EDIT_CLEAR => {
                        let _ = MessageBeep(MESSAGEBOX_STYLE(0));
                        LRESULT(0)
                    },
                    resource::IDM_BKGND_WHITE..=resource::IDM_BKGND_BLACK => {
                        let colors: [GET_STOCK_OBJECT_FLAGS; 5]
                            = [ WHITE_BRUSH, LTGRAY_BRUSH, GRAY_BRUSH, DKGRAY_BRUSH, BLACK_BRUSH];

                        CheckMenuItem(menu, SELECTION as u32, MF_UNCHECKED.0);
                        SELECTION = loword!(wparam.0) as i32;
                        CheckMenuItem(menu, SELECTION as u32, MF_CHECKED.0);
                        let brush =
                            colors[(SELECTION - resource::IDM_BKGND_WHITE) as usize];
                        println!("brush={:?}", brush);
                        let stock_object = GetStockObject(brush);
                        println!("stock_object={:?}", stock_object);
                        let r = SetClassLongW(window, GCL_HBRBACKGROUND, stock_object.0 as i32);
                        let x = GetClassLongW(window, GCL_HBRBACKGROUND);
                        println!("r={} x={}", r, x);
                        let _ = InvalidateRect(window, None, TRUE);
                        LRESULT(0)
                    }
                    _ => {
                        println!("WM_COMMAND DEFAULT");
                        DefWindowProcW(window, message, wparam, lparam)
                    }
                }

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