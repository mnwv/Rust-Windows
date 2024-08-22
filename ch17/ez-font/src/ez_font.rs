use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{DPtoLP, GetDeviceCaps, ModifyWorldTransform, SaveDC, SetGraphicsMode, SetViewportOrgEx, SetWindowOrgEx, GM_ADVANCED, HDC, HFONT, HORZRES, HORZSIZE, LOGFONTW, LOGPIXELSX, LOGPIXELSY, MWT_IDENTITY, VERTRES, VERTSIZE};

pub const EZ_ATTR_BOLD: usize = 1;
pub const EZ_ATTR_ITALIC: usize = 2;
pub const EZ_ATTR_UNDERLINE: usize = 4;
pub const EZ_ATTR_STRIKEFONT: usize = 8;

pub fn ez_create_font(
    hdc: HDC,
    face_name: &str,
    deci_pt_hight:i32,
    deci_pt_width: i32,
    attr: i32,
    log_res: bool) -> HFONT {
    unsafe {
        SaveDC(hdc);
        SetGraphicsMode(hdc, GM_ADVANCED);
        ModifyWorldTransform(hdc, None, MWT_IDENTITY);
        SetViewportOrgEx(hdc, 0, 0, None);
        SetWindowOrgEx(hdc, 0, 0, None);
        let (cx_dpi, cy_dpi) =
        if log_res {
            (GetDeviceCaps(hdc, LOGPIXELSX) as f32,
            GetDeviceCaps(hdc, LOGPIXELSY) as f32)
        } else {
            (25.4 * (GetDeviceCaps(hdc, HORZRES) / GetDeviceCaps(hdc, HORZSIZE)) as f32,
            25.4 * (GetDeviceCaps(hdc, VERTRES) / GetDeviceCaps(hdc, VERTSIZE)) as f32)
        };
        let mut pt:[POINT;1] = [POINT {
            x: deci_pt_width * (cx_dpi / 72.0) as i32,
            y: deci_pt_hight * (cy_dpi / 72.0) as i32,
        }];
        DPtoLP(hdc, &mut pt);

        let lf: LOGFONTW = LOGFONTW {
            lfHeight: fabs
        }
    }
    HFONT(std::ptr::null_mut())
}