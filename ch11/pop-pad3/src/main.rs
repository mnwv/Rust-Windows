#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ptr::{addr_of};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{
            LibraryLoader::{GetModuleHandleW, },
            DataExchange::IsClipboardFormatAvailable,
            Ole::CF_TEXT,
        },
        UI::{
            WindowsAndMessaging::*,
            Input::KeyboardAndMouse::*,
            Controls::Dialogs::{FINDMSGSTRINGW, FINDREPLACEW, FR_DIALOGTERM, FR_FINDNEXT, FR_REPLACE, FR_REPLACEALL},
            Controls::{EM_CANUNDO, EM_GETSEL, EM_LIMITTEXT, EM_SETSEL},
        },
    },
};
use crate::pop_file::*;

#[macro_use] mod macros;
mod pop_file;
mod util;

const EDIT_ID: i32 = 1;
static UNTITLED: &str = "untitled";

static mut DLG_MODELESS: HWND = HWND(std::ptr::null_mut());
const APP_NAME: PCWSTR = w!("PopPad");

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: LoadIconW(instance, APP_NAME).unwrap(),
            hCursor: LoadCursorW(HINSTANCE(std::ptr::null_mut()), IDC_ARROW).unwrap(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszMenuName: APP_NAME,
            lpszClassName: APP_NAME,
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            APP_NAME,
            PCWSTR::null(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        )?;

        let accel = LoadAcceleratorsW(instance, APP_NAME).unwrap();

        let mut message = MSG::default();

        while GetMessageW(&mut message, None, 0, 0).into() {
            if DLG_MODELESS == HWND(std::ptr::null_mut()) || IsDialogMessageW(DLG_MODELESS, &message) == FALSE {
                if TranslateAcceleratorW(hwnd, accel, &message) == 0 {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    static mut NEED_SAVE: bool = false;
    static mut INSTANCE: HINSTANCE = HINSTANCE(std::ptr::null_mut());
    static mut WND_EDIT: HWND = HWND(std::ptr::null_mut());
    static mut OFFSET: i32 = 0;
    static mut FILE_NAME: String = String::new();
    static mut TITLE_NAME: String = String::new();
    static mut MSG_FIND_REP: u32 = 0;
    static mut EDIT_BK_BRUSH: HBRUSH = HBRUSH(std::ptr::null_mut());
    unsafe {
        match message {
            WM_CREATE => {
                println!("WM_CREATE");
                let p : *const CREATESTRUCTW = lparam.0 as *const CREATESTRUCTW;
                INSTANCE = (*p).hInstance;
                // Create the edit control child window
                EDIT_BK_BRUSH = HBRUSH(GetStockObject(BLACK_BRUSH).0);
                WND_EDIT = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("edit"),
                    PCWSTR::null(),
                    WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER |
                        WINDOW_STYLE(
                            (ES_LEFT | ES_MULTILINE | ES_NOHIDESEL | ES_AUTOHSCROLL |
                            ES_AUTOVSCROLL) as u32),
                    0, 0, 0, 0,
                    window,
                    // LoadMenuW(INSTANCE, PCWSTR::from_raw(EDIT_ID as *const u16)).unwrap(),
                    None,
                    INSTANCE,
                    None,
                ).unwrap();
                let _ = SendMessageW(WND_EDIT, EM_LIMITTEXT, WPARAM(3200), LPARAM(0));

                // Initialize common dialog box stuff
                pop_file_initialize(window);
                // PopFontInitialize (hwndEdit) ;

                MSG_FIND_REP = RegisterWindowMessageW (FINDMSGSTRINGW);

                #[allow(static_mut_refs)]
                do_caption(window, &TITLE_NAME);
                LRESULT(0)
            }
            WM_SETFOCUS => {
                let _ = SetFocus(WND_EDIT);
                LRESULT(0)
            }
            WM_SIZE => {
                MoveWindow(
                    WND_EDIT, 0, 0,
                    loword!(lparam.0) as i32,
                    hiword!(lparam.0) as i32,
                    TRUE).unwrap();
                LRESULT(0)
            }
            WM_INITMENUPOPUP => {
                let menu: HMENU = HMENU(lparam.0 as *mut core::ffi::c_void);
                match lparam.0 {
                    1 => {  // Edit menu
                        // Enable Undo if edit control can do it
                        let lresult = SendMessageW(WND_EDIT, EM_CANUNDO, WPARAM(0), LPARAM(0));
                        let flag: MENU_ITEM_FLAGS = if lresult.0 == 0 {MF_GRAYED} else {MF_ENABLED};
                        let _ = EnableMenuItem(menu, 40007/*IDM_EDIT_UNDO*/, flag);

                        // Enable Paste if text is in the clipboard
                        let result = IsClipboardFormatAvailable(CF_TEXT.0 as u32);
                        let flag: MENU_ITEM_FLAGS = if let Some(_) = result.as_ref().err() {
                            MF_GRAYED } else { MF_ENABLED };
                        let _ = EnableMenuItem(menu, 40010/*IDM_EDIT_PASTE*/, flag);

                        // Enable Cut, Copy, and Del if text is selected
                        let sel_beg: i32 = 0;
                        let sel_end: i32 = 0;
                        let _ = SendMessageW(WND_EDIT, EM_GETSEL,
                                             WPARAM(&sel_beg as *const i32 as usize),
                                            LPARAM(&sel_end as *const i32 as isize));
                        let flag: MENU_ITEM_FLAGS = if sel_beg == sel_end { MF_GRAYED } else { MF_ENABLED };
                        let _ = EnableMenuItem(menu, 40008/*IDM_EDIT_CUT*/, flag);
                        let _ = EnableMenuItem(menu, 40009/*IDM_EDIT_COPY*/, flag);
                        let _ = EnableMenuItem(menu, 40011/*IDM_EDIT_CLEAR*/, flag);
                    },
                    2 => {  // SearchMenu
                        // Enable Find, Next, and Replace if modeless
                        let flag: MENU_ITEM_FLAGS = if DLG_MODELESS.0 == std::ptr::null_mut() {
                            MF_ENABLED } else { MF_GRAYED };
                        let _ = EnableMenuItem(menu, 40013/*IDM_SEARCH_FIND*/, flag);
                        let _ = EnableMenuItem(menu, 40014/*IDM_SEARCH_NEXT*/, flag);
                        let _ = EnableMenuItem(menu, 40015/*IDM_SEARCH_REPLACE*/, flag);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                if lparam.0 != 0 && loword!(wparam.0) ==EDIT_ID as u16 {
                    match hiword!(wparam.0) as u32 {
                        EN_UPDATE => {
                            NEED_SAVE = true;
                            return LRESULT(0);
                        },
                        EN_ERRSPACE | EN_MAXTEXT => {
                            MessageBoxW(window, w!("Edit control out of space."),
                                APP_NAME, MB_OK | MB_ICONSTOP);
                            return LRESULT(0);
                        }
                        _ => {}
                    }
                }
                match loword!(wparam.0) {
                    40001/*IDM_FILE_NEW*/ => {
                        if NEED_SAVE && ask_about_save(window, &*addr_of!(TITLE_NAME)) == IDCANCEL {
                            return LRESULT(0);
                        }
                        SetWindowTextW(WND_EDIT, w!("")).unwrap();
                        FILE_NAME = String::new();
                        TITLE_NAME = String::new();
                        do_caption(window, &*addr_of!(TITLE_NAME));
                        NEED_SAVE = false;
                        return LRESULT(0);
                    }
                    40002/*IDM_FILE_OPEN*/ => {
                        if NEED_SAVE && ask_about_save(window, &*addr_of!(TITLE_NAME)) == IDCANCEL {
                            return  LRESULT(0);
                        }
                        //if pop_file_open_dialog(window, *addr_of!(FILE_NAME), file_title) == TRUE {
                        #[allow(static_mut_refs)]
                        if pop_file_open_dialog(window, &mut FILE_NAME, &mut TITLE_NAME) == TRUE {
                            println!("file_name={:?}, file_title={:?}", FILE_NAME, TITLE_NAME);
                        }
                        do_caption(window, &*addr_of!(TITLE_NAME));
                        NEED_SAVE = false;
                        return LRESULT(0);
                    }
                    40003/*IDM_FILE_SAVE*/ | 40004/*IDM_FILE_SAVE_AS*/=> {
                        if loword!(wparam.0) == 40003/*IDM_FILE_SAVE*/ {
                            // if FILE_NAME != PWSTR::null() {

                            // }
                        }
                        // if PopFileSaveDlg() {
                        //
                        // }
                        return LRESULT(0);
                    }
                    40005/*IDM_FILE_PRINT*/ => {
                        //
                        return LRESULT(0);
                    }
                    40006/*IDM_APP_EXIT*/ => {
                        let _ = SendMessageW(window, WM_CLOSE, WPARAM(0), LPARAM(0));
                        return LRESULT(0);
                    }
                                                // Messages from Edit menu
                                                40007/*IDM_EDIT_UNDO*/ => {
                        let _ = SendMessageW(WND_EDIT, WM_UNDO, WPARAM(0), LPARAM(0));
                        return LRESULT(0);
                    }
                    40008/*IDM_EDIT_CUT*/ => {
                        let _ = SendMessageW(WND_EDIT, WM_CUT, WPARAM(0), LPARAM(0));
                        return LRESULT(0);
                    }
                    40009/*IDM_EDIT_COPY*/ => {
                        let _ = SendMessageW(WND_EDIT, WM_COPY, WPARAM(0), LPARAM(0));
                        return LRESULT(0);
                    }
                    40010/*IDM_EDIT_PASTE*/ => {
                        let _ = SendMessageW(WND_EDIT, WM_PASTE, WPARAM(0), LPARAM(0));
                        return LRESULT(0);
                    }
                    40011/*IDM_EDIT_CLEAR*/ => {
                        let _ = SendMessageW(WND_EDIT, WM_CLEAR, WPARAM(0), LPARAM(0));
                        return LRESULT(0);
                    }
                    40012/*IDM_EDIT_SELECT_ALL*/ => {
                        let _ = SendMessageW(WND_EDIT, EM_SETSEL, WPARAM(0), LPARAM(0));
                        return LRESULT(0);
                    }
                                                // Messages from Search menu
                                                40013/*IDM_SEARCH_FIND*/ => {
                        let _ = SendMessageW(WND_EDIT, EM_GETSEL, WPARAM(0),
                                             LPARAM(addr_of!(OFFSET) as *const i32 as isize));
                        // DLG_MODELESS = PopFindFindDlg(window);
                        return LRESULT(0);
                    }
                    40014/*IDM_SEARCH_NEXT*/ => {
                        let _ = SendMessageW(WND_EDIT, EM_GETSEL, WPARAM(0),
                                             LPARAM(addr_of!(OFFSET) as *const i32 as isize));
                        // if {
                        //
                        // }
                        return LRESULT(0);
                    }
                    40015/*IDM_SEARCH_REPLACE*/ => {
                        let _ = SendMessageW(WND_EDIT, EM_GETSEL, WPARAM(0),
                                             LPARAM(addr_of!(OFFSET) as *const i32 as isize));
                        // if {
                        //
                        // }
                        return LRESULT(0);
                    }
                    40016/*IDM_FORMAT_FONT*/ => {
                        // if {}
                        return LRESULT(0);
                    }
                    40017/*IDM_HELP*/ => {

                        return LRESULT(0);
                    }
                    40018/*IDM_APP_ABOUT*/ => {
                        let _ = DialogBoxParamW(INSTANCE,
                                                w!("AboutBox"),
                                                window,
                                                Some(about_dlg_proc),
                                                LPARAM(0));
                        return LRESULT(0);
                    }
                    _ => DefWindowProcW(window, message, wparam, lparam)
                };
                LRESULT(0)
            }
            WM_CTLCOLOREDIT => {
                // println!("WM_CTLCOLOREDIT");
                let hdc = HDC(wparam.0 as *mut core::ffi::c_void);
                let _hwnd = HWND(lparam.0 as *mut core::ffi::c_void);
                SetTextColor(hdc, COLORREF(rgb!(255, 255, 255)));
                SetBkColor(hdc, COLORREF(rgb!(0, 0, 0)));
                LRESULT(EDIT_BK_BRUSH.0 as isize)
            }
            WM_CLOSE => {
                if !NEED_SAVE || ask_about_save(window, &*addr_of!(TITLE_NAME)) !=IDCANCEL {
                    DestroyWindow(window).unwrap();
                }
                LRESULT(0)
            }
            WM_QUERYENDSESSION => {
                if !NEED_SAVE || ask_about_save(window, &*addr_of!(TITLE_NAME)) !=IDCANCEL {
                    LRESULT(1)
                } else {
                    LRESULT(0)
                }
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                let _ = DeleteObject(EDIT_BK_BRUSH);
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => {
                // Process "Find-Replace" messages
                if message == MSG_FIND_REP {
                    let ptr: &FINDREPLACEW = &*(lparam.0 as *const FINDREPLACEW);
                    if ptr.Flags & FR_DIALOGTERM == FR_DIALOGTERM {
                        DLG_MODELESS = HWND(std::ptr::null_mut());
                    }
                    if ptr.Flags & FR_FINDNEXT == FR_FINDNEXT {
                        // if {
                        //
                        // }
                    }
                    if ptr.Flags & FR_REPLACE == FR_REPLACE {
                    //
                    }
                    if ptr.Flags & FR_REPLACEALL == FR_REPLACEALL {
                        //
                    }
                }
                DefWindowProcW(window, message, wparam, lparam)
            },
        }
    }
}

extern "system" fn about_dlg_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> isize {
    unsafe {
        match message {
            WM_INITDIALOG => {
                println!("WM_INITDIALOG lparam={:?}", lparam);
                TRUE.0 as isize
            }
            WM_COMMAND => {
                match MESSAGEBOX_RESULT(loword!(wparam.0) as i32) {
                    IDOK => {
                        println!("IDOK");
                        let _ = EndDialog(window, 0);
                        TRUE.0 as isize
                    }
                    IDCANCEL => {
                        println!("IDCANCEL");
                        let _ = EndDialog(window, 0);
                        TRUE.0 as isize
                    }
                    _ => {
                        println!("Unknown WM_COMMAND. wparam.0={}", wparam.0);
                        FALSE.0 as isize
                    }
                }
            }
            _ => { FALSE.0 as isize }
        }
    }
}

#[allow(static_mut_refs)]
fn do_caption(hwnd: HWND, title_name: &String) {
    let app_name = util::from_wide_ptr(APP_NAME.0);
    let title = if title_name.is_empty() { UNTITLED } else { title_name };
    let caption = format!("{} - {}", &app_name, title);

    let caption_wchar = util::to_wide_chars(&caption);
    unsafe {
        SetWindowTextW(hwnd, PCWSTR::from_raw(caption_wchar.as_ptr())).unwrap();
    }
}

fn ask_about_save(hwnd: HWND, title_name: &String) -> MESSAGEBOX_RESULT {
    let title = if title_name.is_empty() { UNTITLED } else { title_name };
    let msg = format!("Save current changes in {}?", title);
    let msg_wchar = util::to_wide_chars(&msg);
    unsafe {
        let ret
            = MessageBoxW(hwnd, PCWSTR::from_raw(msg_wchar.as_ptr()), APP_NAME,
                          MB_YESNOCANCEL | MB_ICONQUESTION);
        if ret == IDYES {
            let lresult = SendMessageW(hwnd, WM_COMMAND,
                                      WPARAM(40003/*IDM_FILE_SAVE*/), LPARAM(0));
            if lresult.0 == 0 {
                return IDCANCEL;
            }
        }
        ret
    }
}

fn ok_message(hwnd: HWND, message: &String) {
    let msg_wchar = util::to_wide_chars(message);
    unsafe {
        let _ = MessageBoxW(hwnd, PCWSTR::from_raw(msg_wchar.as_ptr()), APP_NAME,
                            MB_OK | MB_ICONEXCLAMATION);
    }
}

