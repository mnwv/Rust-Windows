#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::{WindowsAndMessaging::*, 
                Controls::*,
                Input::KeyboardAndMouse::*,
             },
    Win32::System::SystemServices::*,
};

#[macro_use] mod macros;

const CR_PRIM: [COLORREF;3] = [ rgb!(255,0,0), rgb!(0,255,0), rgb!(0,0,255), ];
const COLOR_LABELS: [*const u16; 3] = [ w!("Red"), w!("Green"), w!("Blue"), ];
static mut ID_FOCUS: isize = 0;
static mut OLD_SCROLL_WNDPROC: [WNDPROC; 3] = unsafe { std::mem::zeroed() };
static mut BRUSHES: [HBRUSH; 3] = unsafe { std::mem::zeroed() };
static mut BRUSH_STATIC: HBRUSH = 0;
static mut HWND_SCROLLS: [HWND; 3] = [0, 0, 0];
static mut HWND_LABELS: [HWND; 3] = [0, 0, 0];
static mut HWND_VALUES: [HWND; 3] = [0, 0, 0];
static mut HWND_RECT: HWND = 0;
static mut COLORS: [i32; 3] = [0, 0, 0];
static mut CY_CHAR: i32 = 0;
static mut RC_COLOR: RECT = unsafe { std::mem::zeroed() };

fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("Colors1");

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
            w!("Color Scroll"),
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
                let instance = GetWindowLongPtrW(window, GWL_HINSTANCE) as HINSTANCE;
                HWND_RECT = CreateWindowExW(
                    0, w!("static"), std::ptr::null(), 
                    WS_CHILD | WS_VISIBLE | SS_WHITERECT,
                    0, 0, 0, 0,
                    window, 9 as HMENU, instance, std::ptr::null()
                );
                for i in 0..3 {
                    HWND_SCROLLS[i] = CreateWindowExW(
                        0, w!("scrollbar"), std::ptr::null(),
                        WS_CHILD | WS_VISIBLE | WS_TABSTOP | SBS_VERT as u32,
                        0, 0, 0, 0,
                        window, i as HMENU, instance, std::ptr::null()
                    );
                    SetScrollRange(HWND_SCROLLS[i], SB_CTL, 0, 255, FALSE);
                    SetScrollPos(HWND_SCROLLS[i], SB_CTL, 0, FALSE);

                    HWND_LABELS[i] = CreateWindowExW(
                        0, w!("static"), COLOR_LABELS[i],
                        WS_CHILD | WS_VISIBLE | SS_CENTER,
                        0, 0, 0, 0,
                        window, (i + 3) as HMENU, instance, std::ptr::null()
                    );
                    HWND_VALUES[i] = CreateWindowExW(
                        0, w!("static"), w!("0"),
                        WS_CHILD | WS_VISIBLE | SS_CENTER,
                        0, 0, 0, 0,
                        window, (i + 6) as HMENU, instance, std::ptr::null()
                    );
                    let wnd_proc = SetWindowLongPtrW(HWND_SCROLLS[i], GWL_WNDPROC, scroll_proc as isize);
                    // 
                    // https://doc.rust-lang.org/std/primitive.fn.html
                    let fnptr = wnd_proc as *const ();
                    let fnptr: unsafe extern "system" fn(_: isize, _: u32, _: usize, _: isize) -> isize 
                        = std::mem::transmute(fnptr);
                    OLD_SCROLL_WNDPROC[i] = Some(fnptr);

                    BRUSHES[i] = CreateSolidBrush(CR_PRIM[i]);
                }
                BRUSH_STATIC = CreateSolidBrush(GetSysColor(COLOR_BTNHIGHLIGHT));
                CY_CHAR = hiword!(GetDialogBaseUnits()) as i32; // 18
                0
            },
            WM_SIZE => {
                let cx_client = loword!(lparam) as i32;
                let cy_client = hiword!(lparam) as i32;

                #[allow(static_mut_refs)]
                SetRect(&mut RC_COLOR, cx_client / 2, 0, cx_client, cy_client);

                 MoveWindow(HWND_RECT, 0, 0, cx_client / 2, cy_client, TRUE);
                for i in 0..3 {
                    MoveWindow(HWND_SCROLLS[i],
                                (2 * i + 1) as i32 *cx_client / 14, 2 * CY_CHAR,
                                cx_client / 14, cy_client - 4 * CY_CHAR, TRUE);
                    MoveWindow(HWND_LABELS[i],
                                (4 * i + 1) as i32 *cx_client / 28, CY_CHAR / 2,
                                cx_client / 7, CY_CHAR, TRUE);
                    MoveWindow(HWND_VALUES[i],
                                (4 * i + 1) as i32 *cx_client / 28, 
                                cy_client - 3 * CY_CHAR / 2,
                                cx_client / 7, CY_CHAR, TRUE);
                }
                SetFocus(window);
                0
            },
            WM_SETFOCUS => {
                SetFocus(HWND_SCROLLS[ID_FOCUS as usize]);
                0
            },
            WM_VSCROLL => {
                let i = GetWindowLongW(lparam as HWND, GWL_ID) as usize;
                match loword!(wparam) as i32 {
                    SB_PAGEDOWN => { COLORS[i] += 15; },
                    SB_LINEDOWN => { COLORS[i] = std::cmp::min(255, COLORS[i] + 1); },
                    SB_PAGEUP => { COLORS[i] -= 15; },
                    SB_LINEUP => { COLORS[i] = std::cmp::max(0, COLORS[i] - 1); },
                    SB_TOP => { COLORS[i] = 0; },
                    SB_BOTTOM => { COLORS[i] = 255; },
                    SB_THUMBPOSITION => { COLORS[i] = hiword!(wparam) as i32; },
                    SB_THUMBTRACK => { COLORS[i] = hiword!(wparam) as i32; },
                    _ => {},
                }
                SetScrollPos(HWND_SCROLLS[i], SB_CTL, COLORS[i], TRUE);

                let s = format!("{}", COLORS[i]);
                use std::ffi::OsStr;
                use std::os::windows::ffi::OsStrExt;
                let wide_chars = OsStr::new(&s).encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
                SetWindowTextW(HWND_VALUES[i], wide_chars.as_ptr());
                DeleteObject(
                    SetClassLongPtrW(window, GCL_HBRBACKGROUND,
                        CreateSolidBrush(rgb!(COLORS[0], COLORS[1], COLORS[2]) as u32)) as isize);
                #[allow(static_mut_refs)]
                InvalidateRect(window, &RC_COLOR, TRUE);
                0
            },
            WM_CTLCOLORSCROLLBAR => {
                let i = GetWindowLongW(lparam as HWND, GWL_ID) as usize;
                return BRUSHES[i] as LRESULT;
            },
            WM_CTLCOLORSTATIC => {
                let i = GetWindowLongW(lparam as HWND, GWL_ID) as usize;
                if i >= 3 && i <= 8 {
                    SetTextColor(wparam as HDC, CR_PRIM[i % 3]);
                    SetBkColor(wparam as HDC, GetSysColor(COLOR_BTNHIGHLIGHT));
                    return BRUSH_STATIC as LRESULT;
                }
                return DefWindowProcW(window, message, wparam, lparam);
            },
            WM_SYSCOLORCHANGE => {
                DeleteObject(BRUSH_STATIC);
                BRUSH_STATIC = CreateSolidBrush(GetSysColor(COLOR_BTNHIGHLIGHT));
                0
            },
            WM_DESTROY => {
                DeleteObject(
                    SetClassLongPtrW(window, GCL_HBRBACKGROUND, 
                        GetStockObject(WHITE_BRUSH)) as HBRUSH
                );
                for i in 0..3 {
                    DeleteObject(BRUSHES[i]);
                }
                DeleteObject(BRUSH_STATIC);
                PostQuitMessage(0);
                0
            },
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}

extern "system" fn scroll_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let id = GetWindowLongW(hwnd, GWL_ID);
        match message {
            WM_KEYDOWN => {
                if wparam == VK_TAB as usize {
                     SetFocus(GetDlgItem(GetParent(hwnd),
                         (id + if GetKeyState(VK_SHIFT as i32) < 0 {2} else {1}) %3));
                }
            },
            WM_SETFOCUS => {
                ID_FOCUS = id as isize;
            },
            _ => {},
        }
        CallWindowProcW(OLD_SCROLL_WNDPROC[id as usize], hwnd, message, wparam, lparam)
    }
}