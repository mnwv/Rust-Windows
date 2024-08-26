#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*, Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::{GetModuleHandleW, },
    Win32::UI::WindowsAndMessaging::*,
};

#[macro_use] mod macros;
// mod resource;

const APP_NAME: PCWSTR = w!("About1");

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            // hIcon: LoadIconW(HINSTANCE(std::ptr::null_mut()), IDI_APPLICATION)?,
            hIcon: LoadIconW(instance, APP_NAME)?,
            hCursor: LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW)?,
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: APP_NAME,
            lpszClassName: APP_NAME,
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            APP_NAME,
            w!("About Box Demo Program"),
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
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_COMMAND => {
                let n = GetWindowLongPtrW(window, GWLP_HINSTANCE);
                let instance = HINSTANCE(n as *mut core::ffi::c_void);
                println!("{}", type_of(&instance));
                match loword!(wparam.0) as u32 {
                    104 => {
                        DialogBoxParamW(instance,
                                        w!("ABOUTBOX_VS"),
                                        window,
                                        Some(about_dlg_proc),
                                        LPARAM(0));
                    }               //IDM_ABOUT	104
                    40001 => {
                        DialogBoxParamW(instance,
                                        w!("ABOUTBOX"),
                                        window,
                                        Some(about_dlg_proc),
                                        LPARAM(0));

                    }
                    105 => {
                        let _ = PostMessageW(window, WM_CLOSE, WPARAM(0), LPARAM(0));
                    }               // IDM_EXIT 105
                    _ => {}
                }
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

extern "system" fn about_dlg_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> isize {
    unsafe {
        match message {
            WM_INITDIALOG => {
                println!("WM_INITDIALOG lparam={:?}", lparam);
                1
            }
            WM_COMMAND => {
                println!("loword!(wparam.0)={}", loword!(wparam.0));    // loword!(wparam.0)=1
                println!("IDOK={:?}", IDOK);                            // IDOK=MESSAGEBOX_RESULT(1)
                println!("IDCANCEL={:?}", IDCANCEL);
                match MESSAGEBOX_RESULT(loword!(wparam.0) as i32) {
                    IDOK => {
                        println!("IDOK");
                        let _ = EndDialog(window, 0);
                        1
                    }
                    IDCANCEL => {
                        println!("IDCANCEL");
                        let _ = EndDialog(window, 0);
                        1
                    }
                    _ => {
                        println!("Unknown WM_COMMAND. wparam.0={}", wparam.0);
                        0
                    }
                }
            }
            _ => { 0 }
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

#[allow(dead_code)]
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
