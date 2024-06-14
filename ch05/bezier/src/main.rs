#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*},
    Win32::System::SystemServices::*,
};

#[macro_use] mod macros;

static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut APT: [POINT; 4] = unsafe { std::mem::zeroed() };

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("Bezier");

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
            s!("Bezier Splines"),
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

fn draw_bezier(hdc: HDC) {
    unsafe {
        PolyBezier(hdc, &APT[0], 4);

        MoveToEx(hdc, APT[0].x, APT[0].y, std::ptr::null_mut());
        LineTo  (hdc, APT[1].x, APT[1].y);
        
        MoveToEx(hdc, APT[2].x, APT[2].y, std::ptr::null_mut());
        LineTo  (hdc, APT[3].x, APT[3].y);
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_SIZE => {
                CX_CLIENT = loword!(lparam);
                CY_CLIENT = hiword!(lparam);

                APT[0].x = CX_CLIENT / 4;
                APT[0].y = CY_CLIENT / 2;

                APT[1].x = CX_CLIENT / 2;
                APT[1].y = CY_CLIENT / 4;

                APT[2].x =     CX_CLIENT / 2;
                APT[2].y = 3 * CY_CLIENT / 4;

                APT[3].x = 3 * CX_CLIENT / 4;
                APT[3].y =     CY_CLIENT / 2;
                0
            }

            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MOUSEMOVE => {
                if (wparam as u32 & MK_LBUTTON == MK_LBUTTON) || (wparam as u32 & MK_RBUTTON == MK_RBUTTON) {
                    let hdc = GetDC(window);

                    SelectObject(hdc, GetStockObject(WHITE_PEN));
                    draw_bezier(hdc);

                    if wparam as u32 & MK_LBUTTON == MK_LBUTTON {
                        APT[1].x = loword!(lparam);
                        APT[1].y = hiword!(lparam);
                    }

                    if wparam as u32 & MK_RBUTTON == MK_RBUTTON {
                        APT[2].x = loword!(lparam);
                        APT[2].y = hiword!(lparam);
                    }

                    SelectObject(hdc, GetStockObject(BLACK_PEN));
                    draw_bezier(hdc);

                    ReleaseDC(window, hdc);
                }
                0
            }

            WM_PAINT => {
                InvalidateRect(window, std::ptr::null(), 0);

                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                draw_bezier(hdc);

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