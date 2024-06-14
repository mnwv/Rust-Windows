#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*},
    Win32::Globalization::lstrlenA,
};

#[macro_use] mod macros;

const HEADING: *const u8 =  s!("Mapping Mode           Left  Right    Top Bottom");
const UND_LINE: *const u8 = s!("------------           ----  -----    --- ------");
static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("WhatSize");

        let wnd_class = WNDCLASSA {
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

        let atom = RegisterClassA(&wnd_class);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExA(
            0,
            app_name,
            s!("What Size is the Window?"),
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

        while GetMessageA(&mut message, 0, 0, 0) != 0 {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    }
}

unsafe fn show(window: HWND, hdc: HDC, x_text: i32, y_text: i32, map_mode: i32, map_mode_str: &str) {
    SaveDC(hdc);

    SetMapMode(hdc, map_mode);
    let mut rect: RECT = std::mem::zeroed();
    GetClientRect(window, &mut rect);
    let mut pt: [POINT;2] = [
        POINT{x:rect.left, y:rect.top},
        POINT{x:rect.right, y:rect.bottom},
    ]; 
    DPtoLP(hdc, &mut pt[0], 2);

    RestoreDC(hdc, -1);

    let s = format!("{:<20}{:>7}{:>7}{:>7}{:>7}{}",
                    map_mode_str, pt[0].x, pt[1].x, pt[0].y, pt[1].y, '\0');
    TextOutA(hdc, x_text, y_text, s.as_ptr(), lstrlenA(s.as_ptr()));
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                let hdc = GetDC(window);
                SelectObject(hdc, GetStockObject(SYSTEM_FIXED_FONT));

                let mut tm: TEXTMETRICA = std::mem::zeroed();
                GetTextMetricsA(hdc, &mut tm);
                CX_CHAR = tm.tmAveCharWidth;
                CY_CHAR = tm.tmHeight + tm.tmExternalLeading;

                ReleaseDC(window, hdc);
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                SelectObject(hdc, GetStockObject(SYSTEM_FIXED_FONT));

                SetMapMode(hdc, MM_ANISOTROPIC);
                SetWindowExtEx(hdc, 1, 1, std::ptr::null_mut());
                SetViewportExtEx(hdc, CX_CHAR, CY_CHAR, std::ptr::null_mut());
                TextOutA(hdc, 1, 1, HEADING, lstrlenA(HEADING));
                TextOutA(hdc, 1, 2, UND_LINE, lstrlenA(UND_LINE));

                show(window, hdc, 1, 3, MM_TEXT, "TEXT (pixels)");
                show(window, hdc, 1, 4, MM_LOMETRIC, "LOMETRIC (.1 mm)");
                show(window, hdc, 1, 5, MM_HIMETRIC, "HIMETRIC (.01 mm)");
                show(window, hdc, 1, 6, MM_LOENGLISH, "LOENGLISH (.01 in)");
                show(window, hdc, 1, 7, MM_HIENGLISH, "HIENGLISH (.001 in)");
                show(window, hdc, 1, 8, MM_TWIPS, "TWIPS (1/1440 in)");

                EndPaint(window, &ps);
                0
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}