#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{
            LibraryLoader::{GetModuleHandleW, },
            Threading::Sleep,
            Diagnostics::Debug::MessageBeep,
            SystemServices::MAXDWORD,
        },
        UI::{
            WindowsAndMessaging::*,
            Input::KeyboardAndMouse::*,
        },
    },
};

#[macro_use] mod macros;

fn main() -> Result<()> {
    const APP_NAME: PCWSTR = w!("HexCalc");
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: DLGWINDOWEXTRA as i32,
            hInstance: instance.into(),
            hIcon: LoadIconW(instance, APP_NAME)?,
            hCursor: LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW)?,
            hbrBackground: GetSysColorBrush(SYS_COLOR_INDEX(COLOR_BTNFACE.0)), //HBRUSH(COLOR_BTNFACE);
            lpszMenuName: PCWSTR::null(),
            lpszClassName: APP_NAME,
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        let result = CreateDialogParamW(
                                    instance,
                                    APP_NAME,
                                    HWND(std::ptr::null_mut()),
                                    None,
                                    LPARAM(0));
        if let Some(err) = result.as_ref().err() {
            println!("CreateDialogParamW() faild. {:?}", err);
            return Ok(());
        }
        let hwnd = result.ok().unwrap();
        let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
        let mut message = MSG::default();

        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    static mut NEW_NUMBER: bool = true;
    static mut OPERATION: char = '=';
    static mut NUMBER: u32 = 0;
    static mut FIRST_NUM: u32 = 0;

    unsafe {
        match message {
            WM_KEYDOWN | WM_CHAR | WM_COMMAND => {
                // 256  // 258      // 273
                let mut wparam2 = wparam.clone();
                if message == WM_KEYDOWN {
                    if wparam2.0 != VK_LEFT.0 as usize {
                        return DefWindowProcW(window, message, wparam, lparam);
                    }
                    wparam2.0 = VK_BACK.0 as usize;
                }
                if message == WM_KEYDOWN || message == WM_CHAR {
                    let mut code:u8 = wparam2.0 as u8;
                    if wparam2.0 == VK_RETURN.0 as usize {
                        code = '=' as u8;
                        wparam2.0 = code as usize;
                    }
                    code = code.to_ascii_uppercase();
                    wparam2.0 = code as usize;
                    let result = GetDlgItem(window, wparam2.0 as i32);
                    if let Some(err) = result.as_ref().err() {
                        println!("GetDlgItem() failed。Error={:?}", err);
                        println!("wparam={:?}", wparam);
                        MessageBeep(MB_OK).unwrap();
                        return DefWindowProcW(window, message, wparam, lparam);
                    }
                    let button = result.ok().unwrap();
                    let _ = SendMessageW(button, BM_SETSTATE, WPARAM(1), LPARAM(0));
                    Sleep(100);
                    let _ = SendMessageW(button, BM_SETSTATE, WPARAM(0), LPARAM(0));
                }
                let _ = SetFocus(window);
                let kcode: u8 = wparam2.0 as u8;

                println!("msg={:?} kcode={:#04X} wparam.0={:#010X}", message, kcode, wparam.0);
                if kcode == VK_BACK.0 as u8 {
                    NUMBER /= 16;
                    show_number(window, NUMBER);
                } else if kcode == VK_ESCAPE.0 as u8 {
                    NUMBER = 0;
                    show_number(window, NUMBER);
                } else if (kcode as char).is_ascii_hexdigit() || (kcode as char).is_ascii_digit() {
                    println!("digit? kcode={:#04X}", kcode);
                    if NEW_NUMBER {
                        FIRST_NUM = NUMBER;
                        NUMBER = 0;
                    }
                    NEW_NUMBER = false;

                    if NUMBER <= MAXDWORD >> 4 {    // MAXDWORD = FFFF_FFFF
                        NUMBER = 16 * NUMBER + kcode as u32 -
                            if (kcode as char).is_ascii_digit() { '0' as u8 as u32 } else { ('A' as u8 - 10) as u32 };
                        show_number(window, NUMBER);
                    } else {
                        MessageBeep(MB_OK).unwrap();
                    }
                } else {
                    println!("operation? kcode={:#04X}", kcode);
                    if !NEW_NUMBER {
                        NUMBER = calc_it(FIRST_NUM, OPERATION, NUMBER);
                        show_number(window, NUMBER);
                        NEW_NUMBER = true;
                        OPERATION = kcode as char;
                    }
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}

fn show_number(hwnd: HWND, number: u32) {
    let s = format!("{:X}", number);
    let v = to_wide_chars(&s);
    unsafe {
        SetDlgItemTextW(hwnd, VK_ESCAPE.0 as i32, PCWSTR::from_raw(v.as_ptr())).unwrap();
    }
}

fn calc_it(first_num:u32, operation: char, num: u32) -> u32 {
    match operation {
        '=' => num,
        '+' => first_num.wrapping_add(num),
        '-' => first_num.wrapping_sub(num),
        '*' => first_num.wrapping_mul(num),
        '&' => first_num & num,
        '|' => first_num | num,
        '^' => first_num ^ num,
        '<' => first_num << num,
        '>' => first_num >> num,
        '/' => if num != 0 { first_num / num } else { MAXDWORD },
        '%' => if num != 0 { first_num % num } else { MAXDWORD },
        _ => 0,
    }
}

#[allow(dead_code)]
fn type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

#[allow(dead_code)]
fn to_wide_chars(str: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>()
}

#[allow(dead_code)]
fn from_wide_ptr(ptr: *const u16) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    unsafe {
        let len = (0..std::isize::MAX).position(|i| *ptr.offset(i) == 0).unwrap();
        let slice = std::slice::from_raw_parts(ptr, len);
        OsString::from_wide(slice).to_string_lossy().into_owned()
    }
}

#[allow(dead_code)]
fn get_err_msg(err_code: i32) -> String {
    use windows::Win32::{
        System::Diagnostics::Debug::{
            FormatMessageW, FORMAT_MESSAGE_ALLOCATE_BUFFER,
            FORMAT_MESSAGE_FROM_SYSTEM,
        },
        Foundation::{LocalFree, HLOCAL},
    };

    unsafe {
        let mut text: *mut u16 = std::ptr::null_mut();
        let n = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            None,
            err_code as u32,
            0,
            PWSTR(&mut text as *mut _ as *mut _),
            2048,
            None);
        if n > 0 {
            let parts = std::slice::from_raw_parts(text, n as usize);
            let s = String::from_utf16(parts).unwrap();
            LocalFree(HLOCAL(text as *mut core::ffi::c_void));
            return s;
        }
        "Failed:FormatMessageW()".to_string()
    }
}
