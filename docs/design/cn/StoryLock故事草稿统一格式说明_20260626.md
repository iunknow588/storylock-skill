# StoryLock故事草稿统一格式说明

## 目标

将“模板文件”和“草稿文件”统一为同一种 JSON 结构。

统一后：

1. 模板库中的每个故事文件，本质上就是一个完整草稿。
2. 当前编辑中的作者草稿，也使用同一结构。
3. 名字可以不同，但字段格式必须一致。

## 目录结构

当前三端统一采用如下结构：

```text
story-drafts/
  manifest.json
  shouzhudaitu-zh.json
  caochong-weighs-elephant-zh.json
  kongrong-shares-pears-en.json
```

其中：

1. `manifest.json` 负责列出有哪些可用故事草稿。
2. 每个 `*.json` 文件都是一个可直接加载的故事草稿。

## manifest 结构

```json
{
  "schemaVersion": "storylock-story-draft-manifest-v1",
  "defaultTemplateId": "shouzhudaitu-zh",
  "items": [
    {
      "templateId": "shouzhudaitu-zh",
      "language": "zh-CN",
      "storyTitle": "守株待兔",
      "fileName": "shouzhudaitu-zh.json"
    }
  ]
}
```

约束：

1. `defaultTemplateId` 必须非空。
2. `items` 当前要求正好 3 个。
3. `fileName` 指向同目录中的草稿文件。

## 故事草稿结构

每个故事文件统一为：

```json
{
  "version": "1",
  "templateId": "shouzhudaitu-zh",
  "language": "zh-CN",
  "storyTitle": "守株待兔",
  "summary": "简述",
  "storyPlot": "完整故事",
  "memoryAnchors": ["锚点1", "锚点2"],
  "elementGroups": ["time", "place", "person", "object", "event", "reaction", "choice", "result"],
  "nodes": []
}
```

## nodes 结构

每个草稿必须包含 24 个节点。

单个节点结构：

```json
{
  "nodeId": "question-01",
  "title": "问题 01",
  "elementId": "time",
  "question": "题目内容",
  "recommendedSelectionMode": "single_select",
  "recommendedCorrectCount": 1,
  "candidatePoolSize": 9,
  "recallPriority": "normal",
  "verifyPolicy": "caseInsensitive + trim",
  "editorNotes": "说明",
  "canonicalAnswerLocalOnly": "正确答案",
  "acceptedAnswersLocalOnly": ["正确答案"],
  "answerOptionsLocalOnly": [
    { "text": "正确答案", "isCorrect": true },
    { "text": "错误答案", "isCorrect": false }
  ]
}
```

约束：

1. `nodes.length` 必须等于 24。
2. 每个节点必须有非空的 `nodeId` 和 `question`。
3. `answerOptionsLocalOnly` 数量必须在 2 到 9 之间。
4. 每个节点必须至少有 1 个 `isCorrect=true` 的选项。
5. 每个节点必须有非空的 `canonicalAnswerLocalOnly`。
6. 每个节点必须至少有 1 个 `acceptedAnswersLocalOnly`。
7. 同一故事内 24 个 `question` 应保持唯一，避免重复题干。

## 当前三端实现

### Windows Host

1. 通过 `assets/story-drafts/manifest.json` 加载模板索引。
2. 逐个读取故事草稿文件。
3. 默认作者草稿直接使用默认模板文件。

### Android Host

1. `AndroidStoryTemplateRepository` 先读取 manifest。
2. 再按 `fileName` 加载 3 个故事草稿文件。
3. 已增加基本字段校验。
4. 当前工作区已经验证可通过 Android debug APK 构建流程读取该目录结构。

### Linux Host

1. `server.mjs` 先读取 manifest。
2. 再按 `fileName` 加载故事草稿文件。
3. 保持 `/story-templates` 接口不变，但底层数据源已改为统一草稿体系。

## 结论

模板和草稿现在可以视为同一个体系：

1. 模板 = 可供选择的故事草稿文件。
2. 草稿 = 当前正在编辑或导入的同结构文件。
3. 区别只在用途，不在格式。

## 推荐校验

可直接运行以下命令检查三端草稿资产是否一致：

```powershell
npm run test:story-drafts
```

该检查会同时验证：

1. 三端 `manifest.json` 是否一致。
2. 三端 3 个草稿文件是否一致。
3. 每个草稿是否满足 24 题、非空字段、答案选项数量范围等约束。
