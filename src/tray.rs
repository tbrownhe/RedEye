//! Native notification-area UI. No window procedure borrows application state.

use std::{cell::RefCell, collections::VecDeque, io::Write, mem::size_of, ptr};

use windows_sys::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETFOCUS,
                NIM_SETVERSION, NIN_SELECT, NOTIFYICON_VERSION_4, NOTIFYICONDATAW,
                Shell_NotifyIconW,
            },
            WindowsAndMessaging::*,
        },
    },
    core::w,
};

const CLASS_NAME: *const u16 = w!("RedEye.TrayWindow");
const ICON_ID: u32 = 1;
const EXIT_ID: usize = 1;
const TRAY_MESSAGE: u32 = WM_APP + 1;
const WAKE_MESSAGE: u32 = WM_APP + 2;
const KEY_SELECT: u32 = NIN_SELECT + 1;

#[derive(Clone, Copy)]
enum Action {
    LeftClick,
    ContextMenu,
}

thread_local! {
    // A callback can run inside GetMessage or TrackPopupMenu. Store only small
    // actions here, never references to Tray or PowerRequest. No queue borrow
    // is held across a Win32 call that could reenter the window procedure.
    static ACTIONS: RefCell<VecDeque<Action>> = const { RefCell::new(VecDeque::new()) };
}

pub struct Tray {
    instance: HINSTANCE,
    class_registered: bool,
    window: HWND,
    menu: HMENU,
    icon: HICON,
    icon_registered: bool,
}

impl Tray {
    pub fn new() -> Result<Self, String> {
        let mut tray = Self {
            instance: ptr::null_mut(),
            class_registered: false,
            window: ptr::null_mut(),
            menu: ptr::null_mut(),
            icon: ptr::null_mut(),
            icon_registered: false,
        };
        if let Err(error) = tray.initialize() {
            return match tray.close() {
                Ok(()) => Err(error),
                Err(cleanup) => Err(format!("{error}\nInitialization cleanup: {cleanup}")),
            };
        }
        Ok(tray)
    }

    fn initialize(&mut self) -> Result<(), String> {
        // SAFETY: Pointers are null or valid initialized structures/static
        // NUL-terminated strings. Each owned handle is saved immediately for
        // cleanup on this thread. Win32 copies the registration strings.
        unsafe {
            self.instance = GetModuleHandleW(ptr::null());
            if self.instance.is_null() {
                return Err(os_error("GetModuleHandleW"));
            }
            let class = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: self.instance,
                lpszClassName: CLASS_NAME,
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                return Err(os_error("RegisterClassW"));
            }
            self.class_registered = true;
            // A hidden top-level tool window can also receive TaskbarCreated
            // broadcasts in a later milestone. Never call ShowWindow.
            self.window = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                CLASS_NAME,
                w!("RedEye"),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                self.instance,
                ptr::null(),
            );
            if self.window.is_null() {
                return Err(os_error("CreateWindowExW"));
            }
            self.menu = CreatePopupMenu();
            if self.menu.is_null() {
                return Err(os_error("CreatePopupMenu"));
            }
            for (flags, id, label) in [
                (MF_STRING | MF_GRAYED, 0, w!("RedEye")),
                (MF_SEPARATOR, 0, ptr::null()),
                (MF_STRING, EXIT_ID, w!("Exit")),
            ] {
                if AppendMenuW(self.menu, flags, id, label) == 0 {
                    return Err(os_error("AppendMenuW"));
                }
            }
        }
        self.icon = load_icon(include_bytes!("../assets/redeye-active.ico"))?;
        let mut data = self.icon_data();
        data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        data.uCallbackMessage = TRAY_MESSAGE;
        data.hIcon = self.icon;
        let tooltip: Vec<u16> = "RedEye — Awake".encode_utf16().collect();
        data.szTip[..tooltip.len()].copy_from_slice(&tooltip);
        // SAFETY: data is initialized with its correct size and live handles.
        // Shell_NotifyIcon copies data synchronously.
        unsafe {
            if Shell_NotifyIconW(NIM_ADD, &data) == 0 {
                return Err("Cannot add the RedEye tray icon. Ensure Windows Explorer is running, then restart RedEye.".into());
            }
            self.icon_registered = true;
            data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
            if Shell_NotifyIconW(NIM_SETVERSION, &data) == 0 {
                return Err("Windows rejected notification icon version 4. Restart RedEye after checking Windows Explorer.".into());
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            let mut message = MSG::default();
            // SAFETY: writable MSG, no window filter; this is the creating thread.
            let result = unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) };
            if result == -1 {
                return Err(os_error("GetMessageW"));
            }
            if result == 0 {
                return if message.wParam == 0 {
                    Ok(())
                } else {
                    Err("Tray event dispatch failed; RedEye is shutting down and clearing its power request.".into())
                };
            }
            // SAFETY: GetMessage filled a valid MSG. The callback never accesses
            // the self borrow held by this method.
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            loop {
                let action = ACTIONS.with(|actions| actions.borrow_mut().pop_front());
                match action {
                    Some(Action::LeftClick) => {
                        let _ = writeln!(
                            std::io::stdout(),
                            "RedEye: left activation received; awake state unchanged (milestone 2)."
                        );
                    }
                    Some(Action::ContextMenu) => {
                        let _ = writeln!(
                            std::io::stdout(),
                            "RedEye: right-click/context-menu received; awake state unchanged."
                        );
                        if self.show_menu()? {
                            return Ok(());
                        }
                    }
                    None => break,
                }
            }
        }
    }

    fn show_menu(&self) -> Result<bool, String> {
        let mut point = POINT::default();
        // SAFETY: handles are live and point is writable. The modal menu may
        // reenter window_proc, which queues actions and never borrows self.
        unsafe {
            if GetCursorPos(&mut point) == 0 {
                return Err(os_error("GetCursorPos"));
            }
            // Required for tray menus to dismiss when clicking outside.
            if SetForegroundWindow(self.window) == 0 {
                return Err(
                    "Windows could not focus RedEye's tray menu. Try opening it again.".into(),
                );
            }
            let selected = TrackPopupMenu(
                self.menu,
                TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
                point.x,
                point.y,
                0,
                self.window,
                ptr::null(),
            );
            if PostMessageW(self.window, WM_NULL, 0, 0) == 0 {
                return Err(os_error("PostMessageW after tray menu"));
            }
            if Shell_NotifyIconW(NIM_SETFOCUS, &self.icon_data()) == 0 {
                return Err("Windows could not return focus to the RedEye tray icon. Restart RedEye if the icon is unavailable.".into());
            }
            // TPM_RETURNCMD zero means cancellation OR failure. Win32 documents
            // no reliable way to distinguish those outcomes.
            Ok(selected as usize == EXIT_ID)
        }
    }

    fn icon_data(&self) -> NOTIFYICONDATAW {
        NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.window,
            uID: ICON_ID,
            ..Default::default()
        }
    }

    pub fn close(&mut self) -> Result<(), String> {
        let mut errors = Vec::new();
        // SAFETY: Only live, owned handles are released, on their creating
        // thread. Successful releases reset ownership to prevent double frees.
        unsafe {
            if self.icon_registered {
                if Shell_NotifyIconW(NIM_DELETE, &self.icon_data()) == 0 {
                    errors.push("Windows could not remove the RedEye tray icon; Explorer may have restarted.".into());
                }
                self.icon_registered = false;
            }
            if !self.window.is_null() {
                if DestroyWindow(self.window) == 0 {
                    errors.push(os_error("DestroyWindow"));
                } else {
                    self.window = ptr::null_mut();
                }
            }
            if !self.menu.is_null() {
                if DestroyMenu(self.menu) == 0 {
                    errors.push(os_error("DestroyMenu"));
                } else {
                    self.menu = ptr::null_mut();
                }
            }
            if !self.icon.is_null() {
                if DestroyIcon(self.icon) == 0 {
                    errors.push(os_error("DestroyIcon"));
                } else {
                    self.icon = ptr::null_mut();
                }
            }
            if self.class_registered {
                if UnregisterClassW(CLASS_NAME, self.instance) == 0 {
                    errors.push(os_error("UnregisterClassW"));
                } else {
                    self.class_registered = false;
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        // Normal paths call close explicitly and surface errors before Drop.
        let _ = self.close();
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == TRAY_MESSAGE {
        let event = lparam as u32 & 0xffff;
        let action = match event {
            NIN_SELECT | KEY_SELECT => Some(Action::LeftClick),
            WM_CONTEXTMENU => Some(Action::ContextMenu),
            _ => None,
        };
        if let Some(action) = action {
            let queued = ACTIONS.with(|actions| {
                if let Ok(mut actions) = actions.try_borrow_mut() {
                    actions.push_back(action);
                    true
                } else {
                    false
                }
            });
            // SAFETY: Windows supplies a live callback HWND. Posting wakes the
            // main loop; no queue or application borrow crosses the call.
            if !queued || unsafe { PostMessageW(window, WAKE_MESSAGE, 0, 0) } == 0 {
                unsafe { PostQuitMessage(1) };
            }
        }
        return 0;
    }
    if message == WAKE_MESSAGE {
        return 0;
    }
    if message == WM_CLOSE {
        // Main removes the icon and clears power before the process exits.
        unsafe { PostQuitMessage(0) };
        return 0;
    }
    // SAFETY: Original message parameters go to the default procedure. No
    // Rust application-state pointer is stored in the HWND.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

pub fn report_error(message: &str) {
    let _ = writeln!(std::io::stderr(), "RedEye: {message}");
    let text: Vec<u16> = format!("{message}\n\nRedEye will exit. Restart it to try again.")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    // SAFETY: text is NUL-terminated and alive for the synchronous call. Null
    // parent is valid and allows reporting initialization failures.
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            text.as_ptr(),
            w!("RedEye error"),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn os_error(operation: &str) -> String {
    format!("{operation} failed: {}", std::io::Error::last_os_error())
}

fn load_icon(bytes: &[u8]) -> Result<HICON, String> {
    // SAFETY: GetSystemMetrics takes only an integer index.
    let width = unsafe { GetSystemMetrics(SM_CXSMICON) }.max(16);
    let height = unsafe { GetSystemMetrics(SM_CYSMICON) }.max(16);
    let image = icon_image(bytes, width.max(height) as u32)?;
    // The decoder requires DWORD alignment. Vec<u32> guarantees that;
    // include_bytes! and Vec<u8> promise only byte alignment.
    let aligned: Vec<u32> = image
        .chunks(4)
        .map(|chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            u32::from_le_bytes(word)
        })
        .collect();
    // SAFETY: ICO entry bounds were checked, buffer is aligned and remains
    // alive throughout synchronous decoding. No LR_SHARED: we own the HICON.
    let icon = unsafe {
        CreateIconFromResourceEx(
            aligned.as_ptr().cast(),
            image.len() as u32,
            1,
            0x0003_0000,
            width,
            height,
            LR_DEFAULTCOLOR,
        )
    };
    if icon.is_null() {
        Err(os_error("CreateIconFromResourceEx"))
    } else {
        Ok(icon)
    }
}

fn icon_image(bytes: &[u8], size: u32) -> Result<&[u8], String> {
    let invalid = || "The embedded RedEye icon has an invalid ICO directory.".to_string();
    if bytes.get(..4) != Some(&[0, 0, 1, 0]) {
        return Err(invalid());
    }
    let count_bytes = bytes.get(4..6).ok_or_else(invalid)?;
    let count = u16::from_le_bytes([count_bytes[0], count_bytes[1]]) as usize;
    let directory_end = 6 + count * 16;
    let directory = bytes.get(6..directory_end).ok_or_else(invalid)?;
    let mut best: Option<(&[u8], u32)> = None;
    for entry in directory.as_chunks::<16>().0 {
        let width = if entry[0] == 0 { 256 } else { entry[0] as u32 };
        let height = if entry[1] == 0 { 256 } else { entry[1] as u32 };
        let len = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
        let offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;
        let end = offset.checked_add(len).ok_or_else(invalid)?;
        if offset < directory_end || len == 0 {
            return Err(invalid());
        }
        let image = bytes.get(offset..end).ok_or_else(invalid)?;
        // Prefer a larger source over upscaling; then choose closest dimensions.
        let score = width.abs_diff(size)
            + height.abs_diff(size)
            + if width < size || height < size {
                1024
            } else {
                0
            };
        if best.is_none_or(|(_, previous)| score < previous) {
            best = Some((image, score));
        }
    }
    best.map(|(image, _)| image).ok_or_else(invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_icon_has_valid_frames_and_corrupt_bounds_are_rejected() {
        let bytes = include_bytes!("../assets/redeye-active.ico");
        for size in [16, 20, 24, 32, 48, 64, 128, 256] {
            assert!(!icon_image(bytes, size).unwrap().is_empty());
        }
        assert!(icon_image(&[], 16).is_err());
        assert!(icon_image(&bytes[..20], 16).is_err());
        let mut corrupt = bytes.to_vec();
        corrupt[18..22].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(icon_image(&corrupt, 16).is_err());
    }
}
