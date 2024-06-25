#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, Input::KeyboardAndMouse::*},
    Win32::System::Diagnostics::Debug::*,
};

#[macro_use] mod macros;

const DIVISIONS: usize = 5;
static mut CHILDREN: [[HWND; DIVISIONS]; DIVISIONS] = unsafe { std::mem::zeroed() };
const CHILD_CLASS: PCWSTR = w!("Checker4_Child");
static mut ID_FOCUS:i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("Checker4");

        let mut wnd_class = WNDCLASSW {
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

        wnd_class.lpfnWndProc = Some(child_wnd_proc);
        wnd_class.cbWndExtra = std::mem::size_of::<i32>() as i32;
        wnd_class.hIcon = 0;
        wnd_class.lpszClassName = CHILD_CLASS;

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            0,
            app_name,
            w!("Checker4 Mouse Hit-Test Demo"),
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
            WM_CREATE => {
                for x in 0..DIVISIONS {
                    for y in 0..DIVISIONS {
                        CHILDREN[x][y] = CreateWindowExW (
                            0,
                            CHILD_CLASS,
                            std::ptr::null(),
                            WS_CHILDWINDOW | WS_VISIBLE,
                            0,
                            0,
                            0,
                            0,
                            window,
                            (y << 8 | x) as isize,
                            GetWindowLongPtrW(window, GWL_HINSTANCE) as isize,
                            std::ptr::null(),
                        );
                    }
                }
                0
            },
            WM_SIZE => {
                let cx_block = loword!(lparam) / DIVISIONS as i32;
                let cy_block = hiword!(lparam) / DIVISIONS as i32;
                for x in 0..DIVISIONS {
                    for y in 0..DIVISIONS {
                        MoveWindow(CHILDREN[x][y], x as i32 * cx_block, y as i32 * cy_block, cx_block, cy_block, TRUE);
                    }
                }
                0
            },
            WM_LBUTTONDOWN => {
                MessageBeep(0);
                0
            },
            WM_SETFOCUS => {
                SetFocus(GetDlgItem(window, ID_FOCUS));
                0
            },
            WM_KEYDOWN => {
                let mut x = ID_FOCUS & 0xFF;
                let mut y = ID_FOCUS >> 8;

                match wparam as u16 {
                    VK_UP => y -= 1,
                    VK_DOWN => y += 1,
                    VK_LEFT => x -= 1,
                    VK_RIGHT => x += 1,
                    VK_HOME => { x = 0; y = 0; },
                    VK_END => { x = DIVISIONS as i32 - 1; y = DIVISIONS as i32 - 1; },
                    _ => { return 0; },
                }

                x = (x + DIVISIONS as i32) % DIVISIONS as i32;
                y = (y + DIVISIONS as i32) % DIVISIONS as i32;
                ID_FOCUS = y << 8 | x;
                SetFocus(GetDlgItem(window, ID_FOCUS));
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

extern "system" fn child_wnd_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                SetWindowLongW(hwnd, 0, 0);
                0
            },
            WM_KEYDOWN => {
                if wparam as u16 != VK_RETURN && wparam as u16 != VK_SPACE {
                    SendMessageW(GetParent(hwnd), message, wparam, lparam);
                    return 0;
                }
                SetWindowLongW(hwnd, 0, 1 ^ GetWindowLongW(hwnd, 0));
                SetFocus(hwnd);
                InvalidateRect(hwnd, std::ptr::null_mut(), FALSE);
                0
            }
            WM_LBUTTONDOWN => {
                SetWindowLongW(hwnd, 0, 1 ^ GetWindowLongW(hwnd, 0));
                SetFocus(hwnd);
                InvalidateRect(hwnd, std::ptr::null_mut(), FALSE);
                0
            },
            WM_SETFOCUS => {
                ID_FOCUS = GetWindowLongW(hwnd, GWL_ID);
                InvalidateRect(hwnd, std::ptr::null_mut(), TRUE);
                0
            },
            WM_KILLFOCUS => {
                InvalidateRect(hwnd, std::ptr::null_mut(), TRUE);
                0
            },
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);

                let mut rect: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rect);
                Rectangle(hdc, 0, 0, rect.right, rect.bottom);

                if GetWindowLongW(hwnd, 0) == 1 {
                    MoveToEx(hdc, 0, 0, std::ptr::null_mut());
                    LineTo(hdc, rect.right, rect.bottom);
                    MoveToEx(hdc, 0, rect.bottom, std::ptr::null_mut());
                    LineTo(hdc, rect.right, 0);
                }

                if hwnd == GetFocus() {
                    rect.left += rect.right / 10;
                    rect.right -= rect.left;
                    rect.top += rect.bottom / 10;
                    rect.bottom -= rect.top;

                    SelectObject(hdc, GetStockObject(NULL_BRUSH));
                    SelectObject(hdc, CreatePen(PS_DASH, 0, 0));
                    Rectangle(hdc, rect.left, rect.top, rect.right, rect.bottom);
                    DeleteObject(SelectObject(hdc, GetStockObject(BLACK_PEN)));
                }

                EndPaint(hwnd, &ps);
                0
            },
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}
