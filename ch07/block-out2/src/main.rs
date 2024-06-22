#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, Input::KeyboardAndMouse::*},
};

#[macro_use] mod macros;

static mut PT_BEG: POINT = unsafe { std::mem::zeroed() };
static mut PT_END: POINT = unsafe { std::mem::zeroed() };
static mut PT_BOX_BEG: POINT = unsafe { std::mem::zeroed() };
static mut PT_BOX_END: POINT = unsafe { std::mem::zeroed() };
static mut BLOCKING: bool = false;
static mut VALID_BOX: bool = false;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("BlockOut2");

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
            w!("Mouse Button & Capture Demo"),
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

fn draw_box_outline(hwnd: HWND) {
    unsafe {
        let hdc = GetDC(hwnd);

        SetROP2(hdc, R2_NOT);
        SelectObject(hdc, GetStockObject(NULL_BRUSH));
        Rectangle(hdc, PT_BEG.x, PT_BEG.y, PT_END.x, PT_END.y);

        ReleaseDC(hwnd, hdc);
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_LBUTTONDOWN => {
                let x = loword!(lparam);
                let y = hiword!(lparam);
                PT_BEG.x = x;
                PT_END.x = x;
                PT_BEG.y = y;
                PT_END.y = y;
                draw_box_outline(window);

                SetCapture(window);
                SetCursor(LoadCursorW(0, IDC_CROSS));
                BLOCKING = true;
                0
            },
            WM_MOUSEMOVE => {
                if BLOCKING {
                    SetCursor(LoadCursorW(0, IDC_CROSS));

                    draw_box_outline(window);

                    PT_END.x = (loword!(lparam) as i16) as i32;
                    PT_END.y = (hiword!(lparam) as i16) as i32;

                    draw_box_outline(window);
                }
                0
            },
            WM_LBUTTONUP => {
                if BLOCKING {
                    draw_box_outline(window);

                    PT_BOX_BEG.x = PT_BEG.x;
                    PT_BOX_BEG.y = PT_BEG.y;
                    PT_BOX_END.x = (loword!(lparam) as i16) as i32;
                    PT_BOX_END.y = (hiword!(lparam) as i16) as i32;

                    ReleaseCapture();
                    SetCursor(LoadCursorW(0, IDC_ARROW));

                    BLOCKING = false;
                    VALID_BOX = true;

                    InvalidateRect(window, std::ptr::null_mut(), TRUE);
                }
                0
            },
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                if VALID_BOX {
                    SelectObject(hdc, GetStockObject(BLACK_BRUSH));
                    Rectangle(hdc, PT_BOX_BEG.x, PT_BOX_BEG.y, PT_BOX_END.x, PT_BOX_END.y);
                }

                if BLOCKING {
                    SetROP2(hdc, R2_NOT);
                    SelectObject(hdc, GetStockObject(NULL_BRUSH));
                    Rectangle(hdc, PT_BEG.x, PT_BEG.y, PT_END.x, PT_END.y);
                }

                EndPaint(window, &ps);
                0
            },
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            },
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
