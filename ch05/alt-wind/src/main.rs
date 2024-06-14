#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*},
};

#[macro_use] mod macros;

const APT_FIGURE: [POINT; 10] = [POINT {x:10, y:70,}, POINT {x:50, y:70}, POINT {x:50, y:10},
                                  POINT {x:90, y:10}, POINT {x:90, y:50}, POINT {x:30, y:50},
                                  POINT {x:30, y:90}, POINT {x:70, y:90}, POINT {x:70, y:30},
                                  POINT {x:10, y:30},];

static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("AltWind");

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
            s!("ALternate and Winding Fill Mode"),
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
                let mut apt: [POINT; 10] = std::mem::zeroed(); 
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                SelectObject(hdc, GetStockObject(GRAY_BRUSH));

                for i in 0..10 {
                    apt[i].x = CX_CLIENT * APT_FIGURE[i].x / 200;
                    apt[i].y = CY_CLIENT * APT_FIGURE[i].y / 100;
                }

                SetPolyFillMode(hdc, ALTERNATE);
                Polygon(hdc, &apt[0], 10);

                for i in 0..10 {
                    apt[i].x += CX_CLIENT / 2;
                }

                SetPolyFillMode(hdc, WINDING);
                Polygon(hdc, &apt[0], 10);

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