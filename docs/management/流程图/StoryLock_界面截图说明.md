# StoryLock 界面截图说明

本文说明 `ui-screenshots` 目录中的图片用途。图片分为两类：`01-07` 为当前实现生成的真实 Host / StoryLock Core 界面截图；`08-26` 为包结构、路径连续性、保存应用边界和视频分镜所需的补充说明图。

## 1. 图片清单

1. [01_host_remote_web.png](./ui-screenshots/01_host_remote_web.png)  
   Host 主界面的远程 Web 页面，展示本地运行状态、连接测试入口和当前包自检信息。
2. [02_host_storylock_page.png](./ui-screenshots/02_host_storylock_page.png)  
   Host 主界面的 StoryLock 页面，展示统一包入口、九宫格测试入口和 StoryLock 状态。
3. [03_host_settings.png](./ui-screenshots/03_host_settings.png)  
   Host 设置窗口，展示界面语言和 StoryLock 加密包目录或文件入口。
4. [04_storylock_core_empty_mode.png](./ui-screenshots/04_storylock_core_empty_mode.png)  
   StoryLock Core 空模式界面，展示未解锁状态下的 24 题总览和保存、学习、导出禁用状态。
5. [05_storylock_authorization_challenge.png](./ui-screenshots/05_storylock_authorization_challenge.png)  
   StoryLock 授权挑战窗口，展示题目、已选数量、应选数量和未完成题号提示。
6. [06_storylock_core_unlocked_questions.png](./ui-screenshots/06_storylock_core_unlocked_questions.png)  
   StoryLock Core 解锁后的问题总览界面，展示当前包中的问题内容已经加载。
7. [07_storylock_core_save_current_package.png](./ui-screenshots/07_storylock_core_save_current_package.png)  
   StoryLock Core 导出页面，展示“导出即保存当前包”的界面状态。
8. [08_storylock_package_structure.png](./ui-screenshots/08_storylock_package_structure.png)  
   StoryLock 包目录结构检查图，展示下载后应确认的 `vault.stlk`、`package-manifest.json`、`resource-catalog.json`、`learning-policy.json` 和配置模板文件。
9. [09_storylock_same_package_path.png](./ui-screenshots/09_storylock_same_package_path.png)  
   同一包路径连续性说明图，展示下载、Host 设置、Core 解锁、保存应用都应指向同一个包根目录。
10. [10_storylock_save_apply_boundary.png](./ui-screenshots/10_storylock_save_apply_boundary.png)  
    保存后的应用边界说明图，展示 StoryLock Core、当前包目录和 Host / 应用之间的数据边界。
11. [11_download_package_landing.png](./ui-screenshots/11_download_package_landing.png)  
    下载完成落盘图，适合视频中展示 StoryLock 包复制到本地固定目录的动作。
12. [12_package_file_checklist.png](./ui-screenshots/12_package_file_checklist.png)  
    包文件校验清单图，展示进入 Host 前应确认的必需文件和不通过处理方式。
13. [13_choose_package_or_vault.png](./ui-screenshots/13_choose_package_or_vault.png)  
    包目录或 `vault.stlk` 选择说明图，展示两种入口都会归一到包根目录。
14. [14_host_package_path_confirm.png](./ui-screenshots/14_host_package_path_confirm.png)  
    Host 当前包路径确认图，展示设置完成后应检查的包路径和脱敏状态。
15. [15_path_normalization_result.png](./ui-screenshots/15_path_normalization_result.png)  
    路径归一结果图，展示用户选择 `vault.stlk` 后系统保存包根目录。
16. [16_vault_conflict_warning.png](./ui-screenshots/16_vault_conflict_warning.png)  
    多个 `vault.stlk` 冲突图，展示不同内容的包不能混用，需要重新选择。
17. [17_empty_mode_security_boundary.png](./ui-screenshots/17_empty_mode_security_boundary.png)  
    空模式安全边界图，展示 Core 已确认包但暂不读取敏感内容。
18. [18_authorization_incomplete_prompt.png](./ui-screenshots/18_authorization_incomplete_prompt.png)  
    授权前未完成提示图，展示未完成题号、已选数量和应选数量。
19. [19_authorization_failed_detail.png](./ui-screenshots/19_authorization_failed_detail.png)  
    授权失败细节图，展示失败信息只给出题号和数量，不泄露正确答案。
20. [20_unlocked_editing_detail.png](./ui-screenshots/20_unlocked_editing_detail.png)  
    解锁后编辑细节图，展示 24 个问题和当前包编辑范围。
21. [21_learning_test_ready.png](./ui-screenshots/21_learning_test_ready.png)  
    学习测试与保存前检查图，展示保存前需要通过的学习和包预检条件。
22. [22_save_current_package_success.png](./ui-screenshots/22_save_current_package_success.png)  
    保存当前包成功图，展示默认保存写回当前包以及后续应用边界。
23. [23_path_picker_success.png](./ui-screenshots/23_path_picker_success.png)  
    路径选择成功图，展示选择包目录或 `vault.stlk` 后系统统一记录包根目录。
24. [24_unlock_success_loaded.png](./ui-screenshots/24_unlock_success_loaded.png)  
    解锁成功并加载当前包图，展示挑战通过后 Core 才显示可编辑内容。
25. [25_host_after_save_apply.png](./ui-screenshots/25_host_after_save_apply.png)  
    保存后 Host 应用状态图，展示 Host 继续使用同一个包并保持脱敏展示。
26. [26_save_as_new_package_boundary.png](./ui-screenshots/26_save_as_new_package_boundary.png)  
    另存为新包的边界图，展示默认保存与另存为之间的路径切换责任。

## 2. 对应关系

- 下载阶段：配合 [02_下载流程图.png](./02_下载流程图.png)、[08_storylock_package_structure.png](./ui-screenshots/08_storylock_package_structure.png)、[11_download_package_landing.png](./ui-screenshots/11_download_package_landing.png) 和 [12_package_file_checklist.png](./ui-screenshots/12_package_file_checklist.png) 阅读。
- 设置阶段：配合 [03_设置流程图.png](./03_设置流程图.png)、[02_host_storylock_page.png](./ui-screenshots/02_host_storylock_page.png)、[03_host_settings.png](./ui-screenshots/03_host_settings.png)、[13_choose_package_or_vault.png](./ui-screenshots/13_choose_package_or_vault.png)、[23_path_picker_success.png](./ui-screenshots/23_path_picker_success.png)、[14_host_package_path_confirm.png](./ui-screenshots/14_host_package_path_confirm.png)、[15_path_normalization_result.png](./ui-screenshots/15_path_normalization_result.png) 和 [16_vault_conflict_warning.png](./ui-screenshots/16_vault_conflict_warning.png) 阅读。
- 使用与授权阶段：配合 [04_使用与授权流程图.png](./04_使用与授权流程图.png)、[04_storylock_core_empty_mode.png](./ui-screenshots/04_storylock_core_empty_mode.png)、[05_storylock_authorization_challenge.png](./ui-screenshots/05_storylock_authorization_challenge.png)、[17_empty_mode_security_boundary.png](./ui-screenshots/17_empty_mode_security_boundary.png)、[18_authorization_incomplete_prompt.png](./ui-screenshots/18_authorization_incomplete_prompt.png)、[19_authorization_failed_detail.png](./ui-screenshots/19_authorization_failed_detail.png)、[24_unlock_success_loaded.png](./ui-screenshots/24_unlock_success_loaded.png) 和 [06_storylock_core_unlocked_questions.png](./ui-screenshots/06_storylock_core_unlocked_questions.png) 阅读。
- 保存与应用阶段：配合 [07_storylock_core_save_current_package.png](./ui-screenshots/07_storylock_core_save_current_package.png)、[20_unlocked_editing_detail.png](./ui-screenshots/20_unlocked_editing_detail.png)、[21_learning_test_ready.png](./ui-screenshots/21_learning_test_ready.png)、[22_save_current_package_success.png](./ui-screenshots/22_save_current_package_success.png)、[26_save_as_new_package_boundary.png](./ui-screenshots/26_save_as_new_package_boundary.png)、[09_storylock_same_package_path.png](./ui-screenshots/09_storylock_same_package_path.png)、[10_storylock_save_apply_boundary.png](./ui-screenshots/10_storylock_save_apply_boundary.png) 和 [25_host_after_save_apply.png](./ui-screenshots/25_host_after_save_apply.png) 阅读。

## 3. 还建议补拍的真实界面

当前文档已经可以完整说明流程。若用于比赛答辩或视频成片，建议后续再补拍以下真实界面，并替换或补充对应说明图：

- 文件选择器选中包目录或 `vault.stlk` 的真实截图，对应 `13` 和 `23`。
- 解锁成功 Toast 或状态变化截图，对应 `24`。
- 保存成功 Toast / 状态栏截图，对应 `22`。
- 另存为新目录后的真实路径对比截图，对应 `26`。
