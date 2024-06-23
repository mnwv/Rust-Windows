#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, },
    Win32::System::SystemInformation::*,
    Win32::Globalization::*,
};

#[macro_use] mod macros;

const ID_TIMER: usize = 1;
static mut IS_24H: bool = false;
static mut SUPPRESS: bool = false;
static mut BRUSH_RED: HBRUSH = 0;
static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;
const SEVEN_SEGMENT: [[bool;7];10] = [
    [true,true,true,false,true,true,true,],     // 0
    [false,false,true,false,false,true,false,], // 1
    [true,false,true,true,true,false,true,],    // 2
    [true,false,true,true,false,true,true,],    // 3
    [false,true,true,true,false,true,false,],   // 4
    [true,true,false,true,false,true,true,],    // 5
    [true,true,false,true,true,true,true,],     // 6
    [true,false,true,false,false,true,false,],  // 7
    [true,true,true,true,true,true,true,],      // 8
    [true,true,true,true,false,true,true,],     // 9
];
const SEGMENT: [[POINT;6];7] = [
    [POINT{x:7,y:6}, POINT{x:11,y:2},POINT{x:31,y:2},POINT{x:35,y:6},POINT{x:31,y:10},POINT{x:11,y:10},],
    [POINT{x:6,y:7},POINT{x:10,y:11},POINT{x:10,y:31},POINT{x:6,y:35},POINT{x:2,y:31},POINT{x:2,y:11},],
    [POINT{x:36,y:7},POINT{x:40,y:11},POINT{x:40,y:31},POINT{x:36,y:35},POINT{x:32,y:31},POINT{x:32,y:11},],
    [POINT{x:7,y:36},POINT{x:11,y:32},POINT{x:31,y:32},POINT{x:35,y:36},POINT{x:31,y:40},POINT{x:11,y:40},],
    [POINT{x:6,y:37},POINT{x:10,y:41},POINT{x:10,y:61},POINT{x:6,y:65},POINT{x:2,y:61},POINT{x:2,y:41},],
    [POINT{x:36,y:37},POINT{x:40,y:41},POINT{x:40,y:61},POINT{x:36,y:65},POINT{x:32,y:61},POINT{x:32,y:41},],
    [POINT{x:7,y:66},POINT{x:11,y:62},POINT{x:31,y:62},POINT{x:35,y:66},POINT{x:31,y:70},POINT{x:11,y:70},],
];

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("DigClock");

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
            w!("Digital Clock"),
            WS_OVERLAPPEDWINDOW,
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

fn display_digit(hdc: HDC, number:i32) {
    for seg in 0..7 {
        if SEVEN_SEGMENT[number as usize][seg] {
            unsafe {
                Polygon(hdc, &SEGMENT[seg] as *const POINT, 6);
            }
        }
    }
}

fn display_two_digits(hdc: HDC, number: i32, suppress: bool) {
    unsafe {
        if !suppress || number / 10 != 0 {
            display_digit(hdc, number / 10);
        }
        OffsetWindowOrgEx(hdc, -42, 0, std::ptr::null_mut());
        display_digit(hdc, number % 10);
        OffsetWindowOrgEx(hdc, -42, 0, std::ptr::null_mut());
    }
}

fn display_colon(hdc: HDC) {
    let pt_colon: [[POINT;4];2] = [
        [POINT{x:2,y:21},POINT{x:6,y:17},POINT{x:10,y:21},POINT{x:6,y:26},],
        [POINT{x:2,y:51},POINT{x:6,y:47},POINT{x:10,y:51},POINT{x:6,y:55},],
    ];
    unsafe {
        Polygon(hdc, &pt_colon[0] as *const POINT, 4);
        Polygon(hdc, &pt_colon[1] as *const POINT, 4);
        OffsetWindowOrgEx(hdc, -12, 0, std::ptr::null_mut());
    }
}

fn display_time(hdc: HDC) {
    unsafe {
        let mut st: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut st);

        if IS_24H {
            display_two_digits(hdc, st.wHour as i32, SUPPRESS);
        } else {
            let hour = if st.wHour as i32 % 12 != 0 { st.wHour as i32} else {12};
            display_two_digits(hdc, hour, SUPPRESS);
        }

        display_colon(hdc);
        display_two_digits(hdc, st.wMinute as i32, false);
        display_colon(hdc);
        display_two_digits(hdc, st.wSecond as i32, false);
    }
}

fn init(hwnd: HWND) {
    unsafe {
        let mut buf: [u16;2] = std::mem::zeroed();
        GetLocaleInfoW(/*LOCALE_USER_DEFAULT*/ 0x0400, LOCALE_ITIME, &mut buf[0], 2);
        IS_24H = if buf[0] == '1' as u16 { true } else { false };
        GetLocaleInfoW(/*LOCALE_USER_DEFAULT*/ 0x0400, LOCALE_ITLZERO, &mut buf[0], 2);
        SUPPRESS = if buf[0] == '0' as  u16 { true } else { false };
        InvalidateRect(hwnd, std::ptr::null_mut(), TRUE);
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                BRUSH_RED = CreateSolidBrush(rgb!(255,0,0));
                SetTimer(window, ID_TIMER, 1000, None);
                init(window);
                0
            },
            WM_SETTINGCHANGE => {
                init(window);
                0
            },
            WM_SIZE => {
                CX_CLIENT = loword!(lparam) as i32;
                CY_CLIENT = hiword!(lparam) as i32;
                0
            },
            WM_TIMER => {
                InvalidateRect(window, std::ptr::null_mut(), TRUE);
                0
            },
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                SetMapMode(hdc, MM_ISOTROPIC);
                SetWindowExtEx(hdc, 276, 72, std::ptr::null_mut());
                SetViewportExtEx(hdc, CX_CLIENT, CY_CLIENT, std::ptr::null_mut());

                SetWindowOrgEx(hdc, 138, 36, std::ptr::null_mut());
                SetViewportOrgEx(hdc, CX_CLIENT / 2, CY_CLIENT / 2, std::ptr::null_mut());

                SelectObject(hdc, GetStockObject(NULL_PEN));
                SelectObject(hdc, BRUSH_RED);

                display_time(hdc);

                EndPaint(window, &ps);
                0
            },
            WM_DESTROY => {
                KillTimer(window, ID_TIMER);
                DeleteObject(BRUSH_RED);
                PostQuitMessage(0);
                0
            },
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
