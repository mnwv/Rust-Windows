use std::ptr::addr_of_mut;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{BOOL, COLORREF, HINSTANCE, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateFontIndirectW, GetObjectW, GetStockObject, HFONT, LOGFONTW, SYSTEM_FONT};
use windows::Win32::UI::Controls::Dialogs::{ChooseFontW, CF_EFFECTS, CF_INITTOLOGFONTSTRUCT, CF_SCREENFONTS, CHOOSEFONTW, CHOOSEFONT_FONT_TYPE};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_SETFONT};

static mut LOG_FONT: LOGFONTW = unsafe { std::mem::zeroed() };
static mut FONT: HFONT = HFONT( std::ptr::null_mut());
pub unsafe fn pop_font_initialize(wnd_edit: HWND) {
    let gdi_obj = GetStockObject(SYSTEM_FONT);
    let ret = GetObjectW(gdi_obj,
                       size_of::<LOGFONTW>() as i32,
                       Some(addr_of_mut!(LOG_FONT) as *mut core::ffi::c_void));
    println!("GetObjectW() returns:{}", ret);
    FONT = CreateFontIndirectW(addr_of_mut!(LOG_FONT));
    let _ = SendMessageW(wnd_edit, WM_SETFONT, WPARAM(FONT.0 as usize), LPARAM(0));
}

pub unsafe fn pop_font_choose_font(hwnd: HWND) -> BOOL {
    let mut cf =  CHOOSEFONTW {
        lStructSize: size_of::<CHOOSEFONTW>() as u32,
        hwndOwner: hwnd,
        hDC: Default::default(),
        lpLogFont: addr_of_mut!(LOG_FONT),
        iPointSize: 0,
        Flags: CF_INITTOLOGFONTSTRUCT | CF_SCREENFONTS | CF_EFFECTS,
        rgbColors: COLORREF(0),
        lCustData: LPARAM(0),
        lpfnHook: None,
        lpTemplateName: PCWSTR::null(),
        hInstance: HINSTANCE(std::ptr::null_mut()),
        lpszStyle: PWSTR::null(),
        nFontType: CHOOSEFONT_FONT_TYPE(0),
        ___MISSING_ALIGNMENT__: 0,
        nSizeMin: 0,
        nSizeMax: 0,
    };
    ChooseFontW(&mut cf)
}
