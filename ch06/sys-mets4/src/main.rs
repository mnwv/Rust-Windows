#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, Controls::*, Input::KeyboardAndMouse::*},
    Win32::Globalization::lstrlenA,
};

mod sys_mets;
#[macro_use] mod macros;

static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;
static mut CX_CAPS: i32 = 0;
static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut MAX_WIDTH: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("SysMets4");

        let wnd_class = WNDCLASSA {
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

        let atom = RegisterClassA(&wnd_class);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExA(
            0,
            app_name,
            s!("Get System Metrics No. 4"),
            WS_OVERLAPPEDWINDOW | WS_VSCROLL | WS_HSCROLL,
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

        while GetMessageA(&mut message, 0, 0, 0) != 0 {
            TranslateMessage(&message);
            DispatchMessageA(&message);
        }
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                let hdc = GetDC(window);
                let mut tm: TEXTMETRICA = std::mem::zeroed();
                GetTextMetricsA(hdc, &mut tm);
                CX_CHAR = tm.tmAveCharWidth;
                CX_CAPS = {
                    let m = if tm.tmPitchAndFamily & 1 == 1 {3} else {2};
                    CX_CHAR * m / 2 };
                CY_CHAR = tm.tmHeight + tm.tmExternalLeading;
                ReleaseDC(window, hdc);
                
                // Save the width of the three columns;
                MAX_WIDTH = 40 * CX_CHAR + 22 * CX_CAPS;
                0
            }

            WM_SIZE => {
                CX_CLIENT = loword!(lparam);
                CY_CLIENT = hiword!(lparam);

                // Set vertical scroll bar range and page size
                let si: SCROLLINFO = SCROLLINFO {
                    cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_RANGE | SIF_PAGE,
                    nMin: 0,
                    nMax: (sys_mets::SYS_METRICS.len() -1) as i32,
                    nPage: (CY_CLIENT / CY_CHAR) as u32,
                    nPos: 0,
                    nTrackPos: 0,
                };
                SetScrollInfo(window, SB_VERT, &si, -1);

                // Set horizontal scroll bar range and page size
                let si:SCROLLINFO = SCROLLINFO {
                    cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_RANGE | SIF_PAGE,
                    nMin: 0,
                    nMax: 2 + MAX_WIDTH / CX_CHAR,
                    nPage: (CX_CLIENT / CX_CHAR) as u32,
                    nPos: 0,
                    nTrackPos: 0,
                };
                SetScrollInfo(window, SB_HORZ, &si, -1);
                0
            }

            WM_VSCROLL => {
                // Get all the vertical scroll bar information.
                let mut si: SCROLLINFO = std::mem::zeroed();
                si.cbSize = std::mem::size_of::<SCROLLINFO>() as u32;
                si.fMask = SIF_ALL;
                GetScrollInfo(window, SB_VERT, &mut si);
                // Save the position for comparison later on
                let vert_pos = si.nPos;
                match loword!(wparam) {
                    SB_TOP => si.nPos = si.nMin,
                    SB_BOTTOM => si.nPos = si.nMax,
                    SB_LINEUP => si.nPos -= 1,
                    SB_LINEDOWN => si.nPos += 1,
                    SB_PAGEUP => si.nPos -= si.nPage as i32,
                    SB_PAGEDOWN => si.nPos += si.nPage as i32,
                    SB_THUMBTRACK => si.nPos = si.nTrackPos,
                    _ => {},
                }

                // Set the position and then retrieve it.
                // Due to adjustments by Windows it may not be the same
                //  as the value set.
                si.fMask = SIF_POS;
                SetScrollInfo(window, SB_VERT, &si, -1);
                GetScrollInfo(window, SB_VERT, &mut si);

                // If the posiotion has changed, scroll the window and update it
                if si.nPos != vert_pos {
                    ScrollWindow(window, 0, CY_CHAR * (vert_pos - si.nPos), std::ptr::null(), std::ptr::null());
                    UpdateWindow(window);
                }
                0
            }

            WM_HSCROLL => {
                // Get all the horizontal scroll bar information
                let mut si: SCROLLINFO = std::mem::zeroed();
                si.cbSize = std::mem::size_of::<SCROLLINFO>() as u32;
                si.fMask = SIF_ALL;
                GetScrollInfo(window, SB_HORZ, &mut si);
                // Save the position for comparison later on
                let horz_pos = si.nPos;

                match loword!(wparam) {
                    SB_LINELEFT => si.nPos -= 1,
                    SB_LINERIGHT => si.nPos += 1,
                    SB_PAGELEFT => si.nPos -= si.nPage as i32,
                    SB_PAGERIGHT => si.nPos += si.nPage as i32,
                    SB_THUMBPOSITION => si.nPos = si.nTrackPos,
                    _ => {},
                }

                // Set the position and then retrieve it.
                // Due to adjustments by Windows it may not be the same
                //  as the value set.
                si.fMask = SIF_POS;
                SetScrollInfo(window, SB_HORZ, &si, -1);
                GetScrollInfo(window, SB_HORZ, &mut si);

                // If the position has changed, scroll the window
                if si.nPos != horz_pos {
                    ScrollWindow(window, CX_CHAR * (horz_pos - si.nPos), 0, std::ptr::null(), std::ptr::null());
                }
                0
            }

            WM_KEYDOWN => {
                match wparam as u16 {
                    VK_HOME => { SendMessageA(window, WM_VSCROLL, SB_TOP as WPARAM, 0); },
                    VK_END => { SendMessageA(window, WM_VSCROLL, SB_BOTTOM as WPARAM, 0); },
                    VK_PRIOR => { SendMessageA(window, WM_VSCROLL, SB_PAGEUP as WPARAM, 0); },
                    VK_NEXT => { SendMessageA(window, WM_VSCROLL, SB_PAGEDOWN as WPARAM, 0); },
                    VK_UP => { SendMessageA(window, WM_VSCROLL, SB_LINEUP as WPARAM, 0); },
                    VK_DOWN => { SendMessageA(window, WM_VSCROLL, SB_LINEDOWN as WPARAM, 0); },
                    VK_LEFT => { SendMessageA(window, WM_HSCROLL, SB_PAGEUP as WPARAM, 0); },
                    VK_RIGHT => { SendMessageA(window, WM_HSCROLL, SB_PAGEDOWN as WPARAM, 0); },
                    _ => {},
                }
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                let mut si: SCROLLINFO = std::mem::zeroed();
                si.cbSize = std::mem::size_of::<SCROLLINFO>() as u32;
                si.fMask = SIF_POS;
                GetScrollInfo(window, SB_VERT, &mut si);
                let vert_pos = si.nPos;
                GetScrollInfo(window, SB_HORZ, &mut si);
                let horz_pos = si.nPos;
                // Find painting limits
                let paint_beg = std::cmp::max(0, vert_pos + ps.rcPaint.top / CY_CHAR) as usize;
                let paint_end = std::cmp::min(sys_mets::SYS_METRICS.len() - 1,
                                                (vert_pos + ps.rcPaint.bottom / CY_CHAR) as usize);
                for i in paint_beg..=paint_end {
                    let sys_met = &sys_mets::SYS_METRICS[i];
                    let mut x = CX_CHAR * (1 - horz_pos);
                    let y: i32 = CY_CHAR * (i as i32 - vert_pos);
                    TextOutA(hdc, x, y, sys_met.label, lstrlenA(sys_met.label));
                    x += 22 * CX_CAPS;
                    TextOutA(hdc, x, y, sys_met.desc, lstrlenA(sys_met.desc));
                    SetTextAlign(hdc, TA_RIGHT | TA_TOP );
                    x += 40 * CX_CHAR;
                    let val = &GetSystemMetrics(sys_met.index);
                    let val_str: &str = &format!("{:5}", val);
                    TextOutA(hdc, x, y, val_str.as_ptr(), 5);
                    SetTextAlign(hdc, TA_LEFT | TA_TOP);
                }
                EndPaint(window, &ps);
                0
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}