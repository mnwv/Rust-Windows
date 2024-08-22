use std::arch::x86_64::__cpuid;
use std::ptr::slice_from_raw_parts;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{CreateFontIndirectW, DPtoLP, DeleteObject, GetDeviceCaps, GetTextMetricsW, ModifyWorldTransform, RestoreDC, SaveDC, SelectObject, SetGraphicsMode, SetViewportOrgEx, SetWindowOrgEx, DEFAULT_CHARSET, FONT_CLIP_PRECISION, FONT_OUTPUT_PRECISION, FONT_QUALITY, GM_ADVANCED, HDC, HFONT, HGDIOBJ, HORZRES, HORZSIZE, LOGFONTW, LOGPIXELSX, LOGPIXELSY, MWT_IDENTITY, TEXTMETRICW, VERTRES, VERTSIZE};

pub const EZ_ATTR_BOLD: i32 = 1;
pub const EZ_ATTR_ITALIC: i32 = 2;
pub const EZ_ATTR_UNDERLINE: i32 = 4;
pub const EZ_ATTR_STRIKEOUT: i32 = 8;

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

        let mut lf: LOGFONTW = LOGFONTW {
            lfHeight: -(pt[0].y.abs() as f32 / 10.0 + 0.5) as i32,
            lfWidth: 0,
            lfEscapement: 0,
            lfOrientation: 0,
            lfWeight: if attr & EZ_ATTR_BOLD == EZ_ATTR_BOLD { 700 } else { 0 },
            lfItalic: if attr & EZ_ATTR_ITALIC == EZ_ATTR_ITALIC { 1 } else { 0 },
            lfUnderline: if attr & EZ_ATTR_UNDERLINE == EZ_ATTR_UNDERLINE { 1 } else { 0 },
            lfStrikeOut: if attr & EZ_ATTR_STRIKEOUT == EZ_ATTR_STRIKEOUT { 1 } else { 0 },
            lfCharSet: DEFAULT_CHARSET,
            lfOutPrecision: FONT_OUTPUT_PRECISION(0),
            lfQuality: FONT_QUALITY(0),
            lfPitchAndFamily: 0,
            lfClipPrecision: FONT_CLIP_PRECISION(0),
            lfFaceName: {
                let mut v: Vec<u16> = face_name.encode_utf16().collect();
                while v.len() < 32 { v.push(0); }
                v.iter().collect()
            }
        };
        let mut font = CreateFontIndirectW(&lf);
        if deci_pt_width != 0 {
            font = SelectObject(hdc, font);
            let mut tm: TEXTMETRICW = std::mem::zeroed();
            GetTextMetricsW(hdc, &mut tm);
            DeleteObject(SelectObject(hdc, font).into());
            lf.lfWidth = tm.tmAveCharWidth * pt[0].x.abs() / pt[0].y.abs(); // +0.5
            font = CreateFontIndirectW(&lf);
        }
        RestoreDC(hdc, -1);
        font
    }
    // HFONT(std::ptr::null_mut())
}