# StoryLock 谜题插件文档

`storylock-puzzle-plugin` 是可复用的九宫格展示与输入收集核心。

当前职责边界：

- 从草稿中提取题目文本和选项文本
- 将每题选项整理为固定 9 个格子
- 维护用户点击后的选择状态
- 提供答案文本归一化工具

明确不负责：

- 保存正确答案
- 解析 `isCorrect`
- 判断用户答案是否正确
- 执行授权或解锁

## 文档导航

- [概述](./概述.md)
- [接口说明](./接口说明.md)
- [使用指南](./使用指南.md)
- [测试说明](./测试说明.md)

## 数据流

```text
草稿题目 JSON
  -> create_open_challenge_from_draft
  -> StoryLockChallengeCell[]
  -> UI 渲染九宫格
  -> toggle_selection 收集用户选择
  -> 宿主提交 cell_id + selected_answers 给外部验证服务
```

正确答案只存在于外部验证服务或宿主验证模块中，不进入本插件。
