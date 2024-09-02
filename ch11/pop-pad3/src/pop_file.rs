use std::io::{Read, Write};
use std::ptr::{addr_of_mut};
use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{BOOL, FALSE, HINSTANCE, HWND, LPARAM, MAX_PATH, TRUE};
use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, GetSaveFileNameW, OFN_CREATEPROMPT, OFN_HIDEREADONLY, OFN_OVERWRITEPROMPT, OPENFILENAMEW, OPEN_FILENAME_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW, SetWindowTextW};
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
    let ret = GetOpenFileNameW(addr_of_mut!(OFN));
    if ret == TRUE {
        *filename = util::from_wide_ptr(buf_file_name.as_ptr());
        *title_name = util::from_wide_ptr(buf_title_name.as_ptr());
    }
    ret
}

pub unsafe fn pop_file_save_dlg(hwnd: HWND, filename: &mut String, title_name: &mut String) -> BOOL {
    let buf_file_name: [u16;MAX_PATH as usize] = [0;MAX_PATH as usize];
    let buf_title_name: [u16;MAX_PATH as usize] = [0;MAX_PATH as usize];
    OFN.hwndOwner = hwnd;
    OFN.lpstrFile = PWSTR::from_raw(buf_file_name.as_ptr() as *mut u16);
    OFN.lpstrFileTitle = PWSTR::from_raw(buf_title_name.as_ptr() as *mut u16);
    OFN.Flags = OFN_OVERWRITEPROMPT;
    let ret = GetSaveFileNameW(addr_of_mut!(OFN));
    if ret == TRUE {
        *filename = util::from_wide_ptr(buf_file_name.as_ptr());
        *title_name = util::from_wide_ptr(buf_title_name.as_ptr());
    }
    ret
}

pub fn pop_file_read(wnd_edit: HWND, file_name: &String) -> BOOL {
    let result = std::fs::File::open(file_name);
    if let Some(err) = result.as_ref().err() {
        println!("open() failed. Error:{}", err.to_string());
        return FALSE;
    }
    let mut file = result.ok().unwrap();
    let mut contents = String::new();
    let result = file.read_to_string(&mut contents);
    if let Some(err) = result.as_ref().err() {
        println!("read_to_string() failed. Error:{}", err.to_string());
        return FALSE;
    }
    let vu16 = util::to_wide_chars(&contents);
    unsafe {
        SetWindowTextW(wnd_edit, PCWSTR::from_raw(vu16.as_ptr())).unwrap();
        TRUE
    }
}

pub fn pop_file_write(wnd_edit: HWND, file_name: &String) -> BOOL {
    let result = std::fs::File::create(file_name);
    if let Some(err) = result.as_ref().err() {
        println!("create() failed. Error:{}", err.to_string());
        return FALSE;
    }
    let len = unsafe { GetWindowTextLengthW(wnd_edit) };
    let mut v: Vec<u16> = vec!(0; len as usize);
    unsafe {
        GetWindowTextW(wnd_edit, &mut v);
    }
    let s = util::from_wide_ptr(v.as_ptr());
    let mut file = result.ok().unwrap();
    let result = file.write_all((&s).as_ref());
    if let Some(err) = result.as_ref().err() {
        println!("write_all() failed. Error:{}", err.to_string());
        return FALSE;
    }
    TRUE
}