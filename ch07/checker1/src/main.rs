#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*,},
    Win32::System::Diagnostics::Debug::*,
};

#[macro_use] mod macros;

const DIVISIONS: usize = 5;
static mut STATE: [[bool; DIVISIONS];DIVISIONS] = unsafe { std::mem::zeroed() };
static mut CX_BLOCK: i32 = 0;
static mut CY_BLOCK: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("Checker1");

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
            w!("Checker1 Mouse Hit-Test Demo"),
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

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_SIZE => {
                CX_BLOCK = loword!(lparam) / DIVISIONS as i32;
                CY_BLOCK = hiword!(lparam) / DIVISIONS as i32;
                0
            }

            WM_LBUTTONDOWN => {
                let x = loword!(lparam) / CX_BLOCK;
                let y = hiword!(lparam) / CY_BLOCK;

                if x < DIVISIONS as i32 && y < DIVISIONS as i32 {
                    STATE[y as usize][x as usize] = !STATE[y as usize][x as usize];

                    let rect: RECT = RECT {
                        left: x*CX_BLOCK,
                        top: y*CY_BLOCK,
                        right: (x + 1) * CX_BLOCK,
                        bottom: (y + 1) * CY_BLOCK,
                    };
                    InvalidateRect(window, &rect, FALSE);
                } else {
                    MessageBeep(0);
                }
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                for x in 0_i32..DIVISIONS as i32 {
                    for y in 0_i32..DIVISIONS as i32 {
                        Rectangle(hdc, x * CX_BLOCK, y * CY_BLOCK,
                                    (x + 1) * CX_BLOCK, (y + 1) * CY_BLOCK);
                        if STATE[y as usize][x as usize] {
                            MoveToEx(hdc, x * CX_BLOCK, y * CY_BLOCK, std::ptr::null_mut());
                            LineTo(hdc, (x + 1) * CX_BLOCK, (y + 1) * CY_BLOCK);
                            MoveToEx(hdc, x * CX_BLOCK, (y + 1) * CY_BLOCK, std::ptr::null_mut());
                            LineTo(hdc, (x + 1) * CX_BLOCK, y * CY_BLOCK);
                        }
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