#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*},
};

#[macro_use] mod macros;

static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("LineDemo");

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
            s!("Line Demonstration"),
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
            WM_SIZE => {
                CX_CLIENT = loword!(lparam);
                CY_CLIENT = hiword!(lparam);
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                Rectangle(hdc,     CX_CLIENT / 8,     CY_CLIENT / 8, 
                               7 * CX_CLIENT / 8, 7 * CY_CLIENT / 8);
                
                MoveToEx (hdc,         0,         0, std::ptr::null_mut());
                LineTo   (hdc, CX_CLIENT, CY_CLIENT);

                MoveToEx (hdc,         0, CY_CLIENT, std::ptr::null_mut());
                LineTo   (hdc, CX_CLIENT,         0);

                Ellipse  (hdc,     CX_CLIENT / 8,    CY_CLIENT / 8, 
                               7 * CX_CLIENT / 8, 7 * CY_CLIENT / 8);

                RoundRect(hdc,     CX_CLIENT / 4,     CY_CLIENT / 4, 
                               3 * CX_CLIENT / 4, 3 * CY_CLIENT / 4,
                                   CX_CLIENT / 4,     CY_CLIENT / 4);
                
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