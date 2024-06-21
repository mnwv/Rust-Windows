#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*,},
    Win32::System::SystemServices::*,
};

#[macro_use] mod macros;

const MAXPOINTS: usize = 1000;
static mut PT: [POINT; MAXPOINTS] = unsafe { std::mem::zeroed() };
static mut COUNT: usize = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("Connect");

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
            w!("Connect-the-Points Mouse Demo"),
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
            WM_LBUTTONDOWN => {
                COUNT = 0;
                InvalidateRect(window, std::ptr::null(), TRUE);
                0
            }

            WM_MOUSEMOVE => {
                if wparam as u32 & MK_LBUTTON == MK_LBUTTON && COUNT < 1000 {
                    PT[COUNT].x = loword!(lparam);
                    PT[COUNT].y = hiword!(lparam);
                    COUNT += 1;
                    let hdc = GetDC(window);
                    SetPixel(hdc, loword!(lparam), hiword!(lparam), 0);
                    ReleaseDC(window, hdc);
                }
                0
            }

            WM_LBUTTONUP => {
                InvalidateRect(window, std::ptr::null(), FALSE);
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                SetCursor(LoadCursorW(0, IDC_WAIT));
                ShowCursor(TRUE);

                if COUNT > 0 {
                    for i in 0..COUNT - 1 {
                        for j in i + 1..COUNT {
                            MoveToEx(hdc, PT[i].x, PT[i].y, std::ptr::null_mut());
                            LineTo(hdc, PT[j].x, PT[j].y);
                        }
                    }
                }
                ShowCursor(FALSE);
                SetCursor(LoadCursorW(0, IDC_ARROW));

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