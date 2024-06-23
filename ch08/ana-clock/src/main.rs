#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW, 
    Win32::UI::{WindowsAndMessaging::*, },
    Win32::System::SystemInformation::*,
};

#[macro_use] mod macros;

const ID_TIMER: usize = 1;
const TWOPI: f32 = 2.0 * 3.14159;

static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut PREVIOUS_TIME: SYSTEMTIME = unsafe { std::mem::zeroed() };

const HANDS_POINTS: [[POINT;5];3] = [
    [ POINT{x:0, y:-150}, POINT{x:100, y:0}, POINT{x:0, y:600}, POINT{x:-100, y:0}, POINT{x:0, y:-150}, ],
    [ POINT{x:0, y:-200}, POINT{x:50, y:0}, POINT{x:0, y:800}, POINT{x:-50, y:0}, POINT{x:0, y:-200}, ],
    [ POINT{x:0, y:0}, POINT{x:0, y:0}, POINT{x:0, y:0}, POINT{x:0, y:0}, POINT{x:0, y:800}, ],
];

fn main() {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("Clock");

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: 0,
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
            w!("Analog Clock"),
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

fn set_isotropic(hdc: HDC, cx_client: i32, cy_client: i32) {
    unsafe {
        SetMapMode(hdc, MM_ISOTROPIC);
        SetWindowExtEx(hdc, 1000, 1000, std::ptr::null_mut());
        SetViewportExtEx(hdc, cx_client / 2, -cy_client / 2, std::ptr::null_mut());
        SetViewportOrgEx(hdc, cx_client / 2, cy_client / 2, std::ptr::null_mut());
    }
}

fn rotate_point(points: &mut [POINT], len: usize, angle: f32) {
    let num_cos = (TWOPI * angle / 360.0).cos();
    let num_sin = (TWOPI * angle / 360.0).sin();

    for i in 0..len {
        let x = points[i].x as f32 * num_cos + points[i].y as f32 * num_sin;
        let y = points[i].y as f32 * num_cos - points[i].x as f32 * num_sin;
        points[i].x = x as i32;
        points[i].y = y as i32;
    }
}

fn draw_clock(hdc: HDC) {
    let mut points:[POINT;3] = unsafe { std::mem::zeroed() };

    for angle in (0..360).step_by(6) {
        points[0].x = 0;
        points[0].y = 900;

        rotate_point(&mut points, 1, angle as f32);

        points[2].x = if angle % 5 != 0 {33} else {100};
        points[2].y = points[2].x;

        points[0].x -= points[2].x / 2;
        points[0].y -= points[2].y / 2;

        points[1].x = points[0].x + points[2].x;
        points[1].y = points[0].y + points[2].y;

        unsafe {
            SelectObject(hdc, GetStockObject(BLACK_BRUSH));
            Ellipse(hdc, points[0].x, points[0].y, points[1].x, points[1].y);
        }
    }
}

fn draw_hands(hdc: HDC, time: &SYSTEMTIME, change: bool) {
    let angles: [i32; 3] = [
        (time.wHour as i32 * 30) % 360 + time.wMinute as i32 / 2,
        time.wMinute as i32 * 6,
        time.wSecond as i32 * 6,
    ];
    let mut hands_points = HANDS_POINTS;
    let start = if change {0} else {2};
    for i in start..3 {
        rotate_point(&mut hands_points[i], 5, angles[i] as f32);
        unsafe {
            Polyline(hdc, &hands_points[i][0], 5);
        }
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_CREATE => {
                SetTimer(window, ID_TIMER, 1000, None);
                GetLocalTime(std::ptr::addr_of_mut!(PREVIOUS_TIME));
                0
            },
            WM_SIZE => {
                CX_CLIENT = loword!(lparam) as i32;
                CY_CLIENT = hiword!(lparam) as i32;
                0
            },
            WM_TIMER => {
                let mut st: SYSTEMTIME = std::mem::zeroed();
                GetLocalTime(&mut st);
                let is_change = st.wHour != PREVIOUS_TIME.wHour ||
                                st.wMinute != PREVIOUS_TIME.wMinute;
                let hdc = GetDC(window);
                set_isotropic(hdc, CX_CLIENT, CY_CLIENT);
                SelectObject(hdc, GetStockObject(WHITE_PEN));
                let prev_time = PREVIOUS_TIME;
                draw_hands(hdc, &prev_time, is_change);

                SelectObject(hdc, GetStockObject(BLACK_PEN));
                draw_hands(hdc, &st, true);
                ReleaseDC(window, hdc);
                PREVIOUS_TIME = st;
                0
            },
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                set_isotropic(hdc, CX_CLIENT, CY_CLIENT);
                draw_clock(hdc);
                let prev_time = PREVIOUS_TIME;
                draw_hands(hdc, &prev_time, true);

                EndPaint(window, &ps);
                0
            },
            WM_DESTROY => {
                KillTimer(window, ID_TIMER);
                PostQuitMessage(0);
                0
            },
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
