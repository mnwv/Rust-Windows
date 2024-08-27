#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*, Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::{GetModuleHandleW, },
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Controls::CheckRadioButton,
    Win32::UI::Input::KeyboardAndMouse::SetFocus,
};
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;

#[macro_use] mod macros;
// mod resource;

const APP_NAME: PCWSTR = w!("About2");
static mut CURRENT_COLOR: i32 = 1000;   // IDC_BLACK
static mut CURRENT_FIGURE: i32 = 1008;  // IDC_RECT

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
                match loword!(wparam.0) as u32 {
                    40001 => {
                        let n = DialogBoxParamW(instance,
                                        w!("ABOUTBOX"),
                                        window,
                                        Some(about_dlg_proc),
                                        LPARAM(0));
                        if n != 0 {
                            InvalidateRect(window, None, TRUE).unwrap();
                        }
                        LRESULT(0)
                    },
                    _ => DefWindowProcW(window, message, wparam, lparam),
                }
            }
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                BeginPaint(window, &mut ps);
                EndPaint(window, &mut ps).unwrap();

                paint_window(window, CURRENT_COLOR, CURRENT_FIGURE);
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

extern "system" fn about_dlg_proc(dlg: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> isize {
    unsafe {
        static mut COLOR: i32 = 0;
        static mut FIGURE: i32 = 0;
        static mut CTRL_BLOCK: HWND = HWND(std::ptr::null_mut());

        match message {
            WM_INITDIALOG => {
                println!("WM_INITDIALOG wparam={:?} lparam={:?}", wparam, lparam);
                let dflt_hwnd = HWND(wparam.0 as *mut core::ffi::c_void);
                let id = GetDlgCtrlID(dflt_hwnd);   // 1000 IDC_BLACK
                println!("dflt_hwnd id={}", id);
                // print_focused_ctrl(dlg);

                COLOR = CURRENT_COLOR;
                FIGURE = CURRENT_FIGURE;
                CheckRadioButton(dlg, 1000, 1007, COLOR).unwrap();
                                                    // IDC_BLACK~IDC_WHITE
                CheckRadioButton(dlg, 1008, 1009, FIGURE).unwrap();
                                                    // IDC_RECT~IDC_ELLIPSE
                let result = GetDlgItem(dlg, 1010 /*IDC_PAINT*/);
                if let Some(err) = result.as_ref().err() {
                    panic!("GetDlgItem() failed. Error:{}", err.message());
                }
                CTRL_BLOCK = GetDlgItem(dlg, 1010 /*IDC_PAINT*/).unwrap();
                let result = GetDlgItem(dlg, COLOR);
                if let Some(err) = result.as_ref().err() {
                    panic!("GetDlgItem(COLOR) failed. COLOR={} Error:{}", COLOR, err.message());
                }
                let color_ctrl = result.ok().unwrap();
                let result = SetFocus(color_ctrl);
                if let Some(err) = result.as_ref().err() {
                    println!("SetFocus(ctrl) failed. ctrl={:?} Error:{}", color_ctrl, err.code());
                }
                //
                // SetFocus(GetDlgItem(dlg, COLOR).unwrap()).unwrap();
                0
            }
            WM_COMMAND => {
                // print_focused_ctrl(dlg);
                match loword!(wparam.0) as i32 {
                    1 => {
                        println!("IDOK");
                        CURRENT_COLOR = COLOR;
                        CURRENT_FIGURE = FIGURE;
                        EndDialog(dlg, TRUE.0 as isize).unwrap();
                        TRUE.0 as isize
                    }
                    2 => {
                        println!("IDCANCEL");
                        let _ = EndDialog(dlg, FALSE.0 as isize);
                        TRUE.0 as isize
                    }
                    1000..=1007 => {    // IDC_BLACK ~
                        COLOR = loword!(wparam.0) as i32;
                        CheckRadioButton(dlg,
                                         1000,      // IDC_BLACK
                                         1007,      // IDC_WHITE
                                         loword!(wparam.0) as i32).unwrap();
                        paint_the_block(CTRL_BLOCK, COLOR, FIGURE);
                        TRUE.0 as isize
                    }
                    1008..=1009 => {    // IDC_RECT || IDC_ELLIPSE
                        FIGURE = loword!(wparam.0) as i32;
                        CheckRadioButton(dlg,
                                         1008,      // IDC_RECT
                                         1009,      // IDC_ELLIPSE
                                         loword!(wparam.0) as i32).unwrap();
                        paint_the_block(CTRL_BLOCK, COLOR, FIGURE);
                        TRUE.0 as isize
                    }
                    _ => {
                        println!("Unknown WM_COMMAND. wparam.0={}", wparam.0);
                        FALSE.0 as isize
                    }
                }
            }
            WM_PAINT => {
                // print_focused_ctrl(dlg);
                paint_the_block(CTRL_BLOCK, COLOR, FIGURE);
                FALSE.0 as isize
            }
            _ => { 0 }
        }
    }
}

fn paint_the_block(ctrl: HWND, color: i32, figure: i32) {
    unsafe {
        InvalidateRect(ctrl, None, TRUE).unwrap();
        UpdateWindow(ctrl).unwrap();
        paint_window(ctrl, color, figure);
    }
}

fn paint_window(window: HWND, color: i32, figure: i32) {
    static COLORS: [COLORREF; 8] = [COLORREF(rgb!(  0,   0,   0)), COLORREF(rgb!(  0,   0, 255)),
        COLORREF(rgb!(  0, 255,   0)), COLORREF(rgb!(  0, 255, 255)),
        COLORREF(rgb!(255,   0,   0)), COLORREF(rgb!(255,   0, 255)),
        COLORREF(rgb!(255, 255,   0)), COLORREF(rgb!(255, 255, 255))];
    unsafe {
        let hdc = GetDC(window);
        let mut rect: RECT = std::mem::zeroed();
        GetClientRect(window, &mut rect).unwrap();
        let brush = CreateSolidBrush(COLORS[(color - 1000) as usize]); // IDC_BLACK:1000
        let brush = HBRUSH(SelectObject(hdc, brush).0);
        if figure == 1008 {     //IDC_RECT:1008
            Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom).unwrap();
        } else {
            Ellipse(hdc, rect.left, rect.top, rect.right, rect.bottom).unwrap();
        }
        DeleteObject(SelectObject(hdc, brush)).unwrap();
        ReleaseDC(window, hdc);
    }
}

#[allow(dead_code)]
#[allow(unused_variables)]
unsafe fn print_focused_ctrl(dlg: HWND) {
    let focus_ctrl = GetFocus();
    let id = GetDlgCtrlID(focus_ctrl);   // 1000 IDC_BLACK
    println!("focus_ctrl={:?} id={}", focus_ctrl, id);    // focus_ctrl=HWND(0x0)
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
