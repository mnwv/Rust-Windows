#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::{WindowsAndMessaging::*, Controls::*, },
};

#[macro_use] mod macros;

const ID_SMALLER: isize = 1;
const ID_LARGER: isize = 2;
static mut INSTANCE: HINSTANCE = 0;
static mut HWND_SMALLER: HWND = 0;
static mut HWND_LARGER: HWND = 0;
static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;
static mut BTN_WIDTTH: i32 = 0;
static mut BTN_HEIGHT: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(instance != 0);
        INSTANCE = instance;

        let app_name = w!("OwnDraw");

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
            w!("Owner-Draw Button Demo"),
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

fn triangle (hdc: HDC, &points: &[POINT;3]) {
    unsafe {
        SelectObject(hdc, GetStockObject(BLACK_BRUSH));
        Polygon(hdc, &points[0], 3);
        SelectObject(hdc, GetStockObject(WHITE_BRUSH));
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                CX_CHAR = loword!(GetDialogBaseUnits()) as i32;
                CY_CHAR = hiword!(GetDialogBaseUnits()) as i32;
                BTN_WIDTTH = 8 * CX_CHAR;
                BTN_HEIGHT = 4 * CY_CHAR;
                HWND_SMALLER = CreateWindowExW(0, w!("button"), w!(""), 
                                        WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                                        0,
                                        0,
                                        BTN_WIDTTH,
                                        BTN_HEIGHT,
                                        window,
                                        ID_SMALLER,
                                        INSTANCE,
                                        std::ptr::null()
                                    );
                HWND_LARGER = CreateWindowExW(0, w!("button"), w!(""), 
                                        WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32,
                                        0,
                                        0,
                                        BTN_WIDTTH,
                                        BTN_HEIGHT,
                                        window,
                                        ID_LARGER,
                                        INSTANCE,
                                        std::ptr::null()
                                    );
                0
            },
            WM_SIZE => {
                CX_CLIENT = loword!(lparam) as i32;
                CY_CLIENT = hiword!(lparam) as i32;

                MoveWindow(HWND_SMALLER, CX_CLIENT / 2 - 3 * BTN_WIDTTH / 2,
                                        CY_CLIENT / 2 - BTN_HEIGHT / 2,
                                    BTN_WIDTTH, BTN_HEIGHT, TRUE);
                MoveWindow(HWND_LARGER, CX_CLIENT / 2 + BTN_WIDTTH / 2,
                                        CY_CLIENT / 2 - BTN_HEIGHT / 2,
                                    BTN_WIDTTH, BTN_HEIGHT, TRUE);
                0
            },
            WM_COMMAND => {
                let mut rc: RECT = std::mem::zeroed();
                GetWindowRect(window, &mut rc);

                match wparam as isize {
                    ID_SMALLER => {
                        rc.left += CX_CLIENT / 20;
                        rc.right -= CX_CLIENT / 20;
                        rc.top += CY_CLIENT / 20;
                        rc.bottom -= CY_CLIENT / 20;
                    },
                    ID_LARGER => {
                        rc.left -= CX_CLIENT / 20;
                        rc.right += CX_CLIENT / 20;
                        rc.top -= CY_CLIENT / 20;
                        rc.bottom += CY_CLIENT / 20;
                    },
                    _ => {}
                }
                MoveWindow(window, rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top, TRUE);
                0
            },
            WM_DRAWITEM => {
                let dis : *mut DRAWITEMSTRUCT = lparam as *mut DRAWITEMSTRUCT;
                FillRect((*dis).hDC, &(*dis).rcItem, GetStockObject(WHITE_BRUSH) as HBRUSH);
                FrameRect((*dis).hDC, &(*dis).rcItem, GetStockObject(BLACK_BRUSH) as HBRUSH);

                let cx = (*dis).rcItem.right - (*dis).rcItem.left;
                let cy = (*dis).rcItem.bottom - (*dis).rcItem.top;

                let mut points: [POINT;3] = std::mem::zeroed();
                match (*dis).CtlID as isize {
                    ID_SMALLER => {
                        points[0].x = 3 * cx / 8;   points[0].y = 1 * cy / 8;
                        points[1].x = 5 * cx / 8;   points[1].y = 1 * cy / 8;
                        points[2].x = 4 * cx / 8;   points[2].y = 3 * cy / 8;
                        triangle((*dis).hDC, &points);

                        points[0].x =  7* cx / 8;   points[0].y = 3 * cy / 8;
                        points[1].x =  7* cx / 8;   points[1].y = 5 * cy / 8;
                        points[2].x =  5* cx / 8;   points[2].y = 4 * cy / 8;

                        triangle((*dis).hDC, &points);
                        points[0].x =  5* cx / 8;   points[0].y = 7 * cy / 8;
                        points[1].x =  3* cx / 8;   points[1].y = 7 * cy / 8;
                        points[2].x =  4* cx / 8;   points[2].y = 5 * cy / 8;

                        triangle((*dis).hDC, &points);
                        points[0].x =  1* cx / 8;   points[0].y = 5 * cy / 8;
                        points[1].x =  1* cx / 8;   points[1].y = 3 * cy / 8;
                        points[2].x =  3* cx / 8;   points[2].y = 4 * cy / 8;
                        triangle((*dis).hDC, &points);
                    },
                    ID_LARGER => {
                        points[0].x = 5 * cx / 8;   points[0].y = 3 * cy / 8;
                        points[1].x = 3 * cx / 8;   points[1].y = 3 * cy / 8;
                        points[2].x = 4 * cx / 8;   points[2].y = 1 * cy / 8;
                        triangle((*dis).hDC, &points);

                        points[0].x = 5 * cx / 8;   points[0].y = 5 * cy / 8;
                        points[1].x = 5 * cx / 8;   points[1].y = 3 * cy / 8;
                        points[2].x = 7 * cx / 8;   points[2].y = 4 * cy / 8;
                        triangle((*dis).hDC, &points);

                        points[0].x = 3 * cx / 8;   points[0].y = 5 * cy / 8;
                        points[1].x = 5 * cx / 8;   points[1].y = 5 * cy / 8;
                        points[2].x = 4 * cx / 8;   points[2].y = 7 * cy / 8;
                        triangle((*dis).hDC, &points);

                        points[0].x = 3 * cx / 8;   points[0].y = 3 * cy / 8;
                        points[1].x = 3 * cx / 8;   points[1].y = 5 * cy / 8;
                        points[2].x = 1 * cx / 8;   points[2].y = 4 * cy / 8;
                        triangle((*dis).hDC, &points);
                    }
                    _ => {},
                }
                if (*dis).itemState & ODS_SELECTED == ODS_SELECTED {
                    InvertRect((*dis).hDC, &(*dis).rcItem);
                }
                if (*dis).itemState & ODS_FOCUS == ODS_FOCUS {
                    (*dis).rcItem.left += cx / 16;
                    (*dis).rcItem.top += cy /16;
                    (*dis).rcItem.right -= cx / 16;
                    (*dis).rcItem.bottom -= cy/ 16;
                    DrawFocusRect((*dis).hDC, &(*dis).rcItem);
                }
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
