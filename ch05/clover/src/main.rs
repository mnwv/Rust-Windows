#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*},
};

#[macro_use] mod macros;

const TWO_PI: f32 = 2.0 * 3.14159;
static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut RGN_CLIP: HRGN = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("Clover");

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
            s!("Draw a Clover"),
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
                let cursor = SetCursor(LoadCursorW(0, IDC_WAIT));
                ShowCursor(TRUE);

                if RGN_CLIP != 0 {
                    DeleteObject(RGN_CLIP);
                }
                let rgn_temp: [HRGN; 6] = [
                    CreateEllipticRgn(0,             CY_CLIENT / 3, CX_CLIENT / 2,     2 * CY_CLIENT / 3),
                    CreateEllipticRgn(CX_CLIENT / 2, CY_CLIENT / 3, CX_CLIENT,         2 * CY_CLIENT / 3),
                    CreateEllipticRgn(CX_CLIENT / 3, 0,             2 * CX_CLIENT / 3, CY_CLIENT / 2),
                    CreateEllipticRgn(CX_CLIENT / 3, CY_CLIENT / 2, 2 * CX_CLIENT / 3, CY_CLIENT),
                    CreateRectRgn(0, 0, 1, 1),
                    CreateRectRgn(0, 0, 1, 1),];
                RGN_CLIP = CreateRectRgn(0, 0, 1, 1);

                CombineRgn(rgn_temp[4], rgn_temp[0], rgn_temp[1], RGN_OR);
                CombineRgn(rgn_temp[5], rgn_temp[2], rgn_temp[3], RGN_OR);
                CombineRgn(RGN_CLIP,    rgn_temp[4], rgn_temp[5], RGN_XOR);

                for i in 0..6 {
                    DeleteObject(rgn_temp[i]);
                }
                SetCursor(cursor);
                ShowCursor(FALSE);
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                SetViewportOrgEx(hdc, CX_CLIENT / 2, CY_CLIENT / 2, std::ptr::null_mut());
                SelectClipRgn(hdc, RGN_CLIP);

                let radius = (CX_CLIENT as f32 / 2.0).hypot(CY_CLIENT as f32 / 2.0);
                let mut angle = 0.0;
                while angle < TWO_PI {
                    MoveToEx(hdc, 0, 0, std::ptr::null_mut());
                    LineTo(hdc, (radius * angle.cos() + 0.5) as i32,
                                (-radius * angle.sin() + 0.5) as i32);
                    angle += TWO_PI / 360.0;
                }

                EndPaint(window, &ps);
                0
            }

            WM_DESTROY => {
                DeleteObject(RGN_CLIP);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}