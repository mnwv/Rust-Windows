#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::{GetModuleHandleW, },
        UI::WindowsAndMessaging::*,
        UI::Controls::{SetScrollPos, SetScrollRange},
    },
};

#[macro_use] mod macros;

static mut DLG_MODELESS: HWND = HWND(std::ptr::null_mut());

fn main() -> Result<()> {
    const APP_NAME: PCWSTR = w!("Colors2");
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: LoadIconW(HINSTANCE(std::ptr::null_mut()), IDI_APPLICATION)?,
            hCursor: LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW)?,
            hbrBackground: CreateSolidBrush(COLORREF(0)),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: APP_NAME,
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            APP_NAME,
            w!("Color Scroll"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        )?;

        DLG_MODELESS = CreateDialogParamW(
                            instance,
                            w!("ColorScrDlg"),
                            hwnd,
                            Some(color_scr_dlg),
                            LPARAM(0))?;

        let mut message = MSG::default();

        while GetMessageW(&mut message, None, 0, 0).into() {
            if DLG_MODELESS.0 == std::ptr::null_mut() || IsDialogMessageW(DLG_MODELESS, &message) == FALSE {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }

        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_DESTROY => {
                println!("WM_DESTROY");
                let obj = GetStockObject(WHITE_BRUSH);
                let n = SetClassLongPtrW(window, GCL_HBRBACKGROUND, obj.0 as isize);
                let _ = DeleteObject(HGDIOBJ(n as *mut core::ffi::c_void));
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}

extern "system" fn color_scr_dlg(dlg: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> isize {
    static mut I_COLOR: [i32;3] = [0;3];
    unsafe {
        match message {
            WM_INITDIALOG => {
                for ctrl_id in 10..13 {
                    let ctrl = GetDlgItem(dlg, ctrl_id).unwrap();
                    SetScrollRange(ctrl, SB_CTL, 0, 255, FALSE).unwrap();
                    let _ = SetScrollPos(ctrl, SB_CTL, 0, FALSE);
                }
                TRUE.0 as isize
            }
            WM_VSCROLL => {
                let ctrl: HWND = HWND(lparam.0 as *mut core::ffi::c_void);
                let ctrl_id = GetWindowLongW(ctrl, GWL_ID);
                let index = ctrl_id as usize - 10;
                let parent = GetParent(dlg).unwrap();
                match SCROLLBAR_COMMAND(loword!(wparam.0) as i32) {
                    SB_PAGEDOWN => I_COLOR[index] = std::cmp::min(255, I_COLOR[index] + 15),
                    SB_LINEDOWN => I_COLOR[index] = std::cmp::min(255, I_COLOR[index] + 1),
                    SB_PAGEUP => I_COLOR[index] = std::cmp::max(0, I_COLOR[index] - 15),
                    SB_LINEUP => I_COLOR[index] = std::cmp::max(0, I_COLOR[index] - 1),
                    SB_TOP => I_COLOR[index] = 0,
                    SB_BOTTOM => I_COLOR[index] = 255,
                    SB_THUMBPOSITION | SB_THUMBTRACK => I_COLOR[index] = hiword!(wparam.0) as i32,
                    _ => return FALSE.0 as isize,
                }
                let _ = SetScrollPos(ctrl, SB_CTL, I_COLOR[index], TRUE);
                SetDlgItemInt(dlg,
                              ctrl_id + 3,
                              I_COLOR[index] as u32, FALSE).unwrap();

                let brush = CreateSolidBrush(
                                        COLORREF(rgb!(I_COLOR[0], I_COLOR[1], I_COLOR[2])));
                let n = SetClassLongPtrW(parent, GCLP_HBRBACKGROUND, brush.0 as isize);
                let _ = DeleteObject(HGDIOBJ(n as *mut core::ffi::c_void));
                let _ = InvalidateRect(parent, None, TRUE);
                TRUE.0 as isize
            }
            _ => FALSE.0 as isize
        }
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
