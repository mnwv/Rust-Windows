#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::{WindowsAndMessaging::*, 
                Input::KeyboardAndMouse::*,
             },
    Win32::System::SystemServices::*,
    Win32::System::Environment::*,
};

#[macro_use] mod macros;

const ID_LIST: isize = 1;
const ID_TEXT: isize = 2;

static mut HWND_LIST: HWND = 0;
static mut HWND_TEXT: HWND = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("Environ");

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
            w!("Environment List Box"),
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

fn print_environment() {
    unsafe {
        let p_var_block: *const u16 = GetEnvironmentStringsW();
        let mut ptr = p_var_block;

        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        while *ptr != 0 {
            let len = (0..std::isize::MAX).position(|i| *ptr.offset(i) == 0).unwrap();
            let slice = std::slice::from_raw_parts(ptr, len);
            let s = OsString::from_wide(slice).to_string_lossy().into_owned();
            println!("{}", s);
            ptr = ptr.add(len + 1);                
        }
        FreeEnvironmentStringsW(p_var_block);
    }
}

fn fill_list_box(hwnd_list: HWND) {
    unsafe {
        let p_var_block: *const u16 = GetEnvironmentStringsW();
        let mut ptr = p_var_block;
        let mut wchars: Vec<u16> = Vec::<u16>::new();
        while *ptr != 0 {
            let c = if *ptr == '=' as u16 {0} else {*ptr};
            wchars.push(c); 
            ptr = ptr.add(1);
            if c == 0 {
                SendMessageW(hwnd_list, LB_ADDSTRING, 0, wchars[..].as_ptr() as LPARAM);
                wchars.clear();
                while *ptr != 0 {
                    ptr = ptr.add(1);
                }
                ptr = ptr.add(1);
            }
        }
        FreeEnvironmentStringsW(p_var_block);
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                let cx_char = loword!(GetDialogBaseUnits()) as i32;
                let cy_char = hiword!(GetDialogBaseUnits()) as i32;
                
                let instance = GetWindowLongPtrW(window, GWL_HINSTANCE) as HINSTANCE;
                HWND_LIST = CreateWindowExW(
                    0, w!("listbox"), std::ptr::null(), 
                    WS_CHILD | WS_VISIBLE | LBS_STANDARD as u32,
                    cx_char, cy_char * 3,
                    cx_char * 16 + GetSystemMetrics(SM_CXVSCROLL),
                    cy_char * 5,
                    window, ID_LIST as HMENU, instance, std::ptr::null()
                );
                HWND_TEXT = CreateWindowExW(
                    0, w!("static"), std::ptr::null(), 
                    WS_CHILD | WS_VISIBLE | SS_LEFT | WS_BORDER,
                    cx_char, cy_char,
                    GetSystemMetrics(SM_CXSCREEN), cy_char,
                    window, ID_TEXT as HMENU, instance, std::ptr::null()
                );
                fill_list_box(HWND_LIST);
                print_environment();
                0
            },
            WM_SETFOCUS => {
                SetFocus(HWND_LIST);
                0
            },
            WM_COMMAND => {
                if loword!(wparam) as isize == ID_LIST && hiword!(wparam) as u32 == LBN_SELCHANGE {
                    let index = SendMessageW(HWND_LIST, LB_GETCURSEL, 0, 0);
                    let length = SendMessageW(HWND_LIST, LB_GETTEXTLEN, index as WPARAM, 0) + 1;
                    let var_name: Vec<u16> = vec![0; length as usize];
                    SendMessageW(HWND_LIST, LB_GETTEXT, index as WPARAM, var_name.as_ptr() as LPARAM);
                    let length = GetEnvironmentVariableW(var_name.as_ptr(), std::ptr::null_mut(), 0);
                    let mut var_value: Vec<u16> = vec![0; length as usize];
                    GetEnvironmentVariableW(var_name.as_ptr(), var_value.as_mut_ptr(), length);
                    SetWindowTextW(HWND_TEXT, var_value.as_ptr());
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
