use crate::WindowsHostConfig;
use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use std::mem::{size_of, zeroed};
use std::process::Command;
use std::ptr::{null, null_mut};
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION,
    NOTIFYICONDATAW, NOTIFYICON_VERSION_4,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, GetMessageW, LoadIconW, MessageBoxW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, TrackPopupMenu, TranslateMessage, HMENU, HWND_MESSAGE, IDI_APPLICATION,
    MB_ICONINFORMATION, MB_OK, MF_SEPARATOR, MF_STRING, MSG, TPM_RIGHTBUTTON, WM_COMMAND,
    WM_DESTROY, WM_LBUTTONDBLCLK, WM_RBUTTONUP, WM_USER, WNDCLASSW, WS_OVERLAPPED,
};

const WM_TRAY_ICON: u32 = WM_USER + 42;
const TRAY_UID: u32 = 4510;
const MENU_OPEN_UI: usize = 1001;
const MENU_HEALTH: usize = 1002;
const MENU_COPY_DIAGNOSTICS: usize = 1003;
const MENU_EXIT: usize = 1004;
const CF_UNICODETEXT: u32 = 13;

static TRAY_CONTEXT: OnceLock<TrayContext> = OnceLock::new();

struct TrayContext {
    base_url: String,
}

pub fn run(config: WindowsHostConfig) -> Result<()> {
    let base_url = format!("http://127.0.0.1:{}", config.host_port);
    let _ = TRAY_CONTEXT.set(TrayContext { base_url });

    unsafe {
        let instance = GetModuleHandleW(null());
        let class_name = wide("YianWindowsHostTrayWindow");
        let window_title = wide("Yian Windows Host Tray");
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            ..zeroed()
        };
        if RegisterClassW(&class) == 0 {
            return Err(anyhow!("failed to register tray window class"));
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            0 as HMENU,
            instance,
            null(),
        );
        if hwnd == null_mut() {
            return Err(anyhow!("failed to create tray message window"));
        }

        add_tray_icon(hwnd)?;
        println!("tray running; right-click the Yian Windows Host icon for local controls");

        let mut message: MSG = zeroed();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        delete_tray_icon(hwnd);
    }

    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_TRAY_ICON => match lparam as u32 {
            WM_RBUTTONUP => {
                show_menu(hwnd);
                0
            }
            WM_LBUTTONDBLCLK => {
                open_management_page();
                0
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        },
        WM_COMMAND => {
            handle_menu_command(wparam & 0xffff);
            0
        }
        WM_DESTROY => {
            delete_tray_icon(hwnd);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn add_tray_icon(hwnd: HWND) -> Result<()> {
    let mut data: NOTIFYICONDATAW = zeroed();
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_UID;
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = WM_TRAY_ICON;
    data.hIcon = LoadIconW(null_mut(), IDI_APPLICATION);
    write_tip(&mut data.szTip, "Yian Windows Host");

    if Shell_NotifyIconW(NIM_ADD, &data) == 0 {
        return Err(anyhow!("failed to add tray icon"));
    }
    data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
    Shell_NotifyIconW(NIM_SETVERSION, &data);
    Ok(())
}

unsafe fn delete_tray_icon(hwnd: HWND) {
    let mut data: NOTIFYICONDATAW = zeroed();
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_UID;
    Shell_NotifyIconW(NIM_DELETE, &data);
}

unsafe fn show_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    if menu == null_mut() {
        return;
    }
    append_menu_item(menu, MENU_OPEN_UI, "打开本地管理页");
    append_menu_item(menu, MENU_HEALTH, "查看健康状态");
    append_menu_item(menu, MENU_COPY_DIAGNOSTICS, "复制诊断信息");
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    append_menu_item(menu, MENU_EXIT, "退出");

    let mut point: POINT = zeroed();
    if GetCursorPos(&mut point) != 0 {
        SetForegroundWindow(hwnd);
        TrackPopupMenu(menu, TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, null());
    }
    DestroyMenu(menu);
}

unsafe fn append_menu_item(menu: HMENU, id: usize, label: &str) {
    let text = wide(label);
    AppendMenuW(menu, MF_STRING, id, text.as_ptr());
}

fn handle_menu_command(command: usize) {
    match command {
        MENU_OPEN_UI => open_management_page(),
        MENU_HEALTH => open_health_page(),
        MENU_COPY_DIAGNOSTICS => match copy_diagnostics() {
            Ok(()) => show_message("诊断信息已复制到剪贴板。"),
            Err(error) => show_message(&format!("复制诊断信息失败：{error}")),
        },
        MENU_EXIT => request_shutdown(),
        _ => {}
    }
}

fn open_management_page() {
    if let Some(context) = TRAY_CONTEXT.get() {
        open_url(&format!("{}/ui", context.base_url));
    }
}

fn open_health_page() {
    if let Some(context) = TRAY_CONTEXT.get() {
        open_url(&format!("{}/health", context.base_url));
    }
}

fn request_shutdown() {
    if let Some(context) = TRAY_CONTEXT.get() {
        let url = format!("{}/shutdown", context.base_url);
        if let Err(error) = Client::new().post(url).send() {
            eprintln!("failed to request local shutdown from tray: {error}");
        }
    }
    unsafe {
        PostQuitMessage(0);
    }
}

fn copy_diagnostics() -> Result<()> {
    let context = TRAY_CONTEXT
        .get()
        .ok_or_else(|| anyhow!("tray context was not initialized"))?;
    let diagnostics = Client::new()
        .get(format!("{}/diagnostics", context.base_url))
        .send()
        .context("failed to call /diagnostics")?
        .text()
        .context("failed to read diagnostics response")?;
    copy_text_to_clipboard(&diagnostics)
}

fn open_url(url: &str) {
    if let Err(error) = Command::new("cmd").args(["/C", "start", "", url]).spawn() {
        eprintln!("failed to open {url}: {error}");
    }
}

fn copy_text_to_clipboard(text: &str) -> Result<()> {
    let wide_text = wide(text);
    let byte_len = wide_text.len() * size_of::<u16>();
    unsafe {
        let handle = GlobalAlloc(GMEM_MOVEABLE, byte_len);
        if handle == null_mut() {
            return Err(anyhow!("GlobalAlloc failed"));
        }
        let ptr = GlobalLock(handle) as *mut u16;
        if ptr.is_null() {
            return Err(anyhow!("GlobalLock failed"));
        }
        ptr.copy_from_nonoverlapping(wide_text.as_ptr(), wide_text.len());
        GlobalUnlock(handle);

        if OpenClipboard(null_mut()) == 0 {
            return Err(anyhow!("OpenClipboard failed"));
        }
        EmptyClipboard();
        if SetClipboardData(CF_UNICODETEXT, handle) == null_mut() {
            CloseClipboard();
            return Err(anyhow!("SetClipboardData failed"));
        }
        CloseClipboard();
    }
    Ok(())
}

fn show_message(message: &str) {
    let title = wide("Yian Windows Host");
    let body = wide(message);
    unsafe {
        MessageBoxW(
            null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn write_tip(target: &mut [u16; 128], text: &str) {
    let source = wide(text);
    for (slot, value) in target.iter_mut().zip(source.iter()) {
        *slot = *value;
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
