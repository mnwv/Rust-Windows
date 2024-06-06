#![windows_subsystem = "windows"]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, Controls::*,},
    Win32::Globalization::lstrlenA,
};

mod sys_mets;
#[macro_use] mod macros;

static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;
static mut CX_CAPS: i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut VSCROLL_POS: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("SysMets2");

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
            s!("Get System Metrics No. 2"),
            WS_OVERLAPPEDWINDOW | WS_VSCROLL,
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

                SetScrollRange(window, SB_VERT, 0, (sys_mets::SYS_METRICS.len() - 1) as i32, 0);
                SetScrollPos(window, SB_VERT, VSCROLL_POS, -1);
                0
            }

            WM_SIZE => {
                CY_CLIENT = hiword!(lparam);
                0
            }

            WM_VSCROLL => {
                match (wparam & 0xFFFF) as i32 {
                    SB_LINEUP => VSCROLL_POS -= 1,
                    SB_LINEDOWN => VSCROLL_POS += 1,
                    SB_PAGEUP => VSCROLL_POS -= CY_CLIENT / CY_CHAR,
                    SB_PAGEDOWN => VSCROLL_POS += CY_CLIENT / CY_CHAR,
                    SB_THUMBPOSITION => VSCROLL_POS = hiword!(wparam),
                    _ => {},
                }

                VSCROLL_POS = std::cmp::max(0, std::cmp::min(VSCROLL_POS, (sys_mets::SYS_METRICS.len() - 1) as i32));

                if VSCROLL_POS != GetScrollPos(window, SB_VERT) {
                    SetScrollPos(window, SB_VERT, VSCROLL_POS, -1);
                    InvalidateRect(window, std::ptr::null(), -1);
                }
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                for (i, sys_met) in (0_i32..).zip(sys_mets::SYS_METRICS.iter()) {
                    let mut x = 0;
                    let y = CY_CHAR * (i - VSCROLL_POS);
                    TextOutA(hdc, x, y, sys_met.label, lstrlenA(sys_met.label));
                    x = 22 * CX_CAPS;
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