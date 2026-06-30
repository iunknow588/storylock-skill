# StoryLock 从下载到应用操作手册

本文用于说明 StoryLock 加密包从“下载到本地”到“在 Host 中应用”的完整操作链路。阅读时建议配合流程图、真实界面截图和补充说明图一起使用。

## 0. 先看哪些图

基础流程图：

1. 总览图：[01_总览图.png](./01_总览图.png)
2. 下载阶段：[02_下载流程图.png](./02_下载流程图.png)
3. 设置阶段：[03_设置流程图.png](./03_设置流程图.png)
4. 使用与授权阶段：[04_使用与授权流程图.png](./04_使用与授权流程图.png)

真实界面截图：

1. Host 远程 Web 页面：[01_host_remote_web.png](./ui-screenshots/01_host_remote_web.png)
2. Host StoryLock 页面：[02_host_storylock_page.png](./ui-screenshots/02_host_storylock_page.png)
3. Host 设置窗口：[03_host_settings.png](./ui-screenshots/03_host_settings.png)
4. StoryLock Core 空模式：[04_storylock_core_empty_mode.png](./ui-screenshots/04_storylock_core_empty_mode.png)
5. StoryLock 授权挑战：[05_storylock_authorization_challenge.png](./ui-screenshots/05_storylock_authorization_challenge.png)
6. 解锁后的问题总览：[06_storylock_core_unlocked_questions.png](./ui-screenshots/06_storylock_core_unlocked_questions.png)
7. 保存当前包页面：[07_storylock_core_save_current_package.png](./ui-screenshots/07_storylock_core_save_current_package.png)

补充说明图与视频分镜图：

1. 包目录结构检查：[08_storylock_package_structure.png](./ui-screenshots/08_storylock_package_structure.png)
2. 同一包路径贯穿全流程：[09_storylock_same_package_path.png](./ui-screenshots/09_storylock_same_package_path.png)
3. 保存后的应用边界：[10_storylock_save_apply_boundary.png](./ui-screenshots/10_storylock_save_apply_boundary.png)
4. 下载完成落盘：[11_download_package_landing.png](./ui-screenshots/11_download_package_landing.png)
5. 包文件校验清单：[12_package_file_checklist.png](./ui-screenshots/12_package_file_checklist.png)
6. 选择包目录或 vault.stlk：[13_choose_package_or_vault.png](./ui-screenshots/13_choose_package_or_vault.png)
7. Host 确认当前包路径：[14_host_package_path_confirm.png](./ui-screenshots/14_host_package_path_confirm.png)
8. 路径归一结果：[15_path_normalization_result.png](./ui-screenshots/15_path_normalization_result.png)
9. 多个 vault.stlk 冲突：[16_vault_conflict_warning.png](./ui-screenshots/16_vault_conflict_warning.png)
10. 空模式安全边界：[17_empty_mode_security_boundary.png](./ui-screenshots/17_empty_mode_security_boundary.png)
11. 授权前未完成提示：[18_authorization_incomplete_prompt.png](./ui-screenshots/18_authorization_incomplete_prompt.png)
12. 挑战失败但不泄露答案：[19_authorization_failed_detail.png](./ui-screenshots/19_authorization_failed_detail.png)
13. 解锁后编辑 24 个问题：[20_unlocked_editing_detail.png](./ui-screenshots/20_unlocked_editing_detail.png)
14. 学习测试与保存前检查：[21_learning_test_ready.png](./ui-screenshots/21_learning_test_ready.png)
15. 保存当前包成功：[22_save_current_package_success.png](./ui-screenshots/22_save_current_package_success.png)
16. 路径选择成功：[23_path_picker_success.png](./ui-screenshots/23_path_picker_success.png)
17. 解锁成功并加载当前包：[24_unlock_success_loaded.png](./ui-screenshots/24_unlock_success_loaded.png)
18. 保存后 Host 应用状态：[25_host_after_save_apply.png](./ui-screenshots/25_host_after_save_apply.png)
19. 另存为新包的边界：[26_save_as_new_package_boundary.png](./ui-screenshots/26_save_as_new_package_boundary.png)

仍建议补拍的真实截图：

- 系统文件选择器中选中包目录或 `vault.stlk` 的瞬间。当前已有说明图 `13` 和 `23`，但真实文件选择器截图更适合比赛答辩。
- 解锁挑战成功后的确认提示或状态变化。当前已有真实图 `06` 与说明图 `24`，如果产品界面存在“解锁成功”Toast，建议补拍。
- 保存成功后的真实 Toast / 状态栏。当前已有真实图 `07` 与说明图 `22`，如果界面有保存成功反馈，建议补拍。
- 另存为到新目录后的真实路径对比。当前已有说明图 `26`，适合先用于文档，后续可换成真实截图。

## 1. 下载 StoryLock 包

对应流程图：[02_下载流程图.png](./02_下载流程图.png)

![下载 StoryLock 包](./ui-screenshots/11_download_package_landing.png)

![包文件校验清单](./ui-screenshots/12_package_file_checklist.png)

![包目录结构检查](./ui-screenshots/08_storylock_package_structure.png)

操作步骤：

1. 获取 StoryLock 加密包。
2. 将包复制到本地固定目录，建议保持目录名稳定，例如 `identity-package`。
3. 确认包根目录中至少包含：
   - `vault.stlk`
   - `package-manifest.json`
   - `resource-catalog.json`
   - `learning-policy.json`
4. 如果包根目录旁边有 `config` 目录，还应确认其中包含策略模板文件：
   - `login-sites.json`
   - `signing-actions.json`
   - `agent-tasks.json`
5. 后续 Host 设置、StoryLock Core 解锁、保存和应用都应指向同一个包根目录。

检查点：

- 可以选择整个包目录，也可以选择包内的 `vault.stlk`。
- 选择 `vault.stlk` 时，系统会自动归一到它所在的包根目录。
- 不要同时混用下载目录、旧导出目录和当前编辑目录中的不同 `vault.stlk`。

## 2. 设置包目录或 vault.stlk

对应流程图：[03_设置流程图.png](./03_设置流程图.png)

![Host StoryLock 页面](./ui-screenshots/02_host_storylock_page.png)

![Host 设置窗口](./ui-screenshots/03_host_settings.png)

![选择包目录或 vault.stlk](./ui-screenshots/13_choose_package_or_vault.png)

![路径选择成功](./ui-screenshots/23_path_picker_success.png)

![Host 中确认当前包路径](./ui-screenshots/14_host_package_path_confirm.png)

![路径归一结果](./ui-screenshots/15_path_normalization_result.png)

操作步骤：

1. 打开 Host 主界面。
2. 进入“底层 StoryLock”页面。
3. 在“StoryLock 加密包”入口中选择包目录或 `vault.stlk`。
4. 如需从全局设置修改，点击右上角设置按钮，在设置窗口中选择同一个包目录或 `vault.stlk`。
5. 系统会把 `vault.stlk` 解析为所在包目录，并把该路径保存为当前 StoryLock Core 数据目录。
6. 如果检测到多个同名 `vault.stlk` 且内容不同，按提示重新选择正确包。

![多个 vault.stlk 冲突](./ui-screenshots/16_vault_conflict_warning.png)

确认点：

- Host 页面中的“StoryLock 加密包”路径应指向当前包。
- “数据边界”应保持 Host 只触发测试与授权，不直接展示敏感故事内容。
- 旧的 `managed_key_package_dir` 和错误导出目录不应继续作为当前包路径使用。

## 3. 打开 StoryLock Core

![StoryLock Core 空模式](./ui-screenshots/04_storylock_core_empty_mode.png)

![空模式安全边界](./ui-screenshots/17_empty_mode_security_boundary.png)

操作步骤：

1. 点击“打开 StoryLock”。
2. StoryLock Core 先进入空模式。
3. 空模式不加载当前包中的故事详情、故事答案和受保护对象。
4. 空模式下保存、学习策略和导出保持不可用。
5. 确认界面中的包路径仍然是第 2 步选择的同一个包。

空模式的意义：

- Host 和 Core 可以确认“目标包是谁”。
- 敏感内容不会在授权前显示。
- 当前包切换后，需要重新解锁。

## 4. 解锁当前包

对应流程图：[04_使用与授权流程图.png](./04_使用与授权流程图.png)

![StoryLock 授权挑战](./ui-screenshots/05_storylock_authorization_challenge.png)

![授权前未完成提示](./ui-screenshots/18_authorization_incomplete_prompt.png)

![挑战失败但不泄露答案](./ui-screenshots/19_authorization_failed_detail.png)

![解锁成功并加载当前包](./ui-screenshots/24_unlock_success_loaded.png)

操作步骤：

1. 在 StoryLock Core 中点击“解锁当前包”。
2. 系统显示当前包的九宫格挑战。
3. 每题显示“已选 X / 应选 Y”。
4. 授权前会提示未完成题号。
5. 挑战不通过时提示第 N 题不匹配、已选数量和应选数量，不显示正确答案内容。
6. 挑战通过后，StoryLock Core 才加载当前包内容。

异常处理：

- 如果提示未完成题号，回到对应题目补选。
- 如果提示数量不匹配，检查该题已选数量是否等于“应选”数量。
- 如果换了包目录或换了 `vault.stlk`，需要重新进入挑战流程。

## 5. 编辑、学习和保存当前包

![StoryLock Core 解锁后问题总览](./ui-screenshots/06_storylock_core_unlocked_questions.png)

![StoryLock Core 保存当前包](./ui-screenshots/07_storylock_core_save_current_package.png)

![解锁后编辑 24 个问题](./ui-screenshots/20_unlocked_editing_detail.png)

![学习测试与保存前检查](./ui-screenshots/21_learning_test_ready.png)

![保存当前包成功](./ui-screenshots/22_save_current_package_success.png)

操作步骤：

1. 解锁后进入 24 个问题总览。
2. 根据需要编辑故事、问题、答案和受保护对象。
3. 编辑期间的临时草稿只属于当前包，不应当作 Host 可读内容。
4. 完成学习测试。
5. 点击导出时保存当前加密包。
6. 默认导出不复制到独立目录；只有选择“另存为”时才产生独立路径。
7. 保存完成后，当前包内的 `vault.stlk`、`package-manifest.json`、`resource-catalog.json` 和 `learning-policy.json` 保持一致。

保存规则：

- “导出”在默认情况下表示保存当前包。
- 如果存在临时草稿，保存时会将它提升到当前 `vault.stlk` 内。
- 保存成功后会记录当前包保存状态。
- 如果另存为到新目录，应明确后续 Host 是否也切换到新目录。

![另存为新包的边界](./ui-screenshots/26_save_as_new_package_boundary.png)

## 6. 应用结果

![同一个 StoryLock 包贯穿全流程](./ui-screenshots/09_storylock_same_package_path.png)

![保存后的应用边界](./ui-screenshots/10_storylock_save_apply_boundary.png)

![保存后 Host 应用状态](./ui-screenshots/25_host_after_save_apply.png)

应用规则：

1. Host 保持存储盲态，不显示故事答案、原始密码、私钥或原始故事文本。
2. StoryLock Core 负责加载、编辑和保存当前包内容。
3. Host 只负责连接测试、授权触发、脱敏状态和本地 API 诊断。
4. 当前包切换后需要重新解锁。
5. 应用阶段继续使用最新保存的同一个 `vault.stlk`。

最终验收：

- 下载、设置、解锁、保存和应用都指向同一个包根目录。
- 未解锁前不会显示敏感故事内容。
- 解锁失败不会泄露正确答案。
- 保存默认写回当前包，只有另存为才产生新包目录。
- Host 侧只展示脱敏状态，不直接读取或编辑 StoryLock 敏感内容。
