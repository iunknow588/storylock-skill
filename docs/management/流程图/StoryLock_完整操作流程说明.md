# StoryLock 完整操作流程说明

## 1. 总体目标

这套流程描述的是 StoryLock 包从获取、进入空模式、选择包、挑战解锁、学习、保存并应用的完整过程。整个过程围绕同一个包根目录组织，核心验收点是：下载、设置、解锁、保存和应用都指向同一个 `vault.stlk` 所在目录。

## 2. 图文对应关系

### 2.1 总览图：[01_总览图.png](./01_总览图.png)

总览图展示获取包、进入空模式、选择包目录或 `vault.stlk`、路径一致性自检、挑战解锁、加载当前包内容、保存当前包并应用的主流程。

### 2.2 下载流程图：[02_下载流程图.png](./02_下载流程图.png)

下载流程图展示包获取、复制到本地固定目录、检查必需文件，并进入后续设置阶段的过程。

对应图片：

- [08_storylock_package_structure.png](./ui-screenshots/08_storylock_package_structure.png)
- [11_download_package_landing.png](./ui-screenshots/11_download_package_landing.png)
- [12_package_file_checklist.png](./ui-screenshots/12_package_file_checklist.png)

### 2.3 设置流程图：[03_设置流程图.png](./03_设置流程图.png)

设置流程图展示统一入口、路径归一、冲突检查、空模式与待解锁状态的过程。

对应界面：

- [02_host_storylock_page.png](./ui-screenshots/02_host_storylock_page.png)
- [03_host_settings.png](./ui-screenshots/03_host_settings.png)
- [13_choose_package_or_vault.png](./ui-screenshots/13_choose_package_or_vault.png)
- [23_path_picker_success.png](./ui-screenshots/23_path_picker_success.png)
- [14_host_package_path_confirm.png](./ui-screenshots/14_host_package_path_confirm.png)
- [15_path_normalization_result.png](./ui-screenshots/15_path_normalization_result.png)
- [16_vault_conflict_warning.png](./ui-screenshots/16_vault_conflict_warning.png)

### 2.4 使用与授权流程图：[04_使用与授权流程图.png](./04_使用与授权流程图.png)

使用与授权流程图展示 Host 启动后进入空模式、发起挑战、解锁并加载当前包内容、学习、保存当前包和最终应用的过程。

对应界面：

- [04_storylock_core_empty_mode.png](./ui-screenshots/04_storylock_core_empty_mode.png)
- [05_storylock_authorization_challenge.png](./ui-screenshots/05_storylock_authorization_challenge.png)
- [17_empty_mode_security_boundary.png](./ui-screenshots/17_empty_mode_security_boundary.png)
- [18_authorization_incomplete_prompt.png](./ui-screenshots/18_authorization_incomplete_prompt.png)
- [19_authorization_failed_detail.png](./ui-screenshots/19_authorization_failed_detail.png)
- [24_unlock_success_loaded.png](./ui-screenshots/24_unlock_success_loaded.png)
- [06_storylock_core_unlocked_questions.png](./ui-screenshots/06_storylock_core_unlocked_questions.png)
- [07_storylock_core_save_current_package.png](./ui-screenshots/07_storylock_core_save_current_package.png)

### 2.5 保存与应用补充图

- [20_unlocked_editing_detail.png](./ui-screenshots/20_unlocked_editing_detail.png)
- [21_learning_test_ready.png](./ui-screenshots/21_learning_test_ready.png)
- [22_save_current_package_success.png](./ui-screenshots/22_save_current_package_success.png)
- [26_save_as_new_package_boundary.png](./ui-screenshots/26_save_as_new_package_boundary.png)
- [09_storylock_same_package_path.png](./ui-screenshots/09_storylock_same_package_path.png)
- [10_storylock_save_apply_boundary.png](./ui-screenshots/10_storylock_save_apply_boundary.png)
- [25_host_after_save_apply.png](./ui-screenshots/25_host_after_save_apply.png)

这些图用于说明保存当前包、另存为新包、Host 继续应用当前包之间的边界。

## 3. 对现有实现的对应

- `core_data/settings.rs` 负责路径归一、冲突检测和旧配置清理。
- `core_data/window_init.rs` 负责把当前包载入窗口，并设置初始状态。
- `editor_flow/draft_nodes.rs` 负责保存时自动修正 `recommendedCorrectCount`。
- `callbacks/learning_export.rs` 负责学习测试、授权前检查和导出即保存当前包。
- `callbacks/lifecycle.rs` 负责设置页入口和包目录选择。

## 4. 新流程定义

1. 启动 Host 后先进入空模式。
2. 空模式下不加载当前包中的故事详情、故事答案和受保护对象，只显示空数据或基础状态。
3. 用户选择具体包目录或 `vault.stlk` 后，系统只记录目标包并执行路径检查。
4. 用户发起挑战并通过后，系统才读取该包中的故事详情、故事答案和受保护对象。
5. 当前包发生切换时，解锁状态需要重新建立。

## 5. 操作顺序

1. 打开 Host，先确认当前主界面和包目录状态。
2. 在 Host 主界面或设置窗口中确认 StoryLock 加密包目录或 `vault.stlk`。
3. 打开 StoryLock Core，先进入空模式。
4. 在空模式下发起挑战。
5. 挑战通过后加载当前包中的问题、对象和学习状态。
6. 在学习和检查完成后保存当前加密包。
7. 保存后 Host 继续使用同一个包路径进行应用，除非用户明确另存为并手动切换到新包。

## 6. 说明

1. 总览图对应完整主流程。
2. 下载、设置、使用三张分图分别对应三个阶段。
3. 流程中的关键约束包括同一个包根目录、解锁后才加载当前包内容，以及导出即保存当前包。
4. Host 保持脱敏展示，StoryLock Core 才负责敏感内容的加载、编辑和保存。
