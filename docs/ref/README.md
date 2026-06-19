# StoryLock 参赛参考文档

本目录用于整理 StoryLock 参加 Skill / Agent 方向活动时需要的中文说明材料。当前口径以 `skill/src` 下三层代码为准，不再以旧 `story-lock` 路径或旧迁移接口作为主线。

## 1. 本目录回答的问题

1. StoryLock 当前作为 Skill 项目的定位是什么。
2. 当前三层代码如何映射为可展示的 Skill 结构。
3. 当前演示与调用命令是什么。
4. 提交比赛时建议准备哪些材料。
5. 哪些历史说明只能作为兼容参考。

## 2. 目录结构

| 文件 | 内容 |
| --- | --- |
| `01-参赛概览.md` | 面向比赛视角的项目简介、定位、价值与边界 |
| `02-技术映射说明.md` | 说明当前三包结构如何映射为 Skill 项目 |
| `03-演示与调用说明.md` | 说明如何运行 selftest、e2e 演示和清理命令 |
| `04-提交材料清单.md` | 整理比赛提交时的材料与检查项 |
| `05-Android宿主实现规范.md` | 说明从 Android 宿主 mock 到真实 Android 宿主的实现边界 |
| `06-评审讲解与演示说明.md` | 说明评审讲解路径、演示命令与常见问题回答 |
| `07-APK分发与安装说明.md` | 说明 APK 产物路径、命名、版本展示、安装与 debug/release 策略 |
| `08-易安部署与域名说明.md` | 说明易安网站、第三层网关、域名、环境变量与同域路由检查 |
| `09-三层术语与PHAROS定位.md` | 统一网站层、网关层、Android 宿主层与 PHAROS 定位 |
| `10-Android真机闭环检查.md` | 说明真实 APK 构建、安装、绑定、注册、relay 回连与本地授权检查 |
| `storylock-skill-guide/` | 历史 Skill guide，当前只作为兼容参考 |

## 3. 当前主线代码

| 路径 | 职责 |
| --- | --- |
| `skill/src/skills/local-story-processing/` | 第一层：故事处理与强度评估 |
| `skill/src/skills/local-story-access/` | 第二层：对象强度、九宫格、本地授权 |
| `skill/src/skills/remote-gateway/` | 第三层：远程签名、密码填充、脱敏 |
| `skill/src/engine/` | 兼容演示包 |

## 4. 推荐阅读顺序

1. `01-参赛概览.md`
2. `02-技术映射说明.md`
3. `03-演示与调用说明.md`
4. `04-提交材料清单.md`
5. `05-Android宿主实现规范.md`
6. `06-评审讲解与演示说明.md`
7. `07-APK分发与安装说明.md`
8. `08-易安部署与域名说明.md`
9. `09-三层术语与PHAROS定位.md`
10. `10-Android真机闭环检查.md`
11. `../design/cn/README.md`
12. `../test/StoryLock测试方案_v1.0.md`

## 5. 外部参考入口

| 名称 | 链接 |
| --- | --- |
| Pharos Skill Engine Guide | `https://docs.pharos.xyz/tooling-and-infrastructure/pharos-skill-engine-guide` |
| Pharos Agent Carnival 官网 | `https://www.pharos.xyz/agent-carnival` |
| 比赛提交入口 | `https://bit.ly/4v6QsWm` |

说明：外部页面可能受访问策略或验证码影响，提交前应以官网和提交页的最新说明为准。
