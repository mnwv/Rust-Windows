#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*},
};

use rand::prelude::*;

#[macro_use] mod macros;

static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("RandRect");

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
            s!("Random Rectangles"),
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
        
        loop {
            if PeekMessageA(&mut message, 0, 0, 0, PM_REMOVE) == TRUE {
                if message.message == WM_QUIT {
                    break;
                }
                TranslateMessage(&message);
                DispatchMessageA(&message);
            } else {
                draw_rectangle(hwnd);
            }
        }
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_SIZE => {
                CX_CLIENT = loword!(lparam);
                CY_CLIENT = hiword!(lparam);
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

unsafe fn draw_rectangle(hwnd: HWND) {
    if CX_CLIENT == 0 || CY_CLIENT == 0 {
        return;
    }

    let mut rect: RECT = std::mem::zeroed();

    SetRect(&mut rect, random::<i32>() % CX_CLIENT, random::<i32>() % CY_CLIENT,
                       random::<i32>() % CX_CLIENT, random::<i32>() % CY_CLIENT);
    
    let brush = CreateSolidBrush(rgb!(random::<u32>() % 256, random::<u32>() % 256, random::<u32>() % 256));

    let hdc = GetDC(hwnd);
    FillRect(hdc, &rect, brush);
    ReleaseDC(hwnd, hdc);
    DeleteObject(brush);
}