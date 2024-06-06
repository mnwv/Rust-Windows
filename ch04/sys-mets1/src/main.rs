#![windows_subsystem = "windows"]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::WindowsAndMessaging::*,
    Win32::Globalization::lstrlenA,
};

mod sys_mets;

static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;
static mut CX_CAPS: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("SysMets1");

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
            s!("Get System Metrics No. 1"),
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
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                for (i, sys_met) in (0_i32..).zip(sys_mets::SYS_METRICS.iter()) {
                    let mut x = 0;
                    let y = CY_CHAR * i;
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