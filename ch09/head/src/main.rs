#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::{WindowsAndMessaging::*, 
                Input::KeyboardAndMouse::*,
                Controls::*,
             },
    Win32::System::SystemServices::*,
    Win32::System::Environment::*,
    Win32::Storage::FileSystem::*,
    Win32::Globalization::*,
};

#[macro_use] mod macros;

const ID_LIST: isize = 1;
const ID_TEXT: isize = 2;
const MAX_READ: usize = 8192;
const DIR_ATTR: u32 = DDL_READWRITE | DDL_READONLY | DDL_HIDDEN | DDL_SYSTEM | 
                        DDL_DIRECTORY | DDL_ARCHIVE | DDL_DRIVES;
const DTFLAGS: u32 = DT_WORDBREAK | DT_EXPANDTABS | DT_NOCLIP | DT_NOPREFIX;

static mut VALID_FILE: bool = false;
static mut BUFFER: [u8; MAX_READ] = unsafe { std::mem::zeroed() };
static mut HWND_LIST: HWND = 0;
static mut HWND_TEXT: HWND = 0;
static mut RECT: RECT = unsafe { std::mem::zeroed() };
static mut FILE_PATH: [u16; MAX_PATH as usize] = unsafe { std::mem::zeroed() };

static mut OLD_LIST_WNDPROC: WNDPROC = None;

fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("head");

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
            w!("head"),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
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
                let cx_char = loword!(GetDialogBaseUnits()) as i32;
                let cy_char = hiword!(GetDialogBaseUnits()) as i32;

                RECT.left = 20 * cx_char;
                RECT.top  =  3 * cy_char;
                
                let instance = GetWindowLongPtrW(window, GWL_HINSTANCE) as HINSTANCE;
                HWND_LIST = CreateWindowExW(
                    0, w!("listbox"), std::ptr::null(), 
                    WS_CHILDWINDOW | WS_VISIBLE | LBS_STANDARD as u32,
                    cx_char, cy_char * 3,
                    cx_char * 13 + GetSystemMetrics(SM_CXVSCROLL),
                    cy_char * 10,
                    window, ID_LIST as HMENU, instance, std::ptr::null()
                );
                let mut path_buffer: [u16; MAX_PATH as usize + 1] = std::mem::zeroed();
                GetCurrentDirectoryW(MAX_PATH + 1, path_buffer.as_mut_ptr());

                HWND_TEXT = CreateWindowExW(
                    0, w!("static"), path_buffer.as_ptr(), 
                    WS_CHILDWINDOW | WS_VISIBLE | SS_LEFT | WS_BORDER,
                    cx_char, cy_char, cx_char * MAX_PATH as i32, cy_char,
                    window, ID_TEXT as HMENU, instance, std::ptr::null()
                );

                let wnd_proc: isize = SetWindowLongPtrW(HWND_LIST, GWL_WNDPROC, list_proc as isize);
                // https://doc.rust-lang.org/std/primitive.fn.html
                let fnptr = wnd_proc as *const ();
                let fnptr: unsafe extern "system" fn(_: isize, _: u32, _: usize, _: isize) -> isize 
                    = std::mem::transmute(fnptr);
                OLD_LIST_WNDPROC = Some(fnptr);
                
                SendMessageW(HWND_LIST, LB_DIR, DIR_ATTR as usize, w!("*.*") as LPARAM);
                0
            },
            WM_SIZE => {
                RECT.right = loword!(lparam) as i32;
                RECT.bottom = hiword!(lparam) as i32;
                0
            }
            WM_SETFOCUS => {
                SetFocus(HWND_LIST);
                0
            },
            WM_COMMAND => {
                if loword!(wparam) as isize == ID_LIST && hiword!(wparam) as u32 == LBN_DBLCLK {
                    let i: isize = SendMessageW(HWND_LIST, LB_GETCURSEL, 0, 0);
                    if LB_ERR == i as i32 {
                        return DefWindowProcW(window, message, wparam, lparam);
                    }
                    let mut path_buffer: [u16; MAX_PATH as usize + 1] = std::mem::zeroed();
                    SendMessageW(HWND_LIST, LB_GETTEXT, i as WPARAM, path_buffer.as_mut_ptr() as LPARAM);
                    let file = CreateFileW(
                                    path_buffer.as_ptr(),
                                    GENERIC_READ,
                                    FILE_SHARE_READ,
                                    std::ptr::null(),
                                    OPEN_EXISTING, 0, 0);
                    if file != INVALID_HANDLE_VALUE {
                        CloseHandle(file);
                        VALID_FILE = true;

                        lstrcpyW(FILE_PATH.as_mut_ptr(), path_buffer.as_ptr());
                        GetCurrentDirectoryW(MAX_PATH + 1, path_buffer.as_mut_ptr());

                        if path_buffer[lstrlenW(path_buffer.as_ptr()) as usize - 1] != '\\' as u16 {
                            lstrcatW(path_buffer.as_mut_ptr(), w!("\\"));
                        }

                        SetWindowTextW(HWND_TEXT, lstrcatW(path_buffer.as_mut_ptr(), FILE_PATH.as_ptr()));
                    } else {
                        VALID_FILE = false;
                        path_buffer[lstrlenW(path_buffer.as_ptr()) as usize - 1] = '\0' as u16;
                        if SetCurrentDirectoryW(&path_buffer[1]) == 0 {
                            path_buffer[3] = ':' as u16;
                            path_buffer[4] = '\0' as u16;
                            SetCurrentDirectoryW(&path_buffer[2]);
                        }

                        GetCurrentDirectoryW(MAX_PATH + 1, path_buffer.as_mut_ptr());
                        SetWindowTextW(HWND_TEXT, path_buffer.as_ptr());
                        SendMessageW(HWND_LIST, LB_RESETCONTENT, 0, 0);
                        SendMessageW(HWND_LIST, LB_DIR, DIR_ATTR as usize, w!("*.*") as LPARAM);
                    }
                    InvalidateRect(window, std::ptr::null_mut(), TRUE);
                }
                0
            },
            WM_PAINT => {
                if !VALID_FILE {
                    return DefWindowProcW(window, message, wparam, lparam);
                }
                let file = CreateFileW(
                    FILE_PATH.as_ptr(),
                    GENERIC_READ,
                    FILE_SHARE_READ,
                    std::ptr::null(),
                    OPEN_EXISTING, 0, 0);
                if file == INVALID_HANDLE_VALUE {
                    return DefWindowProcW(window, message, wparam, lparam);
                }
                let mut i: u32 = 0;
                ReadFile(file, BUFFER.as_mut_ptr(), MAX_READ as u32, &mut i, std::ptr::null_mut());
                CloseHandle(file);
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                SelectObject(hdc, GetStockObject(SYSTEM_FIXED_FONT));
                SetTextColor(hdc, GetSysColor(COLOR_BTNTEXT));
                SetBkColor(hdc, GetSysColor(COLOR_BTNFACE));
                #[allow(static_mut_refs)]
                DrawTextA(hdc, BUFFER.as_ptr(), i as i32, &mut RECT, DTFLAGS);

                EndPaint(window, &ps);
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

extern "system" fn list_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if message == WM_KEYDOWN && wparam == VK_RETURN as usize {
            SendMessageW(GetParent(hwnd), WM_COMMAND, makelong!(1, LBN_DBLCLK) as usize, hwnd as LPARAM);
        }
        CallWindowProcW(OLD_LIST_WNDPROC, hwnd, message, wparam, lparam)
    }
}
