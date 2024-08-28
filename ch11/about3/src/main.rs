#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*, Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::{GetModuleHandleW, },
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::VK_SPACE,
};

#[macro_use] mod macros;
// mod resource;


fn main() -> Result<()> {
    const APP_NAME: PCWSTR = w!("About3");
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let mut wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: LoadIconW(instance, APP_NAME)?,
            hCursor: LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW)?,
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: APP_NAME,
            lpszClassName: APP_NAME,
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.lpfnWndProc = Some(ellip_push_wnd_proc);
        wc.cbClsExtra = 0;
        wc.cbWndExtra = 0;
        wc.hInstance = instance.into();
        wc.hIcon = HICON(std::ptr::null_mut());
        wc.hCursor = LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW)?;
        // let color_index = COLOR_BTNFACE.0 + 1; // COLOR_BTNFACE + 1 相当
        // let index = color_index.into();
        // wc.hbrBackground = GetSysColorBrush(SYS_COLOR_INDEX(index)); //HBRUSH(COLOR_BTNFACE);
        wc.hbrBackground = GetSysColorBrush(SYS_COLOR_INDEX(COLOR_BTNFACE.0)); //HBRUSH(COLOR_BTNFACE);
        wc.lpszMenuName = PCWSTR::null();
        wc.lpszClassName = w!("EllipPush");

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        let _hwnd = CreateWindowExW(
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
        static mut INSTANCE:HINSTANCE = HINSTANCE(std::ptr::null_mut());

        match message {
            WM_CREATE => {
                let cs: &CREATESTRUCTW = &*(lparam.0 as *const CREATESTRUCTW);
                INSTANCE = cs.hInstance;
                LRESULT(0)
            }
            WM_COMMAND => {
                println!("WM_COMMAND wparam.0:{} INSTANCE:{:?}", wparam.0, INSTANCE);
                match loword!(wparam.0) as u32 {
                    40001 => {
                        DialogBoxParamW(INSTANCE,
                                        w!("ABOUTBOX"),
                                        window,
                                        Some(about_dlg_proc),
                                        LPARAM(0));
                        LRESULT(0)
                    },
                    _ => DefWindowProcW(window, message, wparam, lparam),
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

extern "system" fn about_dlg_proc(dlg: HWND, message: u32, wparam: WPARAM, _lparam: LPARAM) -> isize {
    unsafe {
        match message {
            WM_INITDIALOG => {
                TRUE.0 as isize
            }
            WM_COMMAND => {
                match MESSAGEBOX_RESULT(loword!(wparam.0) as i32) {
                    IDOK | IDCANCEL => {
                        EndDialog(dlg, TRUE.0 as isize).unwrap();
                        TRUE.0 as isize
                    }
                    _ => {
                        println!("Unknown WM_COMMAND. wparam.0={}", wparam.0);
                        FALSE.0 as isize
                    }
                }
            }
            _ => { 0 }
        }
    }
}

extern "system" fn ellip_push_wnd_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        // println!("ellip_push_wnd_proc() message={:?}", message);
        match message {
            WM_PAINT => {
                println!("ellip_push_wnd_proc() WM_PAINT");
                let mut rc: RECT = RECT::default();
                let ret = GetClientRect(window, &mut rc);
                println!("GetClientRect() returns:{:?}", ret);
                println!("left:{} right:{} top:{} bottom:{}", rc.left, rc.right, rc.top, rc.bottom);
                let mut text:[u16;40] = [0;40];
                let ret = GetWindowTextW(window, &mut text);
                let pcwstr: PCWSTR = PCWSTR::from_raw(text.as_mut_ptr());
                let len = pcwstr.len();
                println!("pwstr={:?} len={}", pcwstr, len);

                println!("GetWindowTextW() returns:{:?}", ret);
                let mut ps= PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut ps);
                println!("BeginPaint() returns:{:?}", hdc);
                let brush = CreateSolidBrush(COLORREF(GetSysColor(COLOR_WINDOW)));
                println!("CreateSolidBrush() returns:{:?}", brush);
                let brush = HBRUSH(SelectObject(hdc, brush).0);
                let ret = SetBkColor(hdc, COLORREF(GetSysColor(COLOR_WINDOW)));
                println!("SetBkColor() returns:{:?}", ret);
                let ret = SetTextColor(hdc, COLORREF(GetSysColor(COLOR_WINDOWTEXT)));
                println!("SetTextColor() returns:{:?}", ret);
                let ret = Ellipse(hdc, rc.left, rc.top, rc.right, rc.bottom);
                println!("Ellipse() returns:{:?}", ret);
                // let ret = DrawTextW(hdc, &mut text, &mut rc, DT_SINGLELINE | DT_CENTER | DT_VCENTER);
                // let ret = ExtTextOutW(hdc, rc.right, 0, ETO_OPTIONS(0), None, pcwstr, 2, None );
                // println!("DrawTextW() returns:{:?}", ret);
                draw_text(window, hdc, pcwstr);
                let ret = DeleteObject(SelectObject(hdc, brush));
                println!("DeleteObject() returns:{:?}", ret);
                let ret = EndPaint(window, &ps);
                println!("EndPaint() returns:{:?}", ret);
                LRESULT(0)
            }
            WM_KEYUP | WM_LBUTTONUP => {
                if message == WM_KEYUP && wparam.0 != VK_SPACE.0 as usize {
                    return DefWindowProcW(window, message, wparam, lparam);
                }
                SendMessageW(
                    GetParent(window).unwrap(),
                    WM_COMMAND,
                    WPARAM(GetWindowLongW(window, GWL_ID) as usize),
                    LPARAM(window.0 as isize));
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam)
        }
    }
}

fn draw_text(window: HWND, hdc: HDC, pcwstr: PCWSTR) {
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::Graphics::Gdi::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    unsafe {
        let mut rc: RECT = std::mem::zeroed();
        GetClientRect(window.0, &mut rc);
        let hdc = hdc.0;
        DrawTextW(hdc, pcwstr.0, -1, &mut rc, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
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
