#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*},
    Win32::Globalization::lstrlenA,
};

struct DevCapsItem {
	pub index: u32,
	pub label: *const u8,
	pub desc: *const u8,
}

const DEV_CAPS: [DevCapsItem; 20] = [
    DevCapsItem { index:HORZSIZE,      label: s!("HORZSIZE"),     desc: s!("Width in millimeters:"),},
    DevCapsItem { index:VERTSIZE,      label: s!("VERTSIZE"),     desc: s!("Height in millimeters:"),},
    DevCapsItem { index:HORZRES,       label: s!("HORZRES"),      desc: s!("Width in pixels:"),},
    DevCapsItem { index:VERTRES,       label: s!("VERTRES"),      desc: s!("Height in raster lines:"),},
    DevCapsItem { index:BITSPIXEL,     label: s!("BITSPIXEL"),    desc: s!("Color bits per pixel:"),},
    DevCapsItem { index:PLANES,        label: s!("PLANES"),       desc: s!("Number of color planes:"),},
    DevCapsItem { index:NUMBRUSHES,    label: s!("NUMBRUSHES"),   desc: s!("Number of device brushes:"),},
    DevCapsItem { index:NUMPENS,       label: s!("NUMPENS"),      desc: s!("Number of device pens:"),},
    DevCapsItem { index:NUMMARKERS,    label: s!("NUMMARKERS"),   desc: s!("Number of device markers:"),},
    DevCapsItem { index:NUMFONTS,      label: s!("NUMFONTS"),     desc: s!("Number of device fonts:"),},
    DevCapsItem { index:NUMCOLORS,     label: s!("NUMCOLORS"),    desc: s!("Number of device colors:"),},
    DevCapsItem { index:PDEVICESIZE,   label: s!("PDEVICESIZE"),  desc: s!("Size of device structure:"),},
    DevCapsItem { index:ASPECTX,       label: s!("ASPECTX"),      desc: s!("Relative width of pixel:"),},
    DevCapsItem { index:ASPECTY,       label: s!("ASPECTY"),      desc: s!("Relative height of pixel:"),},
    DevCapsItem { index:ASPECTXY,      label: s!("ASPECTXY"),     desc: s!("Relative diagonal of pixel:"),},
    DevCapsItem { index:LOGPIXELSX,    label: s!("LOGPIXELSX"),   desc: s!("Horizontal dots per inch:"),},
    DevCapsItem { index:LOGPIXELSY,    label: s!("LOGPIXELSY"),   desc: s!("Vertical dots per inch:"),},
    DevCapsItem { index:SIZEPALETTE,   label: s!("SIZEPALETTE"),  desc: s!("Number of palette entries:"),},
    DevCapsItem { index:NUMRESERVED,   label: s!("NUMRESERVED"),  desc: s!("Reserved palette entries:"),},
    DevCapsItem { index:COLORRES,      label: s!("COLORRES"),     desc: s!("Actual color resolution:"),},
];

#[macro_use] mod macros;

static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;
static mut CX_CAPS: i32 = 0;

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = s!("DevCap1");

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
            s!("Device Capabilities"),
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
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);
                for (i, dev_cap) in (0_i32..).zip(DEV_CAPS.iter()) {
                    let mut x = 0;
                    let y = (CY_CHAR * i) as i32;
                    TextOutA(hdc, x, y, dev_cap.label, lstrlenA(dev_cap.label));
                    x += 14 * CX_CAPS;
                    TextOutA(hdc, x, y, dev_cap.desc, lstrlenA(dev_cap.desc));
                    SetTextAlign(hdc, TA_RIGHT | TA_TOP );
                    x += 40 * CX_CHAR;
                    let val = &GetDeviceCaps(hdc, dev_cap.index as i32);
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