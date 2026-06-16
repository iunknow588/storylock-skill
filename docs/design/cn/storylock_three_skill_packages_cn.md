# StoryLock 三个 Skill 包拆分策略

## 目的

本文档用于回答以下问题：

1. 是否应将 StoryLock 拆成三个 Skill 包
2. 三个 Skill 包分别负责什么
3. Pharos Skill Engine 适合放在哪一层
4. 本地安全访问应由谁负责

## 结论

当前阶段，建议按三个 Skill 包进行设计，这样更有利于后续实现和边界固定。

建议拆分为：

1. **第一包：本地故事处理 Skill 包**
2. **第二包：本地故事访问 Skill 包**
3. **第三包：远程代理与委托 Skill 包**

这三个包不是按“功能多少”拆，而是按“安全边界和执行角色”拆。

## 包一：本地故事处理 Skill 包

建议名称：

1. `storylock-local-story-processing-skill`

负责：

1. 故事编辑
2. 故事修改
3. 故事完善
4. 发布前内容整理
5. 私密故事材料的本地处理

对应能力：

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`
3. 后续可扩展 `StoryPersistSkill`

特点：

1. 纯本地
2. 接触私密故事内容
3. 不应依赖远程秘密托管

## 包二：本地故事访问 Skill 包

建议名称：

1. `storylock-local-story-access-skill`

负责：

1. challenge 验证
2. session 建立
3. 受保护故事对象读取
4. 受保护故事对象写回
5. 本地对象访问策略判断

对应能力：

1. 未来正式定义的 `StoryAccessSkill`
2. 当前可借鉴 `LoginAuthorizationSkill`
3. 当前可借鉴 `SigningAuthorizationSkill`

特点：

1. 纯本地
2. 是内容处理层的前置授权层
3. 是整个系统的本地安全访问边界

## 包三：远程代理与委托 Skill 包

建议名称：

1. `storylock-remote-gateway-skill`

负责：

1. 接收第三方 Skill 请求
2. 请求包装
3. 远程编排
4. 委托调用本地能力
5. 返回结构化结果
6. 记录责任链与审计元信息

对应能力：

1. 本地 Agent 网关远程入口
2. 代理签名委托入口
3. 第三方 Skill 集成入口

特点：

1. 远程部署
2. 不直接持有私钥
3. 不直接读取本地 secret object
4. 通过受控接口调用第二包，再由第二包调用第一包

## 三包调用链

推荐调用关系：

1. 第三包调用第二包
2. 第二包调用第一包
3. 第一包不直接对第三包开放

也就是说：

1. 远程层不能直接访问底层故事处理
2. 必须先经过本地访问层
3. 本地访问层是唯一的本地安全入口

## 为什么这样拆更好

### 1. 边界更清楚

第一包只负责内容处理。  
第二包只负责授权和对象访问。  
第三包只负责远程接入、包装与委托。

### 2. 更适合逐层最小权限

1. 第三包不需要知道原始私密故事
2. 第二包不需要负责文本生成逻辑
3. 第一包不需要直接面向第三方请求

### 3. 更适合后续独立实现

三个包可以分别定义：

1. 本地处理能力接口
2. 本地访问控制接口
3. 远程委托接口

这样实现时不容易把授权、内容处理、远程接入搅在一起。

## Pharos Skill Engine 是否适合提供本地安全访问

根据官方对 Pharos Skill Engine 的公开说明，它更像是：

1. 一个统一的 Skill 包结构
2. 一个 on-chain / agent capability 的集成入口
3. 一个通过 `SKILL.md`、`references/`、`assets/` 暴露能力的技能包

官方 Agent Center 对 `pharos-skill-engine` 的描述是：

1. “A unified Pharos Skill that bundles essential on-chain capabilities”
2. 能力包括 balance queries、contract interaction、token deployment、airdrops、developer scripting

从这个定位看，Pharos Skill Engine 更适合：

1. 第三层远程 Skill 包
2. 第三方 Agent 的能力接入层
3. 链上能力、远程能力、委托能力的包装层

而不适合直接承担：

1. 第二层本地安全访问边界
2. challenge answers 的本地保管
3. secret object 的本地读取控制
4. 私密故事对象的本地读写策略

## 对 Pharos 的判断

建议采用下面这个判断：

### 可以依赖 Pharos Skill Engine 的部分

1. 第三层远程 Skill 包
2. 远程能力发现
3. Skill 自说明结构
4. 第三方 Skill 接入
5. 远程委托入口

### 不应依赖 Pharos Skill Engine 直接提供的部分

1. 本地 secret store 访问
2. challenge answers 安全保管
3. 本地 session 安全控制
4. 本地签名密钥使用边界
5. 本地故事对象的访问策略执行

## 推荐方案

因此，更合理的实现策略是：

### 第一包

完全由自己实现，纯本地。

### 第二包

完全由自己实现，纯本地。

### 第三包

可以基于 Pharos Skill Engine 的组织方式和远程 Skill 模式构建。

也就是说：

1. 本地安全访问不是交给 Pharos Skill Engine
2. 本地安全访问应由 StoryLock 自己实现
3. Pharos Skill Engine 更适合放在第三层，作为远程访问与能力编排的 Skill 载体

## 最终建议

当前阶段建议直接固定为：

1. `storylock-local-story-processing-skill`
2. `storylock-local-story-access-skill`
3. `storylock-remote-gateway-skill`

其中：

1. 前两包由 StoryLock 自己实现
2. 第三包优先参考并适配 Pharos Skill Engine 的 Skill 包结构

## 当前仓库落地情况

当前仓库中，三个包骨架已经对应到：

1. `src/storylock-local-story-processing-skill/`
2. `src/storylock-local-story-access-skill/`
3. `src/storylock-remote-gateway-skill/`

可作为后续代码实现的直接起点。

## 结论

对你现在的系统来说，三个 Skill 包是合理的。

但是否使用 Pharos Skill Engine，应这样划分：

1. **第一包：不用 Pharos 提供本地安全访问**
2. **第二包：不用 Pharos 提供本地安全访问**
3. **第三包：使用 Pharos Skill Engine 作为远程 Skill 组织与接入层**
