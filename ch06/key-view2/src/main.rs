#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, Input::KeyboardAndMouse::*},
    Win32::Globalization::lstrlenW,
};

#[macro_use] mod macros;

static mut CHAR_SET: i32 = DEFAULT_CHARSET as i32;
static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;
static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut CX_CLIENT_NAX: i32 = 0;
static mut CY_CLIENT_MAX: i32 = 0;
static mut LINES_MAX: i32 = 0;
static mut LINES: i32 = 0;

static mut PMSG: Vec<MSG> = Vec::new();
const TOP: *const u16 = w!("Message       Key        Char     Repeat Scan Ext ALT Prev Tran");
const UND: *const u16 = w!("_______       ___        ____     ______ ____ ___ ___ ____ ____");
const MESSAGE_TEXT: [&str; 8] = ["WM_KEYDOWN", "WM_KEYUP", "WM_CHAR", "WM_DEADCHAR",
                                "WM_SYSKEYDOWN", "WM_SYSKEYUP", "WM_SYSCHAR", "WM_SYSDEADCHAR",];

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("KeyView2");

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: LoadIconW(0, IDI_APPLICATION),
            hCursor: LoadCursorW(0, IDC_ARROW),
            hbrBackground: GetStockObject(WHITE_BRUSH),
            lpszMenuName: std::ptr::null(),
            lpszClassName: app_name,
        };

        let atom = RegisterClassW(&wnd_class);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExW(
            0,
            app_name,
            w!("Keyboard Viewer #2"),
            WS_OVERLAPPEDWINDOW | WS_VSCROLL | WS_HSCROLL,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            0,
            0,
            instance,
            std::ptr::null(),
        );

        ShowWindow(hwnd, SW_SHOWNORMAL);
        UpdateWindow(hwnd);

        let mut message = std::mem::zeroed();

        while GetMessageW(&mut message, 0, 0, 0) != 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

// WM_CREATE, WM_DISPLAYCHANGE からコールされる
unsafe fn init(hwnd: HWND) {
    CX_CLIENT_NAX = GetSystemMetrics(SM_CXMAXIMIZED);
    CY_CLIENT_MAX = GetSystemMetrics(SM_CYMAXIMIZED);
    let hdc = GetDC(hwnd);
    let mut tm: TEXTMETRICW = std::mem::zeroed();
    let font = CreateFontW(0, 0, 0, 0, 0, 0, 0, 0, 
                CHAR_SET as u32, 0, 0, 0,
                FIXED_PITCH as u32, std::ptr::null());
    let old_object = SelectObject(hdc, font);
    GetTextMetricsW(hdc, &mut tm);
    CX_CHAR = tm.tmAveCharWidth;
    CY_CHAR = tm.tmHeight;
    DeleteObject(old_object);
    ReleaseDC(hwnd, hdc);
    PMSG.clear();
    LINES_MAX = CY_CLIENT_MAX / CY_CHAR;
    LINES = 0;
}

unsafe fn redraw(hwnd: HWND) {
    InvalidateRect(hwnd, std::ptr::null(), TRUE);
}

unsafe fn disp_key_msg(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) {
    let msg: MSG = MSG {
        hwnd: hwnd,
        message: message,
        wParam: wparam,
        lParam: lparam,
        time: 0,
        pt: POINT{ x:0, y:0, },
    };
    PMSG.push(msg);
    while PMSG.len() as i32 > LINES_MAX {
        PMSG.remove(0);
    }
    let rect: RECT = RECT{
        left:0,
        right:CX_CLIENT,
        top: CY_CHAR,
        bottom: CY_CHAR * (CY_CLIENT / CY_CHAR),
    };
    ScrollWindow(hwnd, 0, -CY_CHAR, &rect, &rect);
}

fn from_wide_ptr(ptr: *const u16) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    unsafe {
        let len = (0..std::isize::MAX).position(|i| *ptr.offset(i) == 0).unwrap();
        let slice = std::slice::from_raw_parts(ptr, len);
        OsString::from_wide(slice).to_string_lossy().into_owned()
    }
}

fn to_wide_chars(str: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_INPUTLANGCHANGE => {
                CHAR_SET = wparam as i32;
                init(window);
                redraw(window);
                TRUE as isize
            }
            WM_CREATE => {
                init(window);
                redraw(window);
                0
            }

            WM_DISPLAYCHANGE => {
                init(window);
                redraw(window);
                0
            }

            WM_SIZE => {
                CX_CLIENT = loword!(lparam);
                CY_CLIENT = hiword!(lparam);
                redraw(window);
                0
            }

            WM_KEYDOWN => {
                disp_key_msg(window, message, wparam, lparam);
                DefWindowProcW(window, message, wparam, lparam)
            }

            WM_KEYUP => {
                disp_key_msg(window, message, wparam, lparam);
                DefWindowProcW(window, message, wparam, lparam)
            }

            WM_CHAR => {
                disp_key_msg(window, message, wparam, lparam);
                DefWindowProcW(window, message, wparam, lparam)
            }

            WM_DEADCHAR => {
                disp_key_msg(window, message, wparam, lparam);
                DefWindowProcW(window, message, wparam, lparam)
            }

            WM_SYSKEYDOWN => {
                disp_key_msg(window, message, wparam, lparam);
                DefWindowProcW(window, message, wparam, lparam)
            }

            WM_SYSKEYUP => {
                disp_key_msg(window, message, wparam, lparam);
                DefWindowProcW(window, message, wparam, lparam)
            }

            WM_SYSCHAR => {
                disp_key_msg(window, message, wparam, lparam);
                DefWindowProcW(window, message, wparam, lparam)
            }

            WM_SYSDEADCHAR => {
                disp_key_msg(window, message, wparam, lparam);
                DefWindowProcW(window, message, wparam, lparam)
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                let font = CreateFontW(0, 0, 0, 0, 0, 0, 0, 0, 
                    CHAR_SET as u32, 0, 0, 0,
                    FIXED_PITCH as u32, std::ptr::null());
                let old_object = SelectObject(hdc, font);
                    
                SetBkMode(hdc, TRANSPARENT as i32);
                TextOutW(hdc, 0, 0, TOP, lstrlenW(TOP));
                TextOutW(hdc, 0, 0, UND, lstrlenW(UND));

                let mut i = 0;
                for msg in PMSG.iter().rev() {
                    let is_char_msg =   msg.message == WM_CHAR ||
                                        msg.message == WM_SYSCHAR ||
                                        msg.message == WM_DEADCHAR ||
                                        msg.message == WM_SYSDEADCHAR;
                    
                    let mut key_name_wchar: [u16;32] = std::mem::zeroed();
                    let n = GetKeyNameTextW(msg.lParam as i32, key_name_wchar.as_mut_ptr(), 32);
                    let key_name = if n > 0 { from_wide_ptr(key_name_wchar.as_ptr()) } else { "UNKNOWN".to_string() };
                    let key_val = from_wide_ptr([msg.wParam as u16, 0].as_ptr());
                    let s = if is_char_msg {
                        format!("{:<13}            {:#06X}{}{} {:>6} {:>4} {:>3} {:>3} {:>4} {:>4}",
                                MESSAGE_TEXT[(msg.message - WM_KEYFIRST) as usize],
                                msg.wParam,
                                " ",
                                &key_val,
                                loword!(msg.lParam),
                                hiword!(msg.lParam) & 0xFF,
                                if 0x01000000 & msg.lParam != 0 {"Yes"} else {"No"},
                                if 0x20000000 & msg.lParam != 0 {"Yes"} else {"No"},
                                if 0x40000000 & msg.lParam != 0 {"Down"} else {"Up"},
                                if 0x80000000 & msg.lParam != 0 {"Up"} else {"Down"},
                        )
                    } else {
                        format!("{:<13} {:3} {:<15}{}{:>6} {:>4} {:>3} {:>3} {:>4} {:>4}",
                                MESSAGE_TEXT[(msg.message - WM_KEYFIRST) as usize],
                                msg.wParam,
                                &key_name,
                                ' ',
                                loword!(msg.lParam),
                                hiword!(msg.lParam) & 0xFF,
                                if 0x01000000 & msg.lParam != 0 {"Yes"} else {"No"},
                                if 0x20000000 & msg.lParam != 0 {"Yes"} else {"No"},
                                if 0x40000000 & msg.lParam != 0 {"Down"} else {"Up"},
                                if 0x80000000 & msg.lParam != 0 {"Up"} else {"Down"},
                        )
                    };
                    let wide_chars = to_wide_chars(&s);
                    let y = (CY_CLIENT / CY_CHAR - 1 - i) * CY_CHAR;
                    if y > 0 {
                        TextOutW(hdc, 0, y, wide_chars.as_ptr(), lstrlenW(wide_chars.as_ptr()));
                    }
                    i += 1;
                }
                DeleteObject(old_object);
                EndPaint(window, &ps);
                0
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}