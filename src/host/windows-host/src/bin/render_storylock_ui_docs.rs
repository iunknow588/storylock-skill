use anyhow::{Context, Result};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, PlatformError, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

thread_local! {
    static NEXT_WINDOW: RefCell<Option<Rc<MinimalSoftwareWindow>>> = const { RefCell::new(None) };
}

struct ScreenshotPlatform;

impl Platform for ScreenshotPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(NEXT_WINDOW.with(|slot| {
            slot.borrow_mut()
                .take()
                .unwrap_or_else(|| MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer))
        }))
    }
}

slint::slint! {
    import { HostDashboard, SettingsDialog, StoryLockAuthorizationDialog } from "../slint_ui/host_dashboard.slint";
    import { StoryLockCoreApp } from "../slint_ui/storylock_core.slint";

    export {
        HostDashboard,
        SettingsDialog,
        StoryLockAuthorizationDialog,
        StoryLockCoreApp
    }
}

fn main() -> Result<()> {
    let output_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_output_dir);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("create output dir {}", output_dir.display()))?;

    let _ = slint::platform::set_platform(Box::new(ScreenshotPlatform));

    render_host_remote_web(&output_dir.join("01_host_remote_web.png"))?;
    render_host_storylock_page(&output_dir.join("02_host_storylock_page.png"))?;
    render_host_settings(&output_dir.join("03_host_settings.png"))?;
    render_storylock_empty(&output_dir.join("04_storylock_core_empty_mode.png"))?;
    render_storylock_challenge(&output_dir.join("05_storylock_authorization_challenge.png"))?;
    render_storylock_unlocked(&output_dir.join("06_storylock_core_unlocked_questions.png"))?;
    render_storylock_export(&output_dir.join("07_storylock_core_save_current_package.png"))?;

    Ok(())
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("management")
        .join("流程图")
        .join("ui-screenshots")
}

fn render_host_remote_web(path: &Path) -> Result<()> {
    let (window, app) = with_window(960, 540, HostDashboard::new)?;
    fill_host_common(&app);
    app.set_active_page(0);
    snapshot_window(&window, path)
}

fn render_host_storylock_page(path: &Path) -> Result<()> {
    let (window, app) = with_window(960, 540, HostDashboard::new)?;
    fill_host_common(&app);
    app.set_active_page(1);
    snapshot_window(&window, path)
}

fn render_host_settings(path: &Path) -> Result<()> {
    let (window, dialog) = with_window(760, 420, SettingsDialog::new)?;
    dialog.set_language("zh".into());
    dialog.set_encrypted_data_dir(package_path().into());
    dialog.set_core_launch_status(
        "StoryLock: empty mode | package selected | unlock to load content".into(),
    );
    snapshot_window(&window, path)
}

fn render_storylock_empty(path: &Path) -> Result<()> {
    let (window, app) = with_window(960, 540, StoryLockCoreApp::new)?;
    fill_storylock_empty(&app);
    app.set_active_page(1);
    snapshot_window(&window, path)
}

fn render_storylock_challenge(path: &Path) -> Result<()> {
    let (window, dialog) = with_window(960, 540, StoryLockAuthorizationDialog::new)?;
    dialog.set_is_zh(true);
    dialog.set_challenge_count(22);
    dialog.set_current_index(0);
    dialog.set_current_position("1/22 - 已选 0/应选 2 | 未完成题号: 1, 2, 3, 4, 5...".into());
    dialog.set_current_prompt("第 1 题：故事发生在什么时间？".into());
    dialog.set_selected_answer("已选 0/应选 2".into());
    dialog.set_option_1("春天".into());
    dialog.set_option_2("清晨".into());
    dialog.set_option_3("树桩旁".into());
    dialog.set_option_4("冬天".into());
    dialog.set_option_5("城门口".into());
    dialog.set_option_6("傍晚".into());
    dialog.set_option_7("田地里".into());
    dialog.set_option_8("集市".into());
    dialog.set_option_9("河边".into());
    snapshot_window(&window, path)
}

fn render_storylock_unlocked(path: &Path) -> Result<()> {
    let (window, app) = with_window(960, 540, StoryLockCoreApp::new)?;
    fill_storylock_unlocked(&app);
    app.set_active_page(1);
    snapshot_window(&window, path)
}

fn render_storylock_export(path: &Path) -> Result<()> {
    let (window, app) = with_window(960, 540, StoryLockCoreApp::new)?;
    fill_storylock_unlocked(&app);
    app.set_active_page(4);
    app.set_export_ready(true);
    app.set_export_preview(
        format!(
            "current package save\n  vault.stlk\n  package-manifest.json\n  resource-catalog.json\n  learning-policy.json\n\nLocal path: {}\ntemporaryDraft=false\nlearningState=passed\npreflight=OK\n\n导出表示保存当前加密包；另存为时才选择独立路径。",
            package_path()
        )
        .into(),
    );
    snapshot_window(&window, path)
}

fn with_window<T, F>(width: u32, height: u32, factory: F) -> Result<(Rc<MinimalSoftwareWindow>, T)>
where
    T: ComponentHandle,
    F: FnOnce() -> core::result::Result<T, slint::PlatformError>,
{
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    window.set_size(PhysicalSize::new(width, height));
    NEXT_WINDOW.with(|slot| {
        *slot.borrow_mut() = Some(window.clone());
    });
    let component = factory().map_err(|error| anyhow::anyhow!("{error}"))?;
    component.window().request_redraw();
    Ok((window, component))
}

fn snapshot_window(window: &Rc<MinimalSoftwareWindow>, path: &Path) -> Result<()> {
    window.window().request_redraw();
    let _ = window.draw_if_needed(|_| {});
    let snapshot = window
        .window()
        .take_snapshot()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    write_png_rgba(path, snapshot.width(), snapshot.height(), pixels_to_rgba_bytes(&snapshot))
}

fn fill_host_common(app: &HostDashboard) {
    app.set_language("zh".into());
    app.set_product("Yian Windows Host".into());
    app.set_version("0.1.0".into());
    app.set_mode("Local only".into());
    app.set_identity_id("windows-demo-001".into());
    app.set_device_id("local-device".into());
    app.set_storylock_data_dir(package_path().into());
    app.set_connection_test_status("No connection test has run yet.".into());
    app.set_status_summary(
        format!("Local only | local API http://127.0.0.1:8787/health | package {}", package_path())
            .into(),
    );
    app.set_package_self_check(
        "package directory: identity-package\nvault.stlk: exists\nlearning state: locked\nexport state: save current package"
            .into(),
    );
    app.set_nine_grid_status("九宫格测试尚未开始".into());
    app.set_nine_grid_summary("对象: 未选择".into());
    app.set_management_stats(
        "只读观察\nlocal API: health/status only\nremote interface: disabled by default\nStoryLock protected data: not displayed"
            .into(),
    );
    app.set_diagnostics(
        "Host 只做连接测试、授权触发和脱敏状态显示。\n受保护对象、故事答案、故事详情在挑战通过前不加载。".into(),
    );
}

fn fill_storylock_empty(app: &StoryLockCoreApp) {
    app.set_language("zh".into());
    app.set_package_unlocked(false);
    app.set_core_data_dir(package_path().into());
    app.set_story_title("空模式".into());
    app.set_story_summary("选择或确认当前包后，通过挑战解锁受保护内容。".into());
    app.set_story_plot("未解锁状态不加载故事详情、故事答案或受保护对象。".into());
    app.set_question_1("待解锁".into());
    app.set_question_2("待解锁".into());
    app.set_question_3("待解锁".into());
    app.set_question_4("待解锁 Q4".into());
    app.set_question_5("待解锁 Q5".into());
    app.set_question_6("待解锁 Q6".into());
    app.set_question_7("待解锁 Q7".into());
    app.set_question_8("待解锁 Q8".into());
    app.set_question_9("待解锁 Q9".into());
    app.set_question_10("待解锁 Q10".into());
    app.set_question_11("待解锁 Q11".into());
    app.set_question_12("待解锁 Q12".into());
    app.set_question_13("待解锁 Q13".into());
    app.set_question_14("待解锁 Q14".into());
    app.set_question_15("待解锁 Q15".into());
    app.set_question_16("待解锁 Q16".into());
    app.set_question_17("待解锁 Q17".into());
    app.set_question_18("待解锁 Q18".into());
    app.set_question_19("待解锁 Q19".into());
    app.set_question_20("待解锁 Q20".into());
    app.set_question_21("待解锁 Q21".into());
    app.set_question_22("待解锁 Q22".into());
    app.set_question_23("待解锁 Q23".into());
    app.set_question_24("待解锁 Q24".into());
    app.set_node_overview("当前包未解锁；24 个问题内容暂不读取。".into());
    app.set_config_status("当前为空模式；解锁当前包后加载包内容并启用保存、学习策略和导出。".into());
    app.set_learning_status("解锁当前包后再开始学习。".into());
    app.set_protected_object_list("未解锁状态不加载受保护对象。".into());
}

fn fill_storylock_unlocked(app: &StoryLockCoreApp) {
    app.set_language("zh".into());
    app.set_package_unlocked(true);
    app.set_core_data_dir(package_path().into());
    app.set_story_title("守株待兔".into());
    app.set_story_summary("农夫看见兔子撞上树桩，于是每天等在树桩旁。".into());
    app.set_story_plot("挑战通过后，StoryLock 从当前 vault.stlk 加载故事详情、故事答案和受保护对象。".into());
    app.set_node_overview("24 个问题已从当前包加载。".into());
    app.set_question_1("故事发生在什么时间？".into());
    app.set_question_2("农夫在哪里等待？".into());
    app.set_question_3("兔子撞到了什么？".into());
    app.set_question_4("农夫之后做了什么？".into());
    app.set_question_5("故事说明了什么？".into());
    app.set_question_text("故事发生在什么时间？".into());
    app.set_answer_1("春天".into());
    app.set_answer_1_state("correct".into());
    app.set_answer_2("清晨".into());
    app.set_answer_2_state("correct".into());
    app.set_answer_3("田地里".into());
    app.set_answer_3_state("wrong".into());
    app.set_correct_count("2".into());
    app.set_config_status("当前包已解锁；编辑和保存只作用于当前包目录。".into());
    app.set_learning_status("学习通过后保存当前加密包。".into());
    app.set_learning_progress_summary("22 / 22 prompts completed, errors recorded: 0".into());
    app.set_export_package_dir(package_path().into());
}

fn package_path() -> String {
    "E:\\2026OPC大赛\\skill\\src\\host\\windows-host\\target\\codex-build\\debug\\identity-package"
        .to_string()
}

fn pixels_to_rgba_bytes(buffer: &slint::SharedPixelBuffer<slint::Rgba8Pixel>) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((buffer.width() * buffer.height() * 4) as usize);
    for pixel in buffer.as_slice() {
        bytes.push(pixel.r);
        bytes.push(pixel.g);
        bytes.push(pixel.b);
        bytes.push(if pixel.a == 0 { 255 } else { pixel.a });
    }
    bytes
}

fn write_png_rgba(path: &Path, width: u32, height: u32, rgba: Vec<u8>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let stride = (width * 4) as usize;
    let mut raw = Vec::with_capacity((stride + 1) * height as usize);
    for row in 0..height as usize {
        raw.push(0);
        raw.extend_from_slice(&rgba[row * stride..(row + 1) * stride]);
    }

    let mut zlib = Vec::new();
    zlib.extend_from_slice(&[0x78, 0x01]);
    let mut remaining = raw.as_slice();
    while !remaining.is_empty() {
        let block_len = remaining.len().min(65_535);
        let is_final = block_len == remaining.len();
        zlib.push(if is_final { 1 } else { 0 });
        zlib.extend_from_slice(&(block_len as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(block_len as u16)).to_le_bytes());
        zlib.extend_from_slice(&remaining[..block_len]);
        remaining = &remaining[block_len..];
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    write_chunk(&mut png, b"IHDR", &ihdr);
    write_chunk(&mut png, b"IDAT", &zlib);
    write_chunk(&mut png, b"IEND", &[]);

    fs::write(path, png)?;
    Ok(())
}

fn write_chunk(output: &mut Vec<u8>, name: &[u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(name);
    output.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(name.len() + data.len());
    crc_data.extend_from_slice(name);
    crc_data.extend_from_slice(data);
    output.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let mut a = 1u32;
    let mut b = 0u32;
    for byte in data {
        a = (a + *byte as u32) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
