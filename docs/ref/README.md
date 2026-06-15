# StoryLock x Pharos 参赛文档

本目录用于整理 StoryLock 参加 Pharos Skill / Agent 方向活动时需要的中文说明材料。

重点不是重复 `story-lock/doc/design/` 里的正式设计内容，而是把比赛需要的表述、演示、调用与提交材料整理成一套可直接使用的文档。

## 1. 本目录回答的问题

1. StoryLock 作为 Skill 方案的项目定位是什么
2. StoryLock 当前如何映射到 Pharos Skill Engine 风格的结构
3. StoryLock 现有 Demo 与调用入口是什么
4. 提交比赛时建议准备哪些材料
5. 已迁移出的可复用 Skill 版本在哪里

## 2. 目录结构

| 文件 | 内容 |
|------|------|
| `01-参赛概览.md` | 面向比赛视角的项目简介、定位、价值与边界 |
| `02-技术映射说明.md` | 对照 Pharos Skill Engine 思路，说明 StoryLock 当前如何映射为 Skill 结构 |
| `03-演示与调用说明.md` | 说明如何运行 Demo、如何展示、如何引用代码入口 |
| `04-提交材料清单.md` | 整理比赛提交时的材料与检查项 |
| `storylock-skill-guide/` | 技术附录，保留一份更标准的 Skill 说明 |
| `../../src/storylock-skill-engine/` | 已迁移出的可复用 Skill 版本，采用 `SKILL.md + references + assets` 结构 |

## 3. 与仓库其他材料的关系

| 目录 | 职责 |
|------|------|
| `story-lock/doc/design/` | 正式需求、系统边界与安全语义 |
| `story-lock/doc/architecture/` | 宿主接入、云端联调与系统拼装 |
| `story-lock/doc/usecase/` | 故事模板、工作台、样例与使用流程 |
| `skill/docs/ref/` | 比赛提交视角下的 Skill 说明、演示说明与提交清单 |
| `skill/src/storylock-skill-engine/` | 从 `story-lock` 迁出的可复用 Skill 打包版本 |

## 4. 外部参考入口

| 名称 | 链接 |
|------|------|
| Pharos Skill Engine Guide | `https://docs.pharos.xyz/tooling-and-infrastructure/pharos-skill-engine-guide` |
| Pharos Agent Carnival 官网 | `https://www.pharos.xyz/agent-carnival` |
| 比赛提交入口 | `https://bit.ly/4v6QsWm` |

说明：

1. 技术文档链接当前可正常访问。
2. 官网入口当前会跳转到 `https://www.pharos.xyz/403`。
3. 提交短链当前跳转到 DoraHacks 的 `pharos-phase1` 页面，实际访问可能受风控或验证码影响。
4. StoryLock 本地实现仍以本仓库现有代码与设计文档为准。
5. 若比赛规则更新，应以官网与提交页的最新说明为准。
