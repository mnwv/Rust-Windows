#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::*, 
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, 
    Win32::UI::{WindowsAndMessaging::*, Input::KeyboardAndMouse::*},
};

#[macro_use] mod macros;

static mut CHAR_SET: i32 = DEFAULT_CHARSET as i32;
static mut CX_CHAR: i32 = 0;
static mut CY_CHAR: i32 = 0;
static mut CX_CLIENT: i32 = 0;
static mut CY_CLIENT: i32 = 0;
static mut CX_BUFFER: i32 = 0;
static mut CY_BUFFER: i32 = 0;
static mut X_CARET: i32 = 0;
static mut Y_CARET: i32 = 0;
static mut BUFFER: Vec<u16> = Vec::new();

fn main() {
    unsafe {
        let instance = GetModuleHandleA(std::ptr::null());
        debug_assert!(instance != 0);

        let app_name = w!("Typer");

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
            w!("Typing Program"),
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

// WM_CREATE, WM_INPUTLANGCHANGE からコールされる
unsafe fn init(hwnd: HWND) {
    let hdc = GetDC(hwnd);
    let mut tm: TEXTMETRICW = std::mem::zeroed();
    let font = CreateFontW(0, 0, 0, 0, 0, 0, 0, 0, 
                CHAR_SET as u32, 0, 0, 0,
                FIXED_PITCH as u32, std::ptr::null());
    let old_object = SelectObject(hdc, font);
    GetTextMetricsW(hdc, &mut tm);
    CX_CHAR = tm.tmAveCharWidth;
    CY_CHAR = tm.tmHeight;
    DeleteObject(old_object);
    ReleaseDC(hwnd, hdc);
    CX_BUFFER = std::cmp::max(1, CX_CLIENT / CX_CHAR);
    CY_BUFFER = std::cmp::max(1, CY_CLIENT / CY_CHAR);
    BUFFER.clear();
    for _ in 0..CY_BUFFER * CX_BUFFER {
        BUFFER.push(' ' as u16);
    }
    X_CARET = 0;
    Y_CARET = 0;
    if hwnd == GetFocus() {
        SetCaretPos(X_CARET * CX_CHAR, Y_CARET * CY_CHAR);
    }
    InvalidateRect(hwnd, std::ptr::null(), TRUE);
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_INPUTLANGCHANGE => {
                CHAR_SET = wparam as i32;
                init(window);
                0
            }
            WM_CREATE => {
                init(window);
                0
            }

            WM_SIZE => {
                CX_CLIENT = loword!(lparam);
                CY_CLIENT = hiword!(lparam);
                init(window);
                0
            }

            WM_SETFOCUS => {
                CreateCaret(window, 0, CX_CHAR, CY_CHAR);
                SetCaretPos(X_CARET * CX_CHAR, Y_CARET * CY_CHAR);
                ShowCaret(window);
                0
            }

            WM_KILLFOCUS => {
                HideCaret(window);
                DestroyCaret();
                0
            }

            WM_KEYDOWN => {
                match wparam as u16 {
                    VK_HOME => { X_CARET = 0; },
                    VK_END => { X_CARET = CX_BUFFER - 1; },
                    VK_PRIOR => { Y_CARET = 0; },
                    VK_NEXT => { Y_CARET = CY_BUFFER - 1; },
                    VK_LEFT => { X_CARET = std::cmp::max(X_CARET - 1, 0)},
                    VK_RIGHT => { X_CARET = std::cmp::min(X_CARET + 1, CX_BUFFER - 1); },
                    VK_UP => { Y_CARET = std::cmp::max(Y_CARET - 1, 0); },
                    VK_DOWN => { Y_CARET = std::cmp::min(Y_CARET + 1, CY_BUFFER - 1); },
                    VK_DELETE => {  // Delete charactor on the caret position
                        for x in X_CARET..CX_BUFFER - 1 {
                            BUFFER[(Y_CARET*CX_BUFFER+x) as usize] = 
                                BUFFER[(Y_CARET*CX_BUFFER+x+1) as usize];
                        }
                        BUFFER[(Y_CARET*CX_BUFFER+CX_BUFFER-1) as usize] = ' ' as u16;
                        
                        HideCaret(window);
                        let hdc = GetDC(window);
                        let font = CreateFontW(0, 0, 0, 0, 0, 0, 0, 0, 
                            CHAR_SET as u32, 0, 0, 0,
                            FIXED_PITCH as u32, std::ptr::null());
                        let old_object = SelectObject(hdc, font);
                        TextOutW(hdc, X_CARET * CX_CHAR, Y_CARET * CY_CHAR,
                            &BUFFER[(Y_CARET*CX_BUFFER+X_CARET) as usize] as *const u16,
                            CX_BUFFER-X_CARET);
                        DeleteObject(SelectObject(hdc, old_object));
                        ReleaseDC(window, hdc);
                        ShowCaret(window);
                    },
                    _ => {},
                };
                SetCaretPos(X_CARET * CX_CHAR, Y_CARET * CY_CHAR);
                0
            }

            WM_CHAR => {
                for _ in 0..loword!(lparam) {   // repeat number
                    match char::from_u32(wparam as u32) {
                        Some('\x08') => {
                            if X_CARET > 0 {
                                X_CARET -= 1;
                                SendMessageW(window, WM_KEYDOWN, VK_DELETE as usize, 1);
                            }
                        },
                        Some('\t') => {
                            loop {
                                SendMessageW(window, WM_CHAR, ' ' as usize, 1);
                                if X_CARET % 8 == 0 { break; }
                            }
                        },
                        Some('\n') => {
                            println!("Enter");
                        },
                        Some('\r') => {
                            X_CARET = 0;
                            Y_CARET += 1;
                            if Y_CARET == CY_BUFFER {
                                Y_CARET = 0;
                            }
                            println!("CR, Y_CARET={}", Y_CARET);

                        },
                        Some('\x1b') => {
                            BUFFER.clear();
                            for _ in 0..CY_BUFFER * CX_BUFFER {
                                BUFFER.push(' ' as u16);
                            }
                            X_CARET = 0;
                            Y_CARET = 0;
                            InvalidateRect(window, std::ptr::null(), 0);
                        },
                        _ => {
                            BUFFER[(Y_CARET*CX_BUFFER+X_CARET) as usize] = wparam as u16;

                            HideCaret(window);
                            let hdc = GetDC(window);
                            let font = CreateFontW(0, 0, 0, 0, 0, 0, 0, 0, 
                                CHAR_SET as u32, 0, 0, 0,
                                FIXED_PITCH as u32, std::ptr::null());
                            let old_object = SelectObject(hdc, font);
                            TextOutW(hdc, X_CARET * CX_CHAR, Y_CARET * CY_CHAR,
                                        &BUFFER[(Y_CARET*CX_BUFFER+X_CARET) as usize], 1);
                            DeleteObject(SelectObject(hdc, old_object));
                            ReleaseDC(window, hdc);
                            ShowCaret(window);
                            X_CARET += 1;
                            if X_CARET == CX_BUFFER {
                                X_CARET = 0;
                                Y_CARET += 1;
                                if Y_CARET == CY_BUFFER{
                                    Y_CARET = 0;
                                }
                            }
                        },
                    }
                }
                SetCaretPos(X_CARET * CX_CHAR, Y_CARET * CY_CHAR);
                0
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(window, &mut ps);

                let font = CreateFontW(0, 0, 0, 0, 0, 0, 0, 0, 
                    CHAR_SET as u32, 0, 0, 0,
                    FIXED_PITCH as u32, std::ptr::null());
                let old_object = SelectObject(hdc, font);
                    
                for y in 0..CY_BUFFER {
                    TextOutW(hdc, 0, y*CY_CHAR, &BUFFER[(y*CX_BUFFER) as usize], CX_BUFFER);
                }

                DeleteObject(SelectObject(hdc, old_object));
                EndPaint(window, &ps);
                0
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}