#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, },
    Win32::System::Diagnostics::Debug::*,
};

#[macro_use] mod macros;

const ID_TIMER: usize = 1;
static mut FLIP_FLOP: bool = false;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("Beeper1");

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
            s!("Beeper1 Timer Demo"),
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

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                SetTimer(window, ID_TIMER, 1000, None);
                0
            },
            WM_TIMER => {
                MessageBeep(1);
                FLIP_FLOP = !FLIP_FLOP;
                InvalidateRect(window, std::ptr::null_mut(), FALSE);
                0
            },
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                let mut rc: RECT = std::mem::zeroed();
                GetClientRect(window, &mut rc);
                let brush = CreateSolidBrush(if FLIP_FLOP { rgb!(155,0,0) } else { rgb!(0,0,255) });
                FillRect(hdc, &rc, brush);

                EndPaint(window, &ps);
                DeleteObject(brush);
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