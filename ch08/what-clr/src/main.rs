#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::{WindowsAndMessaging::*, },
};

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[macro_use] mod macros;

const ID_TIMER: usize = 1;
static mut CR: COLORREF = 0;        // u32
static mut CR_LAST: COLORREF = 0;   // u32
static mut HDC_SCREEN: HDC = 0;     // isize

fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("WhatClr");

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

        let mut cx_window = 0;
        let mut cy_window = 0;
        find_window_size(&mut cx_window, &mut cy_window);

        let hwnd = CreateWindowExW(
            0,
            app_name,
            w!("What Color"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            cx_window,
            cy_window,
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

fn find_window_size(cx_window: &mut i32, cy_window: &mut i32) {
    unsafe {
        let hdc_screen = CreateICW(w!("DISPLAY"), std::ptr::null(), std::ptr::null(), std::ptr::null());
        let mut tm: TEXTMETRICW = std::mem::zeroed();
        GetTextMetricsW(hdc_screen, &mut tm);
        DeleteDC(hdc_screen);
        *cx_window = 2 * GetSystemMetrics(SM_CXBORDER) + 12 * tm.tmAveCharWidth;
        *cy_window = 2 * GetSystemMetrics(SM_CYBORDER) +
                        GetSystemMetrics(SM_CYCAPTION) +
                        2 * tm.tmHeight;
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                HDC_SCREEN = CreateDCW(w!("DISPLAY"), std::ptr::null(), std::ptr::null(), std::ptr::null());
                SetTimer(window, ID_TIMER, 1000, None);
                0
            },
            WM_DISPLAYCHANGE => {
                DeleteDC(HDC_SCREEN);
                HDC_SCREEN = CreateDCW(w!("DISPLAY"), std::ptr::null(), std::ptr::null(), std::ptr::null());
                0
            },
            WM_TIMER => {
                let mut pt: POINT = std::mem::zeroed();
                GetCursorPos(&mut pt);
                CR = GetPixel(HDC_SCREEN, pt.x, pt.y);
                // println!("CursorPos x={} y={} CR={:#010X}", pt.x, pt.y, CR);
                if CR != CR_LAST {
                    CR_LAST = CR;
                    InvalidateRect(window, std::ptr::null_mut(), FALSE);
                }
                0
            },
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                let mut rc: RECT = std::mem::zeroed();
                GetClientRect(window, &mut rc);

                let s = format!("  {:02X} {:02X} {:02X}  ",
                                rvalue!(CR), gvalue!(CR), bvalue!(CR));
                println!("s={}", s);
                let wide_chars = OsStr::new(&s).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
                DrawTextW(hdc, wide_chars.as_ptr(), -1, &mut rc, DT_SINGLELINE | DT_CENTER | DT_VCENTER);

                EndPaint(window, &ps);
                0
            },
            WM_DESTROY => {
                DeleteDC(HDC_SCREEN);
                KillTimer(window, ID_TIMER);
                PostQuitMessage(0);
                0
            },
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
