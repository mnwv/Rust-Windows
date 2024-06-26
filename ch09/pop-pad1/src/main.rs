#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::{WindowsAndMessaging::*, 
                Input::KeyboardAndMouse::*,
             },
};

#[macro_use] mod macros;

const ID_EDIT: isize = 1;
const APP_NAME: *const u16 = w!("PopPad1");
static mut HWND_EDIT: HWND = 0;


fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(instance != 0);

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
            lpszClassName: APP_NAME,
        };

        let atom = RegisterClassW(&wnd_class);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExW(
            0,
            APP_NAME,
            APP_NAME,
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
                let createstruct : *const CREATESTRUCTW = lparam as *const CREATESTRUCTW;
                let instance = (*createstruct).hInstance;
                HWND_EDIT = CreateWindowExW(
                    0, w!("edit"), std::ptr::null(), 
                    WS_CHILD | WS_VISIBLE | WS_HSCROLL | WS_VSCROLL | WS_BORDER |
                    (ES_LEFT | ES_MULTILINE | ES_AUTOHSCROLL | ES_AUTOVSCROLL) as u32,
                    0, 0, 0, 0,
                    window, ID_EDIT as HMENU, instance, std::ptr::null()
                );
                0
            },
            WM_SETFOCUS => {
                SetFocus(HWND_EDIT);
                0
            },
            WM_SIZE => {
                MoveWindow(HWND_EDIT, 0, 0, loword!(lparam) as i32, hiword!(lparam) as i32, TRUE);
                0
            },
            WM_COMMAND => {
                if loword!(wparam) == ID_EDIT as i16 {
                    if hiword!(wparam) == EN_ERRSPACE as i16 ||
                        hiword!(wparam) == EN_MAXTEXT as i16 {
                            MessageBoxW(window, w!("Edit control out of space."), 
                                        APP_NAME, MB_OK | MB_ICONSTOP);
                        }
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
