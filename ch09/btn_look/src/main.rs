#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::{WindowsAndMessaging::*, },
    Win32::Globalization::lstrlenW,
};

#[macro_use] mod macros;

struct ButtonItem {
	style: i32,
	text: *const u16,
}

const BUTTONS: [ButtonItem; 10] = [
	ButtonItem { style: BS_PUSHBUTTON,      text: w!("PUSHBUTTON"), },
    ButtonItem { style: BS_DEFPUSHBUTTON,   text: w!("DEFPUSHBUTTON"), },
    ButtonItem { style: BS_CHECKBOX,        text: w!("CHECKBOX"), },
    ButtonItem { style: BS_AUTOCHECKBOX,    text: w!("AUTOCHECKBOX"), },
    ButtonItem { style: BS_RADIOBUTTON,     text: w!("RADIOBUTTON"), },
    ButtonItem { style: BS_3STATE,          text: w!("3TATE"), },
    ButtonItem { style: BS_AUTO3STATE,      text: w!("AUTO3STATE"), },
    ButtonItem { style: BS_GROUPBOX,        text: w!("GROUPBOX"), },
    ButtonItem { style: BS_AUTORADIOBUTTON, text: w!("AUTORADIOBUTTON"), },
    ButtonItem { style: BS_OWNERDRAW,       text: w!("OWNERDRAW"), },
];

const TOP: *const u16 = w!("message         wParam        lParam");
const UND: *const u16 = w!("_______         ______        ______");

static mut HWND_BUTTONS: [HWND; 10] = unsafe { std::mem::zeroed() };
static mut RECT: RECT = unsafe { std::mem::zeroed() };
static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("BtnLook");

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
            w!("Button Look"),
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

fn to_wide_chars(str: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}

fn drawitem_command(hwnd: HWND, message: &str, wparam: WPARAM, lparam: LPARAM) {
    unsafe {
        #[allow(static_mut_refs)]
        ScrollWindow(hwnd, 0, -CY_CHAR, &RECT, &RECT);

        let hdc = GetDC(hwnd);
        SelectObject(hdc, GetStockObject(SYSTEM_FIXED_FONT));
        let s = format!("{:<16}{:04X}-{:04X} {:04X}-{:04X}",
                        message, hiword!(wparam), loword!(wparam), hiword!(lparam), loword!(lparam));
        let wide_chars = to_wide_chars(&s);
        TextOutW(hdc, 24 * CX_CHAR, CY_CHAR * (RECT.bottom / CY_CHAR - 1),
            wide_chars.as_ptr(), lstrlenW(wide_chars.as_ptr()));
        ReleaseDC(hwnd, hdc);
        #[allow(static_mut_refs)]
        ValidateRect(hwnd, &RECT);
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                CX_CHAR = loword!(GetDialogBaseUnits()) as i32;
                CY_CHAR = hiword!(GetDialogBaseUnits()) as i32;
                let createstruct : *const CREATESTRUCTW = lparam as *const CREATESTRUCTW;
                let instance = (*createstruct).hInstance;
                for i in 0..BUTTONS.len() {
                    HWND_BUTTONS[i] = CreateWindowExW(0, w!("button"),
                                        BUTTONS[i].text,
                                        WS_CHILD | WS_VISIBLE | BUTTONS[i].style as u32,
                                        CX_CHAR,
                                        CY_CHAR * (1 + 2 * i as i32),
                                        20 * CX_CHAR,
                                        7 * CY_CHAR / 4,
                                        window,
                                        i as HMENU,
                                        instance,
                                        std::ptr::null()
                                        );
                }
                0
            },
            WM_SIZE => {
                RECT.left = 24 * CX_CHAR;
                RECT.top = 2 * CY_CHAR;
                RECT.right = loword!(lparam) as i32;
                RECT.bottom = hiword!(lparam) as i32;
                0
            },
            WM_PAINT => {
                #[allow(static_mut_refs)]
                InvalidateRect(window, &RECT, TRUE);

                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                SelectObject(hdc, GetStockObject(SYSTEM_FIXED_FONT));
                SetBkMode(hdc, TRANSPARENT as i32);

                TextOutW(hdc, 24 * CX_CHAR, CY_CHAR, TOP, lstrlenW(TOP));
                TextOutW(hdc, 24 * CX_CHAR, CY_CHAR, UND, lstrlenW(UND));

                EndPaint(window, &ps);
                0
            },
            WM_DRAWITEM => {
                drawitem_command(window, "DRAWITEM", wparam, lparam);
                0
            }
            WM_COMMAND => {
                drawitem_command(window, "COMMAND", wparam, lparam);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            },
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
