#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, Controls::*, Input::KeyboardAndMouse::*},
    Win32::Globalization::lstrlenW,
};

#[macro_use] mod macros;

static mut I_FONT: i32 = 0;

struct StockFont <'a> {
    id: i32,
    name: &'a str,
}

static STOCK_FONTS: [StockFont;7] = [
    StockFont { id:OEM_FIXED_FONT, name: "OEM_FIXED_FONT" },
    StockFont { id:ANSI_FIXED_FONT, name: "ANSI_FIXED_FONT" },
    StockFont { id:ANSI_VAR_FONT, name: "ANSI_VAR_FONT" },
    StockFont { id:SYSTEM_FONT, name: "SYSTEM_FONT" },
    StockFont { id:DEVICE_DEFAULT_FONT, name: "DEVICE_DEFAULT_FONT" },
    StockFont { id:SYSTEM_FIXED_FONT, name: "SYSTEM_FIXED_FONT" },
    StockFont { id:DEFAULT_GUI_FONT, name: "DEFAULT_GUI_FONT" },
];

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("StockFont");

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
            w!("Stock Fonts"),
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
            WM_CREATE => {
                SetScrollRange(window, SB_VERT, 0, (STOCK_FONTS.len() - 1) as i32, TRUE);
                0
            }

            WM_DISPLAYCHANGE => {
                InvalidateRect(window, std::ptr::null(), TRUE);
                0
            }

            WM_VSCROLL => {
                match loword!(wparam) {
                    SB_TOP => { I_FONT = 0; },
                    SB_BOTTOM => { I_FONT = (STOCK_FONTS.len() - 1) as i32; },
                    SB_LINEUP => { I_FONT -= 1; },
                    SB_PAGEUP => { I_FONT -= 1; },
                    SB_LINEDOWN => { I_FONT += 1; },
                    SB_PAGEDOWN => { I_FONT += 1; },
                    SB_THUMBPOSITION => { I_FONT = hiword!(wparam) as i32; },
                    _ => {},
                }
                I_FONT = std::cmp::max(0, std::cmp::min(STOCK_FONTS.len() as i32 - 1, I_FONT));
                SetScrollPos(window, SB_VERT, I_FONT, TRUE);
                InvalidateRect(window, std::ptr::null(), TRUE);
                0
            }

            WM_KEYDOWN => {
                match wparam as u16 {
                    VK_HOME =>  { SendMessageW(window, WM_VSCROLL, SB_TOP as usize, 0); },
                    VK_END =>   { SendMessageW(window, WM_VSCROLL, SB_BOTTOM as usize, 0); },
                    VK_PRIOR => { SendMessageW(window, WM_VSCROLL, SB_LINEUP as usize, 0); },
                    VK_LEFT =>  { SendMessageW(window, WM_VSCROLL, SB_LINEUP as usize, 0); },
                    VK_UP =>    { SendMessageW(window, WM_VSCROLL, SB_LINEUP as usize, 0); },
                    VK_NEXT =>  { SendMessageW(window, WM_VSCROLL, SB_PAGEDOWN as usize, 0); },
                    VK_RIGHT => { SendMessageW(window, WM_VSCROLL, SB_PAGEDOWN as usize, 0); },
                    VK_DOWN =>  { SendMessageW(window, WM_VSCROLL, SB_PAGEDOWN as usize, 0); },
                    _ => {}
                }
                0
            }
 
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                let stock_font = GetStockObject(STOCK_FONTS[I_FONT as usize].id);
                
                let old_object = SelectObject(hdc, stock_font);

                let mut face_name_wchar: [u16;LF_FACESIZE as usize] = std::mem::zeroed();
                let n = GetTextFaceW(hdc, LF_FACESIZE as i32, face_name_wchar.as_mut_ptr());
                println!("I_FONT={}, stock_font={} old_object={} n={}", I_FONT, stock_font, old_object, n);

                let face_name = if n > 0 { from_wide_ptr(face_name_wchar.as_ptr())} else { "UNKNOWN".to_string() };

                let mut tm: TEXTMETRICW = std::mem::zeroed();
                let is_valid_tm = GetTextMetricsW(hdc, &mut tm);
                println!("is_valid_tm={}", is_valid_tm);
                if is_valid_tm == 0 {
                    SelectObject(hdc, old_object);
                    SetTextColor(hdc, rgb!(255, 0, 0));
                    let s = format!("GetTextMetricsW() failed. stock font name:{}", STOCK_FONTS[I_FONT as usize].name);
                    let s_wchar = to_wide_chars(&s);
                    TextOutW(hdc, 0, 0, s_wchar.as_ptr(), lstrlenW(s_wchar.as_ptr()));
                    EndPaint(window, &ps);
                    return 0;
                }
                let cx_grid = std::cmp::max(3 * tm.tmAveCharWidth, 2 * tm.tmMaxCharWidth);
                let cy_grid = tm.tmHeight + 3;
                let header = format!(" {}: Face Name = {} CharSet = {}",
                    STOCK_FONTS[I_FONT as usize].name, face_name, tm.tmCharSet);
                println!("header={}", header);
                let header_wchars = to_wide_chars(&header);
                TextOutW(hdc, 0, 0, header_wchars.as_ptr(), lstrlenW(header_wchars.as_ptr()));
                
                SetTextAlign(hdc, TA_TOP | TA_CENTER);

                println!("vertical and horizontal lines");
                for i in 0_i32..17_i32 {
                    MoveToEx(hdc, (i + 2) * cx_grid,  2 * cy_grid, std::ptr::null_mut());
                    LineTo  (hdc, (i + 2) * cx_grid, 19 * cy_grid);

                    MoveToEx(hdc,      cx_grid, (i + 3) * cy_grid, std::ptr::null_mut());
                    LineTo  (hdc, 18 * cx_grid, (i + 3) * cy_grid);
                }

                println!("vertical and horizontal headings");
                for i in 0_i32..16_i32 {
                    let mut s = format!("{:X}-", i);
                    let mut s_wchar = to_wide_chars(&s);
                    let mut x = (2 * i + 5) * cx_grid / 2;
                    let mut y = 2 * cy_grid + 2;
                    TextOutW(hdc, x, y, s_wchar.as_ptr(), lstrlenW(s_wchar.as_ptr()));
                    s = format!("-{:X}", i);
                    s_wchar = to_wide_chars(&s);
                    x = 3 * cx_grid / 2;
                    y = (i + 3) * cy_grid + 2;
                    TextOutW(hdc, x, y, s_wchar.as_ptr(), lstrlenW(s_wchar.as_ptr()));
                }

                // characters
                for y in 0_i32..16_i32 {
                    for x in 0_i32..16_i32 {
                        //s = char::from(16 * x + y);
                        let s = [(16 * x + y) as u16, 0];
                        TextOutW(hdc, (2 * x + 5) * cx_grid / 2, (y + 3) * cy_grid + 2,
                            s.as_ptr(), lstrlenW(s.as_ptr()));
                    }
                }

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