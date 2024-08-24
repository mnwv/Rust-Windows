mod resource_id;

use windows::{
    core::*,
    Win32::UI::WindowsAndMessaging::*,
};
use crate::resource_id::*;

pub fn create_menu() -> Result<HMENU> {
    unsafe {
        let menu = CreateMenu()?;
        let submenu = CreatePopupMenu()?;
        AppendMenuW(submenu, MF_STRING, IDM_FILE_NEW, w!("新規(&N)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_FILE_OPEN, w!("開く(&O)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_FILE_SAVE, w!("保存(&S"))?;
        AppendMenuW(submenu, MF_STRING, IDM_FILE_SAVE_AS, w!("名前を付けて保存(&A)..."))?;
        AppendMenuW(submenu, MF_SEPARATOR, 0, PCWSTR(std::ptr::null()))?;
        AppendMenuW(submenu, MF_STRING, IDM_APP_EXIT, w!("終了(&X)"))?;
        AppendMenuW(menu, MF_STRING | MF_POPUP, submenu.0 as usize, w!("ファイル(&F)"))?;

        let submenu = CreatePopupMenu()?;
        AppendMenuW(submenu, MF_STRING, IDM_EDIT_UNDO, w!("元に戻す(&U)"))?;
        AppendMenuW(submenu, MF_SEPARATOR, 0, PCWSTR(std::ptr::null()))?;
        AppendMenuW(submenu, MF_STRING, IDM_EDIT_CUT, w!("切り取り(&T)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_EDIT_COPY, w!("コピー(&C)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_EDIT_PASTE, w!("貼り付け(&P)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_EDIT_CLEAR, w!("削除(&L)"))?;
        AppendMenuW(menu, MF_STRING | MF_POPUP, submenu.0 as usize, w!("編集(&E)"))?;

        let submenu = CreatePopupMenu()?;
        AppendMenuW(submenu, MF_STRING | MF_CHECKED, IDM_BKGND_WHITE, w!("White(&W)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_BKGND_LTGRAY, w!("Light Gray(&L)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_BKGND_GRAY, w!("Gray(&G)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_BKGND_DKGRAY, w!("Dark Gray(&D)"))?;
        AppendMenuW(submenu, MF_STRING, IDM_BKGND_BLACK, w!("Black(&B)"))?;
        AppendMenuW(menu, MF_STRING | MF_POPUP, submenu.0 as usize, w!("背景(&B)"))?;

        let submenu = CreatePopupMenu()?;
        AppendMenuW(submenu, MF_STRING, IDM_TIMER_START, w!("開始(&S)"))?;
        AppendMenuW(submenu, MF_STRING | MF_GRAYED, IDM_TIMER_STOP, w!("停止(&T)"))?;
        AppendMenuW(menu, MF_STRING | MF_POPUP, submenu.0 as usize, w!("タイマー(&T)"))?;

        let submenu = CreatePopupMenu()?;
        AppendMenuW(submenu, MF_STRING, IDM_APP_HELP, w!("ヘルプ(&H)..."))?;
        AppendMenuW(submenu, MF_STRING, IDM_APP_ABOUT, w!("Menu Demo2について(&A)..."))?;
        AppendMenuW(menu, MF_STRING | MF_POPUP, submenu.0 as usize, w!("ヘルプ(&H)"))?;

        Ok(menu)
    }
}