use super::*;
use std::cell::Cell;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetDlgItem,
    GetForegroundWindow, GetMessageW, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW,
    IsDialogMessageW, IsWindow, LoadCursorW, MessageBoxW, RegisterClassW, SetForegroundWindow,
    SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow, TranslateMessage, CREATESTRUCTW,
    CW_USEDEFAULT, GWLP_USERDATA, HMENU, HWND_NOTOPMOST, HWND_TOPMOST, IDC_ARROW,
    MB_ICONINFORMATION, MB_OK, MSG, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_NCCREATE,
    WNDCLASSW, WS_BORDER, WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN, WS_EX_CONTROLPARENT,
    WS_EX_DLGMODALFRAME, WS_EX_WINDOWEDGE, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

const BS_DEFPUSHBUTTON: WINDOW_STYLE = 0x0000_0001;
const BS_PUSHBUTTON: WINDOW_STYLE = 0x0000_0000;
static NATIVE_OBJECT_EDITOR_OPEN: AtomicBool = AtomicBool::new(false);

pub(crate) fn set_core_status(core: &StoryLockCoreApp, result: Result<()>, success_message: &str) {
    match result {
        Ok(()) => {
            core.set_config_status(SharedString::from(success_message));
            core.set_export_preview(SharedString::from(build_export_preview(Path::new(
                core.get_core_data_dir().as_str(),
            ))));
        }
        Err(error) => core.set_config_status(SharedString::from(format!("Save failed: {error}"))),
    }
}

pub(crate) fn open_answer_editor_dialog(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    answer_editor: Rc<RefCell<Option<AnswerEditorDialog>>>,
) {
    if answer_editor.borrow().is_none() {
        match AnswerEditorDialog::new() {
            Ok(dialog) => {
                wire_answer_editor_callbacks(&dialog, core.as_weak(), package_dir.to_path_buf());
                *answer_editor.borrow_mut() = Some(dialog);
            }
            Err(error) => {
                core.set_config_status(SharedString::from(format!(
                    "Answer editor failed to open: {error}"
                )));
                return;
            }
        }
    }

    if let Some(dialog) = answer_editor.borrow().as_ref() {
        copy_core_question_to_answer_editor(core, dialog);
        if let Err(error) = dialog.show() {
            core.set_config_status(SharedString::from(format!(
                "Answer editor failed to show: {error}"
            )));
        }
    }
}

pub(crate) fn open_storylock_core_settings_dialog(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>>,
) {
    if settings_dialog.borrow().is_none() {
        match StoryLockCoreSettingsDialog::new() {
            Ok(dialog) => {
                wire_storylock_core_settings_callbacks(
                    &dialog,
                    core.as_weak(),
                    package_dir.to_path_buf(),
                    Rc::clone(&settings_dialog),
                );
                *settings_dialog.borrow_mut() = Some(dialog);
            }
            Err(error) => {
                core.set_config_status(SharedString::from(format!(
                    "Settings failed to open: {error}"
                )));
                return;
            }
        }
    }

    if let Some(dialog) = settings_dialog.borrow().as_ref() {
        copy_core_settings_to_dialog(core, dialog);
        if let Err(error) = dialog.show() {
            core.set_config_status(SharedString::from(format!(
                "Settings failed to show: {error}"
            )));
        }
    }
}

pub(crate) fn open_object_editor_dialog(
    core: &StoryLockCoreApp,
    package_dir: &Path,
    object_editor: Rc<RefCell<Option<ObjectEditorDialog>>>,
) {
    let _ = object_editor;
    if NATIVE_OBJECT_EDITOR_OPEN.swap(true, Ordering::SeqCst) {
        return;
    }
    match prompt_managed_object_with_native_dialog(
        core.get_language().as_str(),
        core.get_display_name().as_str(),
        core.get_provider_id().as_str(),
        core.get_secret_reference().as_str(),
    ) {
        Ok(Some(result)) => match result.action {
            NativeManagedObjectDialogAction::Save => {
                core.set_display_name(SharedString::from(result.uri.as_str()));
                core.set_provider_id(SharedString::from(result.username.as_str()));
                core.set_secret_reference(SharedString::from(result.password.as_str()));
                if core.get_object_kind().trim().is_empty() {
                    core.set_object_kind(SharedString::from("password_fill"));
                }
                if core.get_resource_group().trim().is_empty() {
                    core.set_resource_group(SharedString::from("normal"));
                }
                set_core_status(
                    core,
                    save_object_editor_resource_from_window(core, package_dir),
                    "Managed object saved.",
                );
            }
            NativeManagedObjectDialogAction::Delete => {
                set_core_status(
                    core,
                    delete_object_editor_resource_from_window(core, package_dir),
                    "Managed object deleted.",
                );
            }
        },
        Ok(None) => {}
        Err(error) => {
            core.set_config_status(SharedString::from(format!(
                "Object editor failed to open: {error}"
            )));
        }
    }
    NATIVE_OBJECT_EDITOR_OPEN.store(false, Ordering::SeqCst);
}

enum NativeManagedObjectDialogAction {
    Save,
    Delete,
}

struct NativeManagedObjectDialogResult {
    action: NativeManagedObjectDialogAction,
    uri: String,
    username: String,
    password: String,
}

fn prompt_managed_object_with_native_dialog(
    language: &str,
    uri: &str,
    username: &str,
    password: &str,
) -> Result<Option<NativeManagedObjectDialogResult>> {
    let labels = NativeManagedObjectDialogLabels::from_language(language);
    native_managed_object_dialog(uri, username, password, &labels)
}

const ID_URI: i32 = 1001;
const ID_USERNAME: i32 = 1002;
const ID_PASSWORD: i32 = 1003;
const ID_SAVE: i32 = 1004;
const ID_CLOSE: i32 = 1005;
const ID_DELETE: i32 = 1006;

struct NativeManagedObjectDialogLabels {
    title: &'static str,
    username: &'static str,
    password: &'static str,
    save: &'static str,
    delete: &'static str,
    close: &'static str,
}

impl NativeManagedObjectDialogLabels {
    fn from_language(language: &str) -> Self {
        if language == "zh" {
            Self {
                title: "\u{53d7}\u{63a7}\u{5bf9}\u{8c61}\u{7f16}\u{8f91}",
                username: "\u{7528}\u{6237}\u{540d}",
                password: "\u{5bc6}\u{7801}",
                save: "\u{4fdd}\u{5b58}",
                delete: "\u{5220}\u{9664}",
                close: "\u{5173}\u{95ed}",
            }
        } else {
            Self {
                title: "Managed Object Editor",
                username: "Username",
                password: "Password",
                save: "Save",
                delete: "Delete",
                close: "Close",
            }
        }
    }
}

struct NativeManagedObjectDialogState {
    initial_uri: String,
    initial_username: String,
    initial_password: String,
    labels: NativeManagedObjectDialogLabels,
    owner: HWND,
    result: Option<NativeManagedObjectDialogResult>,
}

fn dialog_background_brush() -> HBRUSH {
    unsafe { CreateSolidBrush(0x00F5F3EE) }
}

fn native_managed_object_dialog(
    uri: &str,
    username: &str,
    password: &str,
    labels: &NativeManagedObjectDialogLabels,
) -> Result<Option<NativeManagedObjectDialogResult>> {
    let class_name = wide_null("StoryLockManagedObjectDialog");
    let title = wide_null(labels.title);
    let state = Box::new(NativeManagedObjectDialogState {
        initial_uri: uri.to_string(),
        initial_username: username.to_string(),
        initial_password: password.to_string(),
        labels: NativeManagedObjectDialogLabels {
            title: labels.title,
            username: labels.username,
            password: labels.password,
            save: labels.save,
            delete: labels.delete,
            close: labels.close,
        },
        owner: 0 as HWND,
        result: None,
    });
    let state_ptr = Box::into_raw(state);

    unsafe {
        let instance = GetModuleHandleW(null());
        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(native_managed_object_dialog_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: dialog_background_brush(),
            ..std::mem::zeroed()
        };
        RegisterClassW(&wnd_class);
        let owner = GetForegroundWindow();
        (*state_ptr).owner = owner;

        let hwnd = CreateWindowExW(
            (WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CONTROLPARENT) as WINDOW_EX_STYLE,
            class_name.as_ptr(),
            title.as_ptr(),
            (WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE | WS_CLIPCHILDREN)
                as WINDOW_STYLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            680,
            250,
            owner,
            null_mut(),
            instance,
            state_ptr.cast(),
        );
        if hwnd.is_null() {
            let _ = Box::from_raw(state_ptr);
            return Err(anyhow::anyhow!(
                "native object editor window creation failed"
            ));
        }

        if !owner.is_null() {
            EnableWindow(owner, 0);
        }
        SetWindowTextW(hwnd, title.as_ptr());
        ShowWindow(hwnd, SW_SHOW);
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
        SetForegroundWindow(hwnd);
        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        );
        let mut msg: MSG = std::mem::zeroed();
        while IsWindow(hwnd) != 0 {
            let status = GetMessageW(&mut msg, null_mut(), 0, 0);
            if status == -1 {
                if !owner.is_null() {
                    EnableWindow(owner, 1);
                    SetForegroundWindow(owner);
                }
                let _ = Box::from_raw(state_ptr);
                return Err(anyhow::anyhow!("native object editor message loop failed"));
            }
            if status == 0 {
                break;
            }
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if IsWindow(hwnd) == 0 {
                break;
            }
        }

        if !owner.is_null() {
            EnableWindow(owner, 1);
            SetForegroundWindow(owner);
        }
        let state = Box::from_raw(state_ptr);
        Ok(state.result)
    }
}

unsafe extern "system" fn native_managed_object_dialog_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create = &*(lparam as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
            1
        }
        WM_CREATE => {
            init_native_managed_object_dialog_controls(hwnd);
            0
        }
        WM_COMMAND => {
            let control_id = (wparam & 0xffff) as i32;
            match control_id {
                ID_SAVE => {
                    save_native_managed_object_dialog(hwnd);
                    0
                }
                ID_CLOSE => {
                    DestroyWindow(hwnd);
                    0
                }
                ID_DELETE => {
                    delete_native_managed_object_dialog(hwnd);
                    0
                }
                _ => DefWindowProcW(hwnd, message, wparam, lparam),
            }
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => 0,
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

unsafe fn init_native_managed_object_dialog_controls(hwnd: HWND) {
    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NativeManagedObjectDialogState;
    if state_ptr.is_null() {
        return;
    }
    let state = &*state_ptr;

    let uri_label = if state.labels.title == "\u{53d7}\u{63a7}\u{5bf9}\u{8c61}\u{7f16}\u{8f91}" {
        "\u{5bf9}\u{8c61}\u{540d}\u{79f0}"
    } else {
        "URI"
    };
    create_static(hwnd, uri_label, 24, 26, 100, 24, 0);
    create_edit(hwnd, ID_URI, &state.initial_uri, 130, 22, 500, 24, false);
    create_static(hwnd, state.labels.username, 24, 66, 100, 24, 0);
    create_edit(
        hwnd,
        ID_USERNAME,
        &state.initial_username,
        130,
        62,
        500,
        24,
        false,
    );
    create_static(hwnd, state.labels.password, 24, 106, 100, 24, 0);
    create_edit(
        hwnd,
        ID_PASSWORD,
        &state.initial_password,
        130,
        102,
        500,
        24,
        false,
    );
    create_button(hwnd, state.labels.save, ID_SAVE, 280, 152, 100, 30, true);
    create_button(hwnd, state.labels.close, ID_CLOSE, 400, 152, 100, 30, false);
    create_button(
        hwnd,
        state.labels.delete,
        ID_DELETE,
        520,
        152,
        100,
        30,
        false,
    );
}

unsafe fn create_static(
    parent: HWND,
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) {
    let class = wide_null("STATIC");
    let text = wide_null(text);
    CreateWindowExW(
        0,
        class.as_ptr(),
        text.as_ptr(),
        (WS_CHILD | WS_VISIBLE) as WINDOW_STYLE,
        x,
        y,
        width,
        height,
        parent,
        id as HMENU,
        null_mut(),
        null_mut(),
    );
}

unsafe fn create_edit(
    parent: HWND,
    id: i32,
    value: &str,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    _password: bool,
) {
    let class = wide_null("EDIT");
    let value = wide_null(value);
    CreateWindowExW(
        0,
        class.as_ptr(),
        value.as_ptr(),
        (WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER) as WINDOW_STYLE,
        x,
        y,
        width,
        height,
        parent,
        id as HMENU,
        null_mut(),
        null_mut(),
    );
}

unsafe fn create_button(
    parent: HWND,
    text: &str,
    id: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    tabstop: bool,
) {
    let class = wide_null("BUTTON");
    let text = wide_null(text);
    let mut style = (WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON) as WINDOW_STYLE;
    if tabstop {
        style |= WS_TABSTOP as WINDOW_STYLE | BS_DEFPUSHBUTTON;
    }
    CreateWindowExW(
        0,
        class.as_ptr(),
        text.as_ptr(),
        style,
        x,
        y,
        width,
        height,
        parent,
        id as HMENU,
        null_mut(),
        null_mut(),
    );
}

unsafe fn save_native_managed_object_dialog(hwnd: HWND) {
    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NativeManagedObjectDialogState;
    if state_ptr.is_null() {
        DestroyWindow(hwnd);
        return;
    }
    (*state_ptr).result = Some(NativeManagedObjectDialogResult {
        action: NativeManagedObjectDialogAction::Save,
        uri: get_window_text(GetDlgItem(hwnd, ID_URI)),
        username: get_window_text(GetDlgItem(hwnd, ID_USERNAME)),
        password: get_window_text(GetDlgItem(hwnd, ID_PASSWORD)),
    });
    DestroyWindow(hwnd);
}

unsafe fn delete_native_managed_object_dialog(hwnd: HWND) {
    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NativeManagedObjectDialogState;
    if state_ptr.is_null() {
        DestroyWindow(hwnd);
        return;
    }
    (*state_ptr).result = Some(NativeManagedObjectDialogResult {
        action: NativeManagedObjectDialogAction::Delete,
        uri: get_window_text(GetDlgItem(hwnd, ID_URI)),
        username: get_window_text(GetDlgItem(hwnd, ID_USERNAME)),
        password: get_window_text(GetDlgItem(hwnd, ID_PASSWORD)),
    });
    DestroyWindow(hwnd);
}

unsafe fn get_window_text(hwnd: HWND) -> String {
    if hwnd.is_null() {
        return String::new();
    }
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return String::new();
    }
    let mut buffer = vec![0u16; len as usize + 1];
    let written = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
    String::from_utf16_lossy(&buffer[..written as usize])
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn open_learning_test_dialog(
    core: &StoryLockCoreApp,
    learning_dialog: Rc<RefCell<Option<LearningTestDialog>>>,
) {
    if let Some(dialog) = learning_dialog.borrow().as_ref() {
        copy_core_learning_to_dialog(core, dialog);
        dialog.window().request_redraw();
        return;
    }

    let new_modal_owner: Rc<Cell<HWND>>;
    match LearningTestDialog::new() {
        Ok(dialog) => {
            let modal_owner = Rc::new(Cell::new(null_mut()));
            wire_learning_test_dialog_callbacks(
                &dialog,
                core.as_weak(),
                Rc::clone(&learning_dialog),
                Rc::clone(&modal_owner),
            );
            *learning_dialog.borrow_mut() = Some(dialog);
            new_modal_owner = modal_owner;
        }
        Err(error) => {
            core.set_config_status(SharedString::from(format!(
                "Learning dialog failed to open: {error}"
            )));
            return;
        }
    }

    if let Some(dialog) = learning_dialog.borrow().as_ref() {
        copy_core_learning_to_dialog(core, dialog);
        let owner = unsafe { GetForegroundWindow() };
        new_modal_owner.set(owner);
        if !owner.is_null() {
            unsafe {
                EnableWindow(owner, 0);
            }
        }
        if let Err(error) = dialog.show() {
            restore_learning_modal_owner(&new_modal_owner);
            core.set_config_status(SharedString::from(format!(
                "Learning dialog failed to show: {error}"
            )));
        }
        dialog.window().request_redraw();
    }
}

pub(crate) fn wire_learning_test_dialog_callbacks(
    dialog: &LearningTestDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    learning_dialog: Rc<RefCell<Option<LearningTestDialog>>>,
    modal_owner: Rc<Cell<HWND>>,
) {
    let weak = dialog.as_weak();
    let core_for_restart = core_weak.clone();
    let restart_slot = Rc::clone(&learning_dialog);
    let modal_owner_for_restart = Rc::clone(&modal_owner);
    dialog.on_restart_learning(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_restart.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            let _ = dialog.hide();
            restore_learning_modal_owner(&modal_owner_for_restart);
            *restart_slot.borrow_mut() = None;
            core.invoke_run_learning();
        }
    });

    let weak = dialog.as_weak();
    let core_for_previous = core_weak.clone();
    dialog.on_learning_previous(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_previous.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            core.invoke_learning_previous();
        }
    });

    let weak = dialog.as_weak();
    let core_for_next = core_weak.clone();
    let next_slot = Rc::clone(&learning_dialog);
    let modal_owner_for_next = Rc::clone(&modal_owner);
    dialog.on_learning_next(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_next.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            core.invoke_learning_next();
            if core.get_export_ready() {
                show_learning_passed_message();
                let _ = dialog.hide();
                restore_learning_modal_owner(&modal_owner_for_next);
                *next_slot.borrow_mut() = None;
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_toggle = core_weak.clone();
    dialog.on_learning_toggle_answer(move |index| {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_toggle.upgrade()) {
            copy_dialog_learning_to_core(&dialog, &core);
            toggle_learning_answer_state(&core, index.max(0) as usize);
            copy_core_learning_to_dialog(&core, &dialog);
        }
    });

    let weak = dialog.as_weak();
    let close_slot_for_button = Rc::clone(&learning_dialog);
    let modal_owner_for_button = Rc::clone(&modal_owner);
    dialog.on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            let _ = dialog.hide();
        }
        restore_learning_modal_owner(&modal_owner_for_button);
        *close_slot_for_button.borrow_mut() = None;
    });

    let weak = dialog.as_weak();
    let close_slot_for_window = learning_dialog;
    dialog.window().on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            let _ = dialog.hide();
        }
        restore_learning_modal_owner(&modal_owner);
        *close_slot_for_window.borrow_mut() = None;
        slint::CloseRequestResponse::HideWindow
    });
}

fn restore_learning_modal_owner(modal_owner: &Cell<HWND>) {
    let owner = modal_owner.replace(null_mut());
    if !owner.is_null() && unsafe { IsWindow(owner) } != 0 {
        unsafe {
            EnableWindow(owner, 1);
            SetForegroundWindow(owner);
        }
    }
}

fn show_learning_passed_message() {
    let title = wide_null("九宫格测试");
    let message = wide_null("测试已通过，可以导出。");
    unsafe {
        MessageBoxW(
            GetForegroundWindow(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

pub(crate) fn wire_storylock_core_settings_callbacks(
    dialog: &StoryLockCoreSettingsDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    package_dir: std::path::PathBuf,
    settings_dialog: Rc<RefCell<Option<StoryLockCoreSettingsDialog>>>,
) {
    let close_slot = Rc::clone(&settings_dialog);
    let core_for_close = core_weak.clone();
    let close_settings = Rc::new(move |dialog: &StoryLockCoreSettingsDialog| {
        if let Some(core) = core_for_close.upgrade() {
            copy_dialog_settings_to_core(dialog, &core);
            if let Err(error) = save_storylock_ui_settings(&settings_from_storylock_core(&core)) {
                core.set_config_status(SharedString::from(format!(
                    "Settings save failed: {error}"
                )));
            }
        }
        let _ = dialog.hide();
        *close_slot.borrow_mut() = None;
    });

    let weak = dialog.as_weak();
    let close_settings_for_button = Rc::clone(&close_settings);
    dialog.on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            close_settings_for_button(&dialog);
        }
    });

    let weak = dialog.as_weak();
    let close_settings_for_window = Rc::clone(&close_settings);
    dialog.window().on_close_requested(move || {
        if let Some(dialog) = weak.upgrade() {
            close_settings_for_window(&dialog);
        }
        slint::CloseRequestResponse::HideWindow
    });

    let weak = dialog.as_weak();
    let core_for_language = core_weak.clone();
    dialog.on_language_changed(move |language| {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_language.upgrade()) {
            core.set_language(language);
            copy_core_settings_to_dialog(&core, &dialog);
            if let Err(error) = save_storylock_ui_settings(&settings_from_storylock_core(&core)) {
                core.set_config_status(SharedString::from(format!(
                    "Settings save failed: {error}"
                )));
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_browse = core_weak.clone();
    let browse_fallback_dir = package_dir.clone();
    dialog.on_browse_core_data_dir(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_browse.upgrade()) {
            copy_dialog_settings_to_core(&dialog, &core);
            let current_dir = storylock_core_package_dir_from_window(&core, &browse_fallback_dir);
            if let Some(selected_dir) = pick_storylock_folder_once(&current_dir, |dialog| dialog) {
                let package_dir = resolve_storylock_core_package_path(&selected_dir);
                match ensure_storylock_core_package(&package_dir) {
                    Ok(()) => {
                        initialize_storylock_core_empty_window(&core, &package_dir);
                        if let Err(error) =
                            save_storylock_ui_settings(&settings_from_storylock_core(&core))
                        {
                            core.set_config_status(SharedString::from(format!(
                                "Settings save failed: {error}"
                            )));
                        }
                        core.set_config_status(SharedString::from(
                            "StoryLock Core target package selected. Unlock the current package to load package content.",
                        ));
                        copy_core_settings_to_dialog(&core, &dialog);
                    }
                    Err(error) => {
                        core.set_config_status(SharedString::from(format!(
                            "Workspace load failed: {error}"
                        )));
                    }
                }
            }
        }
    });
}

pub(crate) fn wire_answer_editor_callbacks(
    dialog: &AnswerEditorDialog,
    core_weak: slint::Weak<StoryLockCoreApp>,
    package_dir: std::path::PathBuf,
) {
    let weak = dialog.as_weak();
    let core_for_window_close = core_weak.clone();
    let close_dir = package_dir.clone();
    dialog.window().on_close_requested(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_window_close.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            match save_current_node_from_window(&core, &close_dir) {
                Ok(()) => {
                    let _ = clear_learning_completed_state(&close_dir);
                    core.set_export_ready(false);
                    core.set_config_status(SharedString::from(
                        "Answer editor saved current question. Learning must run again before export.",
                    ));
                }
                Err(error) => core.set_config_status(SharedString::from(format!(
                    "Answer editor save failed: {error}"
                ))),
            }
            let _ = dialog.hide();
        }
        slint::CloseRequestResponse::HideWindow
    });

    let weak = dialog.as_weak();
    let core_for_previous = core_weak.clone();
    let previous_dir = package_dir.clone();
    dialog.on_previous_node(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_previous.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &previous_dir).is_ok() {
                let _ = clear_learning_completed_state(&previous_dir);
                core.set_export_ready(false);
                let next_index = core.get_node_index().saturating_sub(1);
                load_node_into_window(&core, &previous_dir, next_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_next = core_weak.clone();
    let next_dir = package_dir.clone();
    dialog.on_next_node(move || {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_next.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &next_dir).is_ok() {
                let _ = clear_learning_completed_state(&next_dir);
                core.set_export_ready(false);
                let next_index = (core.get_node_index() + 1).min(23);
                load_node_into_window(&core, &next_dir, next_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });

    let weak = dialog.as_weak();
    let core_for_select = core_weak;
    let select_dir = package_dir;
    dialog.on_select_node(move |value| {
        if let (Some(dialog), Some(core)) = (weak.upgrade(), core_for_select.upgrade()) {
            copy_answer_editor_to_core(&dialog, &core);
            if save_current_node_from_window(&core, &select_dir).is_ok() {
                let _ = clear_learning_completed_state(&select_dir);
                core.set_export_ready(false);
                let selected_index = value
                    .parse::<i32>()
                    .ok()
                    .map(|number| number - 1)
                    .unwrap_or_else(|| core.get_node_index());
                load_node_into_window(&core, &select_dir, selected_index);
                copy_core_question_to_answer_editor(&core, &dialog);
            }
        }
    });
}

pub(crate) fn copy_core_settings_to_dialog(
    core: &StoryLockCoreApp,
    dialog: &StoryLockCoreSettingsDialog,
) {
    dialog.set_language(core.get_language());
    dialog.set_core_data_dir(core.get_core_data_dir());
}

pub(crate) fn copy_dialog_settings_to_core(
    dialog: &StoryLockCoreSettingsDialog,
    core: &StoryLockCoreApp,
) {
    core.set_language(dialog.get_language());
    core.set_core_data_dir(dialog.get_core_data_dir());
}

pub(crate) fn copy_core_question_to_answer_editor(
    core: &StoryLockCoreApp,
    dialog: &AnswerEditorDialog,
) {
    dialog.set_language(core.get_language());
    dialog.set_selected_question(core.get_selected_question());
    dialog.set_question_text(core.get_question_text());
    dialog.set_answer_1(core.get_answer_1());
    dialog.set_answer_1_state(core.get_answer_1_state());
    dialog.set_answer_2(core.get_answer_2());
    dialog.set_answer_2_state(core.get_answer_2_state());
    dialog.set_answer_3(core.get_answer_3());
    dialog.set_answer_3_state(core.get_answer_3_state());
    dialog.set_answer_4(core.get_answer_4());
    dialog.set_answer_4_state(core.get_answer_4_state());
    dialog.set_answer_5(core.get_answer_5());
    dialog.set_answer_5_state(core.get_answer_5_state());
    dialog.set_answer_6(core.get_answer_6());
    dialog.set_answer_6_state(core.get_answer_6_state());
    dialog.set_answer_7(core.get_answer_7());
    dialog.set_answer_7_state(core.get_answer_7_state());
    dialog.set_answer_8(core.get_answer_8());
    dialog.set_answer_8_state(core.get_answer_8_state());
    dialog.set_answer_9(core.get_answer_9());
    dialog.set_answer_9_state(core.get_answer_9_state());
}

pub(crate) fn copy_answer_editor_to_core(dialog: &AnswerEditorDialog, core: &StoryLockCoreApp) {
    core.set_selected_question(dialog.get_selected_question());
    core.set_question_text(dialog.get_question_text());
    core.set_answer_1(dialog.get_answer_1());
    core.set_answer_1_state(dialog.get_answer_1_state());
    core.set_answer_2(dialog.get_answer_2());
    core.set_answer_2_state(dialog.get_answer_2_state());
    core.set_answer_3(dialog.get_answer_3());
    core.set_answer_3_state(dialog.get_answer_3_state());
    core.set_answer_4(dialog.get_answer_4());
    core.set_answer_4_state(dialog.get_answer_4_state());
    core.set_answer_5(dialog.get_answer_5());
    core.set_answer_5_state(dialog.get_answer_5_state());
    core.set_answer_6(dialog.get_answer_6());
    core.set_answer_6_state(dialog.get_answer_6_state());
    core.set_answer_7(dialog.get_answer_7());
    core.set_answer_7_state(dialog.get_answer_7_state());
    core.set_answer_8(dialog.get_answer_8());
    core.set_answer_8_state(dialog.get_answer_8_state());
    core.set_answer_9(dialog.get_answer_9());
    core.set_answer_9_state(dialog.get_answer_9_state());
}

pub(crate) fn copy_core_learning_to_dialog(core: &StoryLockCoreApp, dialog: &LearningTestDialog) {
    dialog.set_language(core.get_language());
    dialog.set_learning_plan_summary(core.get_learning_plan_summary());
    dialog.set_learning_status(core.get_learning_status());
    dialog.set_learning_progress_summary(core.get_learning_progress_summary());
    dialog.set_learning_total_questions(core.get_learning_total_questions());
    dialog.set_learning_current_question(core.get_learning_current_question());
    dialog.set_learning_checked_prompts(core.get_learning_checked_prompts());
    dialog.set_learning_total_prompts(core.get_learning_total_prompts());
    dialog.set_learning_error_count(core.get_learning_error_count());
    dialog.set_learning_progress_percent(core.get_learning_progress_percent());
    dialog.set_learning_progress_headline(core.get_learning_progress_headline());
    dialog.set_learning_action_hint(core.get_learning_action_hint());
    dialog.set_learning_position(core.get_learning_position());
    dialog.set_learning_question(core.get_learning_question());
    dialog.set_learning_answer_1(core.get_learning_answer_1());
    dialog.set_learning_answer_1_state(core.get_learning_answer_1_state());
    dialog.set_learning_answer_2(core.get_learning_answer_2());
    dialog.set_learning_answer_2_state(core.get_learning_answer_2_state());
    dialog.set_learning_answer_3(core.get_learning_answer_3());
    dialog.set_learning_answer_3_state(core.get_learning_answer_3_state());
    dialog.set_learning_answer_4(core.get_learning_answer_4());
    dialog.set_learning_answer_4_state(core.get_learning_answer_4_state());
    dialog.set_learning_answer_5(core.get_learning_answer_5());
    dialog.set_learning_answer_5_state(core.get_learning_answer_5_state());
    dialog.set_learning_answer_6(core.get_learning_answer_6());
    dialog.set_learning_answer_6_state(core.get_learning_answer_6_state());
    dialog.set_learning_answer_7(core.get_learning_answer_7());
    dialog.set_learning_answer_7_state(core.get_learning_answer_7_state());
    dialog.set_learning_answer_8(core.get_learning_answer_8());
    dialog.set_learning_answer_8_state(core.get_learning_answer_8_state());
    dialog.set_learning_answer_9(core.get_learning_answer_9());
    dialog.set_learning_answer_9_state(core.get_learning_answer_9_state());
    dialog.set_learning_result(core.get_learning_result());
}

pub(crate) fn copy_dialog_learning_to_core(dialog: &LearningTestDialog, core: &StoryLockCoreApp) {
    core.set_learning_answer_1(dialog.get_learning_answer_1());
    core.set_learning_answer_1_state(dialog.get_learning_answer_1_state());
    core.set_learning_answer_2(dialog.get_learning_answer_2());
    core.set_learning_answer_2_state(dialog.get_learning_answer_2_state());
    core.set_learning_answer_3(dialog.get_learning_answer_3());
    core.set_learning_answer_3_state(dialog.get_learning_answer_3_state());
    core.set_learning_answer_4(dialog.get_learning_answer_4());
    core.set_learning_answer_4_state(dialog.get_learning_answer_4_state());
    core.set_learning_answer_5(dialog.get_learning_answer_5());
    core.set_learning_answer_5_state(dialog.get_learning_answer_5_state());
    core.set_learning_answer_6(dialog.get_learning_answer_6());
    core.set_learning_answer_6_state(dialog.get_learning_answer_6_state());
    core.set_learning_answer_7(dialog.get_learning_answer_7());
    core.set_learning_answer_7_state(dialog.get_learning_answer_7_state());
    core.set_learning_answer_8(dialog.get_learning_answer_8());
    core.set_learning_answer_8_state(dialog.get_learning_answer_8_state());
    core.set_learning_answer_9(dialog.get_learning_answer_9());
    core.set_learning_answer_9_state(dialog.get_learning_answer_9_state());
}
