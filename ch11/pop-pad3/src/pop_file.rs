use std::ptr::addr_of;
use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, LPARAM, MAX_PATH, TRUE};
use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, OFN_CREATEPROMPT, OFN_HIDEREADONLY, OPENFILENAMEW, OPEN_FILENAME_FLAGS};
use crate::util;

static mut OFN: OPENFILENAMEW = unsafe { std::mem::zeroed() };

pub unsafe fn pop_file_initialize(hwnd: HWND) {
    OFN.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    OFN.hwndOwner = hwnd;
    OFN.hInstance = HINSTANCE(std::ptr::null_mut());
    OFN.lpstrFilter
        = w!("Text Files (*.TXT)\0*.txt\0Ascii Files (*.ASC)\0*.asc\0All Files (*.*)\0*.*\0\0");
    OFN.lpstrCustomFilter = PWSTR::null();
    OFN.nMaxCustFilter = 0;
    OFN.nFilterIndex = 0;
    OFN.lpstrFile = PWSTR::null();
    OFN.nMaxFile = MAX_PATH;
    OFN.lpstrFileTitle = PWSTR::null();
    OFN.nMaxFileTitle = MAX_PATH;
    OFN.lpstrInitialDir = PCWSTR::null();
    OFN.lpstrTitle = PCWSTR::null();
    OFN.Flags = OPEN_FILENAME_FLAGS(0);
    OFN.nFileOffset = 0;
    OFN.nFileExtension = 0;
    OFN.lpstrDefExt = w!("txt");
    OFN.lCustData = LPARAM(0);
    OFN.lpfnHook = None;
    OFN.lpTemplateName = PCWSTR::null();
}

pub unsafe fn pop_file_open_dialog(hwnd: HWND, filename: &mut String, title_name: &mut String) -> BOOL{
    let buf_file_name: [u16;MAX_PATH as usize] = [0;MAX_PATH as usize];
    let buf_title_name: [u16;MAX_PATH as usize] = [0;MAX_PATH as usize];
    OFN.hwndOwner = hwnd;
    OFN.lpstrFile = PWSTR::from_raw(buf_file_name.as_ptr() as *mut u16);
    OFN.lpstrFileTitle = PWSTR::from_raw(buf_title_name.as_ptr() as *mut u16);
    OFN.Flags = OFN_HIDEREADONLY | OFN_CREATEPROMPT;
    let ret = GetOpenFileNameW(addr_of!(OFN) as *mut OPENFILENAMEW);
    if ret == TRUE {
        *filename = util::from_wide_ptr(buf_file_name.as_ptr());
        *title_name = util::from_wide_ptr(buf_title_name.as_ptr());
    }
    ret
}