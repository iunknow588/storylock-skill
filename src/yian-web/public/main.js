const I18N = {
  zh: {
    pageTitle: "易安 | Yian",
    pageDescription: "易安帮助普通用户下载易安 App、完成本地设备绑定、查看请求状态，并在本地设备上确认需要授权的请求。",
    "header.kicker": "StoryLock 本地确认",
    "header.brand": "易安",
    "nav.menu": "菜单",
    "nav.home": "首页",
    "nav.overview": "产品说明",
    "nav.demo": "视频说明",
    "nav.architecture": "安全方式",
    "nav.binding": "下载绑定",
    "nav.apk": "安装版本",
    "nav.userGuide": "使用说明",
    "nav.flow": "使用流程",
    "nav.flowGallery": "图解流程",
    "nav.faq": "常见问题",
    "nav.observeDebug": "观察调试",
    "nav.help": "帮助说明",
    "hero.kicker": "StoryLock 本地安全确认",
    "hero.imageAlt": "第三方 Agent 通过云平台上的易安远程入口与私人智能助理双向通信，StoryLock 本地核心只与私人智能助理本地通信的示意图",
    "hero.title": "用易安把 StoryLock 绑定到你的本地设备",
    "hero.copy": "下载易安 App，完成一次绑定。之后第三方 Agent 或云服务发起的敏感请求，会先到你的私人智能助理，再由 StoryLock 本地核心确认后继续。",
    "hero.download": "下载易安 App",
    "hero.downloadWindows": "下载 Windows 本地宿主原型",
    "hero.downloadAndroid": "下载 Android 版本",
    "hero.downloadLinux": "下载 Linux 原型包",
    "hero.bind": "打开绑定入口",
    "hero.status": "查看请求状态",
    "hero.stats.roleLabel": "第一步",
    "hero.stats.roleValue": "下载并安装 App",
    "hero.stats.capabilityLabel": "第二步",
    "hero.stats.capabilityValue": "打开绑定入口",
    "hero.stats.localLabel": "使用时",
    "hero.stats.localValue": "在本地设备上确认请求",
    "overview.kicker": "产品说明",
    "overview.title": "易安把云端请求接到你的本地设备确认",
    "overview.copy1": "易安远程入口部署在三方云服务平台上，供第三方 Agent、pharos Agent 或 OpenClaw 通过 Skill 发起受控请求。",
    "overview.copy2": "请求到达你的本地设备后，先由私人智能助理解释来源、内容和风险，再通过受控本地接口交给 StoryLock 本地核心确认；最小结果只回到私人智能助理。",
    "overview.copy3": "私人智能助理可以用 AI 协助生成故事模板和交互提示，但不直接接触故事存储服务；StoryLock 本地核心保持无网络、负责敏感存储和授权确认。",
    "video.kicker": "视频说明",
    "video.title": "通过演示视频了解易安如何完成本地授权确认",
    "video.copy1": "这个视频文件使用固定公开地址 /assets/videos/yian-demo.mp4。后续只要替换服务器上的同名文件，对外 URI 就不会变化。",
    "video.copy2": "视频用于快速展示下载、绑定、本地确认和请求状态查看流程，适合在介绍易安时直接播放。",
    "video.unsupported": "当前浏览器不支持直接播放视频。",
    "architecture.kicker": "安全方式",
    "architecture.title": "云端负责发起，本地负责确认和保护",
    "architecture.layer1.title": "云端 Agent 发起",
    "architecture.layer1.copy": "第三方 Agent、pharos Agent 或 OpenClaw 通过 Skill 调用部署在云平台上的易安远程入口。",
    "architecture.layer1.item1": "通过 Skill 访问",
    "architecture.layer1.item2": "由云平台中转",
    "architecture.layer1.item3": "不接触本地秘密",
    "architecture.layer2.title": "私人智能助理解释",
    "architecture.layer2.copy": "私人智能助理与易安远程入口双向通信，接收请求、返回确认状态，并向你解释来源、内容和风险。",
    "architecture.layer2.item1": "本地设备保持联网",
    "architecture.layer2.item2": "解释请求风险",
    "architecture.layer2.item3": "不读故事存储",
    "architecture.layer3.title": "本地核心确认",
    "architecture.layer3.copy": "StoryLock 本地核心保持无网络，只接受私人智能助理的本地受控调用，最小结果也只返回给私人智能助理。",
    "architecture.layer3.item1": "本地受控调用",
    "architecture.layer3.item2": "系统确认保护",
    "architecture.layer3.item3": "只回私人智能助理",
    "binding.kicker": "下载与绑定",
    "binding.title": "下载后如何绑定",
    "binding.step1.title": "1. 下载 App",
    "binding.step1.copy": "从易安下载易安 App 安装包。下载前请确认页面来源可信，安装后保留 App 并允许它联网。",
    "binding.step2.title": "2. 打开绑定链接",
    "binding.step2.copy": "点击“打开绑定入口”，系统会唤起可信本地设备上的私人智能助理并写入本次绑定信息。请确认连接的是你信任并准备使用的设备。",
    "binding.step3.title": "3. 完成注册",
    "binding.step3.copy": "私人智能助理完成首次注册后，回到易安查看请求状态。若显示离线，请打开 App、检查网络，必要时重新绑定。",
    "apk.kicker": "安装版本",
    "apk.title": "安装说明与当前版本",
    "apk.platforms.title": "选择本地版本",
    "apk.platforms.windows": "Windows 本地宿主原型",
    "apk.platforms.android": "Android 手机版本",
    "apk.platforms.linux": "Linux 原型版本",
    "apk.platforms.all": "查看全部版本",
    "apk.install.title": "安装流程",
    "apk.install.step1": "根据当前设备选择 Windows 本地宿主原型、Android 或 Linux 原型包。",
    "apk.install.step2": "在本地电脑或 Android 设备上安装，并只允许来自可信来源的安装。",
    "apk.install.step3": "从易安打开绑定链接，按本地设备提示完成绑定。",
    "apk.install.step4": "完成首次注册后，回到站点查看 App 在线状态。",
    "apk.rules.title": "版本信息",
    "apk.rules.copy": "下载前请核对页面展示的版本号、文件大小和校验值。当前如果还没有正式安装包，下载入口会提示未配置或跳转到已配置的下载地址。",
    "apk.rules.path": "文件来源：以页面下载入口实际提供的安装包为准。",
    "apk.rules.name": "文件名称：优先使用带版本号的易安 App 安装包。",
    "apk.rules.version": "版本号：以安装版本页和下载入口展示的信息为准。",
    "apk.rules.policy": "测试版只用于安装验证；正式候选版用于对外试用。安装前请核对版本、来源和校验值。",
    "apk.metadata.loading": "正在读取版本信息。",
    "apk.metadata.unavailable": "暂未读取到版本信息。",
    "apk.metadata.copy": "复制校验值",
    "apk.metadata.copied": "已复制",
    "apk.metadata.prototype": "原型限制",
    "apk.metadata.versionName": "版本号",
    "apk.metadata.versionCode": "版本代码",
    "apk.metadata.packageKind": "包类型",
    "apk.metadata.releaseChannel": "发布渠道",
    "apk.metadata.fileSizeBytes": "文件大小",
    "apk.metadata.checksum": "SHA-256",
    "userGuide.kicker": "使用说明",
    "userGuide.title": "普通用户如何使用易安",
    "userGuide.use.title": "开始使用",
    "userGuide.use.copy": "先下载并安装易安 App，再回到易安打开绑定入口。绑定完成后，私人智能助理会在你的本地设备上接收需要本地确认的请求。",
    "userGuide.approve.title": "本地确认",
    "userGuide.approve.copy": "当需要签名或填充密码时，请在本地设备上查看请求内容，完成问题确认与系统生物识别或设备凭据确认。没有你的本地确认，请求不会完成。",
    "userGuide.notice.title": "注意事项",
    "userGuide.notice.copy": "只从可信页面下载 App；不要把绑定链接转发给他人；安装后保持本地设备网络可用；如果状态显示离线，请重新打开 App 或重新绑定。",
    "flow.kicker": "使用流程",
    "flow.title": "从下载到完成一次本地确认",
    "flow.item1": "在易安页面点击下载，获取易安 App 安装包。",
    "flow.item2": "在本地设备上安装 App，并允许来自当前可信来源的安装。",
    "flow.item3": "回到易安打开绑定入口，按本地设备提示完成首次绑定。",
    "flow.item4": "绑定完成后，在请求状态页查看本地设备是否在线、是否有待处理请求。",
    "flow.item5": "之后遇到需要授权的操作时，先由私人智能助理解释请求来源、内容和风险。",
    "flow.item6": "确认无误后，再由 StoryLock 本地核心配合本地设备解锁、生物识别或设备凭据完成确认。",
    "flowGallery.kicker": "图解流程",
    "flowGallery.title": "把下载、设置、授权和保存放到同一张路线图里",
    "flowGallery.copy1": "这组流程图用于说明 StoryLock 包从下载到本地、选择当前包、进入空模式、挑战解锁、加载内容、学习检查、保存当前包并继续应用的完整链路。",
    "flowGallery.copy2": "核心原则是始终围绕同一个包根目录：下载、设置、解锁、保存和 Host 应用都应该指向同一个 vault.stlk 所在目录。点击任意图片可以打开大图查看细节。",
    "flowGallery.completeAlt": "StoryLock 完整操作流程图",
    "flowGallery.completeCaption": "完整操作流程图：从获取 StoryLock 包到保存并应用当前包。",
    "flowGallery.overviewAlt": "StoryLock 操作总览图",
    "flowGallery.overviewTitle": "总览图",
    "flowGallery.overviewCopy": "先理解同一个包目录如何贯穿下载、解锁、保存和应用。",
    "flowGallery.downloadAlt": "StoryLock 下载流程图",
    "flowGallery.downloadTitle": "下载阶段",
    "flowGallery.downloadCopy": "下载后检查包结构、固定本地目录和关键文件。",
    "flowGallery.setupAlt": "StoryLock 设置流程图",
    "flowGallery.setupTitle": "设置阶段",
    "flowGallery.setupCopy": "选择包目录或 vault.stlk，完成路径归一和冲突检查。",
    "flowGallery.approvalAlt": "StoryLock 使用与授权流程图",
    "flowGallery.approvalTitle": "授权与保存",
    "flowGallery.approvalCopy": "空模式下先挑战解锁，再加载内容、学习、保存当前包。",
    "flowGallery.boundaryTitle": "安全边界",
    "flowGallery.boundaryCloudTitle": "云端入口",
    "flowGallery.boundaryCloudCopy": "只负责展示、下载、中转请求和返回最小状态。",
    "flowGallery.boundaryAssistantTitle": "私人智能助理",
    "flowGallery.boundaryAssistantCopy": "解释来源、内容和风险，不直接读取故事存储。",
    "flowGallery.boundaryCoreTitle": "StoryLock Core",
    "flowGallery.boundaryCoreCopy": "离线处理敏感内容，授权前不展示答案和受保护对象。",
    "flowGallery.troubleTitle": "遇到问题先判断",
    "flowGallery.troubleItem1": "下载后先核对文件名、大小和校验值。",
    "flowGallery.troubleItem2": "选择包时确认只指向一个包根目录或一个 vault.stlk。",
    "flowGallery.troubleItem3": "空模式看不到内容是正常保护，挑战通过后才加载。",
    "flowGallery.troubleItem4": "保存后 Host 应继续使用同一路径，另存为新包需要手动切换。",
    "faq.kicker": "常见问题",
    "faq.title": "绑定或确认遇到问题时先看这里",
    "faq.offline.title": "状态显示离线怎么办？",
    "faq.offline.copy": "先确认本地设备网络可用，再打开易安 App 等待几秒。如果仍然离线，回到易安重新打开绑定入口完成一次新的绑定。",
    "faq.bind.title": "绑定链接应该连接到什么设备？",
    "faq.bind.copy": "绑定链接用于把当前使用环境连接到可信本地设备。重点是确认设备可信、归属清楚，并且是你后续准备用来处理请求的设备。",
    "faq.install.title": "安装时提示风险怎么办？",
    "faq.install.copy": "只从你确认可信的易安页面下载安装包。安装前核对版本号、文件大小和校验值；如果来源不确定，先不要安装。",
    "faq.changeDevice.title": "更换本地设备后怎么处理？",
    "faq.changeDevice.copy": "在新本地设备上重新下载并安装 App，然后从易安重新绑定。旧本地设备不再使用时，请在可用的管理入口中移除旧连接，或停止使用旧设备。",
    "runtime.kicker": "请求状态",
    "runtime.title": "查看当前请求、来源和本地设备情况",
    "runtime.loadGateway": "加载请求状态",
    "runtime.loadRegistrations": "查看设备连接",
    "runtime.refresh": "刷新当前分区",
    "runtime.autorefreshOn": "自动刷新：开",
    "runtime.autorefreshOff": "自动刷新：关",
    "runtime.requestQueue": "待处理请求",
    "runtime.requestSource": "请求来源",
    "runtime.currentMode": "通信方式",
    "runtime.activeHosts": "在线设备",
    "runtime.pending": "等待加载",
    "runtime.relayTransport": "Relay 传输",
    "runtime.relayWait": "默认等待",
    "runtime.relayCoordination": "协调方式",
    "runtime.relayReadiness": "生产就绪",
    "runtime.waitingPolls": "Waiting Poll",
    "runtime.pendingResponses": "Pending Response",
    "runtime.group.current": "当前策略",
    "runtime.group.recent": "最近事件",
    "runtime.group.total": "累计统计",
    "runtime.summaryWaiting": "等待生成运行摘要。",
    "runtime.summaryUnavailable": "当前还没有足够的运行状态信息。",
    "runtime.refreshedWaiting": "最近刷新：等待加载",
    "runtime.lastResolved": "最近成功返回",
    "runtime.lastTimeout": "最近超时",
    "runtime.lastIdleTimeout": "最近空闲超时",
    "runtime.lastReplacedPoll": "最近替换轮询",
    "runtime.lastClientClosed": "最近客户端断开",
    "runtime.totalRequests": "累计请求",
    "runtime.resolvedResponses": "累计成功返回",
    "runtime.timeoutCount": "累计超时",
    "runtime.idleTimeoutCount": "累计空闲超时",
    "runtime.replacedPollCount": "累计替换轮询",
    "runtime.clientClosedPollCount": "累计客户端断开",
    "runtime.responseTitle": "查看原始 JSON",
    "runtime.responseNote": "这里用于查看当前请求、绑定和在线状态；普通使用时请重点确认是否有待处理请求、来源是否可信、设备是否在线。",
    "help.kicker": "帮助说明",
    "help.title": "在同一套页面里完成查看、下载、绑定和确认",
    "help.copy1": "易安用于把需要确认的重要请求交回到用户自己的本地设备上处理。下载并安装易安 App，完成一次绑定后，就可以在本地设备上查看请求来源、理解请求内容，并确认是否继续。",
    "help.copy2": "帮助说明现在和产品说明、安全方式、下载绑定等页面使用同一套导航、语言切换和上一页/下一页交互，不再跳到独立页面。",
    "help.imageAlt": "易安与 StoryLock 本地确认关系示意图",
    "help.start.title": "快速开始",
    "help.start.step1": "根据当前设备选择 Windows 本地宿主原型、Android 或 Linux 原型版本。",
    "help.start.step2": "安装前核对页面展示的版本号、文件大小和校验值。",
    "help.start.step3": "安装易安 App 后，回到易安页面打开绑定入口。",
    "help.start.step4": "绑定完成后，在请求状态页确认本地设备是否在线。",
    "help.approve.title": "确认请求",
    "help.approve.step1": "先在本地设备上查看请求来源、操作内容和风险提示。",
    "help.approve.step2": "确认请求与你正在进行的操作一致，再继续本地确认。",
    "help.approve.step3": "通过本地设备解锁、生物识别或设备凭据完成确认。",
    "help.notice.title": "注意事项",
    "help.notice.item1": "只从可信的易安页面下载安装包。",
    "help.notice.item2": "不要把绑定链接转发给其他人。",
    "help.notice.item3": "遇到不认识的请求，先不要确认，再检查来源。",
    "pager.prev": "前一页",
    "pager.next": "后一页",
    "footer.brand": "易安 Yian",
    "footer.copy": "用于下载易安 App、完成本地设备绑定，并查看当前请求状态。",
    "footer.androidHost": "易安 App",
    "footer.registrations": "设备连接",
    "footer.help": "帮助说明",
    "footer.observeDebug": "观察调试",
    "observeDebug.kicker": "观察调试",
    "observeDebug.title": "把 host 测试和 web 测试集中放到这里",
    "observeDebug.subnav.runtime": "运行状态",
    "observeDebug.subnav.webTests": "Web 联调",
    "observeDebug.subnav.hostTests": "Host 联调",
    "observeDebug.runtimeTitle": "连接与运行状态",
    "observeDebug.runtimeCopy": "这里集中查看 gateway、relay、设备连接和当前请求状态。",
    "observeDebug.webTestsTitle": "Web 测试入口",
    "observeDebug.webTestsCopy": "保留 StoryLock 工作台和 Agent 控制台，供联调和演示使用。",
    "observeDebug.storylockWorkbench": "StoryLock 工作台",
    "observeDebug.agentConsole": "Agent 控制台",
    "observeDebug.hostTestsTitle": "Host 调试说明",
    "observeDebug.hostTestsCopy": "Windows Host、本地 relay 和设备状态观察统一从这里进入，避免主页面承载联调噪音。",
    "observeDebug.hostActions.health": "读取 Host Health",
    "observeDebug.hostActions.status": "读取 Host 状态",
    "observeDebug.hostActions.diagnostics": "读取 Host 诊断",
    "observeDebug.hostActions.refresh": "刷新当前分区",
    "observeDebug.hostActions.autorefreshOn": "自动刷新：开",
    "observeDebug.hostActions.autorefreshOff": "自动刷新：关",
    "observeDebug.hostSummaryWaiting": "等待生成 Host 摘要。",
    "observeDebug.hostRefreshedWaiting": "最近刷新：等待加载",
    "observeDebug.hostGroup.endpoints": "本地接口",
    "observeDebug.hostGroup.current": "当前状态",
    "observeDebug.hostGroup.notes": "最近错误",
    "observeDebug.hostEndpointActions.openManagement": "打开管理页",
    "observeDebug.hostEndpointActions.copyStatus": "复制状态接口",
    "observeDebug.hostEndpointActions.copyDiagnostics": "复制诊断接口",
    "observeDebug.hostSummary.managementUrl": "管理页",
    "observeDebug.hostSummary.statusUrl": "状态接口",
    "observeDebug.hostSummary.diagnosticsUrl": "诊断接口",
    "observeDebug.hostSummary.gatewayUrl": "Gateway",
    "observeDebug.hostSummary.online": "本地 Host",
    "observeDebug.hostSummary.relay": "Relay 状态",
    "observeDebug.hostSummary.mode": "运行模式",
    "observeDebug.hostSummary.lastError": "最近错误",
    "observeDebug.hostTestsItem1": "先看设备是否在线，再看是否有待处理请求。",
    "observeDebug.hostTestsItem2": "需要联调时再打开工作台或控制台，不放在首页主流程。",
    "observeDebug.hostTestsItem3": "原始 JSON 只用于观察和排障，不作为普通用户阅读入口。",
    unconfigured: "未配置",
    unavailable: "未获取",
    connectionRelay: "云端中转连接",
    connectionDeepLink: "本地唤起绑定",
    connectionLocal: "本地直连",
    siteLoaded: "站点已加载。",
    gatewayMissing: "当前未读取到实时请求状态，可稍后再试或直接使用下载与绑定入口。",
    siteOnly: "当前未连接到状态服务，页面仍可用于查看使用说明、下载和绑定入口。",
    waiting: "等待请求状态。",
  },
  en: {
    pageTitle: "Yian | StoryLock Local Approval",
    pageDescription: "Yian helps regular users download the Yian app, bind their local device, inspect request status, and approve sensitive requests on the local device.",
    "header.kicker": "StoryLock Local Approval",
    "header.brand": "Yian",
    "nav.menu": "Menu",
    "nav.home": "Home",
    "nav.overview": "Product",
    "nav.demo": "Demo",
    "nav.architecture": "Safety",
    "nav.binding": "Download",
    "nav.apk": "Install",
    "nav.userGuide": "Guide",
    "nav.flow": "Flow",
    "nav.flowGallery": "Diagrams",
    "nav.faq": "FAQ",
    "nav.observeDebug": "Observe & Debug",
    "nav.help": "Help",
    "hero.kicker": "StoryLock Local Safety",
    "hero.imageAlt": "Diagram showing third-party agents calling Yian Remote Entry, bidirectional communication with a private assistant, and local-only communication with the offline StoryLock Local Core",
    "hero.title": "Bind StoryLock to your local device with Yian",
    "hero.copy": "Download the Yian app and complete one binding step. After that, sensitive requests from third-party agents or cloud services go to your private assistant first, then continue only after StoryLock Local Core approves them.",
    "hero.download": "Download Yian App",
    "hero.downloadWindows": "Download Windows Host Prototype",
    "hero.downloadAndroid": "Download for Android",
    "hero.downloadLinux": "Download Linux Prototype",
    "hero.bind": "Open Binding Entry",
    "hero.status": "View Request Status",
    "hero.stats.roleLabel": "Step 1",
    "hero.stats.roleValue": "Download and install the app",
    "hero.stats.capabilityLabel": "Step 2",
    "hero.stats.capabilityValue": "Open the binding entry",
    "hero.stats.localLabel": "When using it",
    "hero.stats.localValue": "Approve requests on your local device",
    "overview.kicker": "Product",
    "overview.title": "Yian connects cloud requests to local device approval",
    "overview.copy1": "Yian Remote Entry runs on a third-party cloud platform and lets third-party agents, pharos Agent, or OpenClaw call controlled requests through Skills.",
    "overview.copy2": "After a request reaches your local device, the private assistant explains the source, content, and risk, then calls StoryLock Local Core through a controlled local interface; minimal results return only to the assistant.",
    "overview.copy3": "The private assistant can use AI to help create story templates and interaction hints, but it does not directly access story storage. The offline StoryLock Local Core protects sensitive storage and authorization.",
    "video.kicker": "Video",
    "video.title": "Watch how Yian completes local authorization approval",
    "video.copy1": "This video uses the stable public URL /assets/videos/yian-demo.mp4. Later edits can replace the file at the same path without changing the external URI.",
    "video.copy2": "Use the video to introduce the download, binding, local approval, and request-status flow.",
    "video.unsupported": "This browser cannot play the video directly.",
    "architecture.kicker": "Safety",
    "architecture.title": "Cloud starts the request; local components protect approval",
    "architecture.layer1.title": "Cloud Agent Starts",
    "architecture.layer1.copy": "Third-party agents, pharos Agent, or OpenClaw call Yian Remote Entry on a cloud platform through Skills.",
    "architecture.layer1.item1": "Skill-based access",
    "architecture.layer1.item2": "Cloud relay",
    "architecture.layer1.item3": "No local secrets",
    "architecture.layer2.title": "Private Assistant Explains",
    "architecture.layer2.copy": "The private assistant communicates bidirectionally with Yian Remote Entry, receives requests, returns confirmation status, and explains source, content, and risk.",
    "architecture.layer2.item1": "Keep local device online",
    "architecture.layer2.item2": "Explain request risk",
    "architecture.layer2.item3": "No story storage access",
    "architecture.layer3.title": "Local Core Approves",
    "architecture.layer3.copy": "StoryLock Local Core stays offline, accepts only controlled local calls from the private assistant, and returns minimal results only to that assistant.",
    "architecture.layer3.item1": "Controlled local call",
    "architecture.layer3.item2": "System approval",
    "architecture.layer3.item3": "Return only to assistant",
    "binding.kicker": "Download and Binding",
    "binding.title": "How to bind after download",
    "binding.step1.title": "1. Download the app",
    "binding.step1.copy": "Download the Yian app package from Yian. Before installing, confirm that the page source is trusted, then keep the app installed and online.",
    "binding.step2.title": "2. Open the binding link",
    "binding.step2.copy": "Click the binding entry. It opens the private assistant on a trusted local device and writes this binding information. Confirm that the device is trusted and intended for later requests.",
    "binding.step3.title": "3. Complete registration",
    "binding.step3.copy": "After first registration, return to Yian and check request status. If it shows offline, open the app, check the network, or bind again.",
    "apk.kicker": "Install Version",
    "apk.title": "Install guide and current version",
    "apk.platforms.title": "Choose Local Version",
    "apk.platforms.windows": "Windows Host Prototype",
    "apk.platforms.android": "Android Phone Version",
    "apk.platforms.linux": "Linux Prototype",
    "apk.platforms.all": "View All Versions",
    "apk.install.title": "Install Flow",
    "apk.install.step1": "Choose the Windows local host prototype, Android package, or Linux prototype for this device.",
    "apk.install.step2": "Install it on the local computer or Android device, and only allow trusted sources.",
    "apk.install.step3": "Open the Yian binding link and follow the local device prompts to finish binding.",
    "apk.install.step4": "After first registration, return to the site and inspect app online status.",
    "apk.rules.title": "Version Information",
    "apk.rules.copy": "Before downloading, check the displayed version, file size, and checksum. If no public package is configured yet, the download entry will show that state or redirect to the configured download address.",
    "apk.rules.path": "File source: use the package actually provided by the download entry.",
    "apk.rules.name": "File name: prefer a Yian app package with a visible version number.",
    "apk.rules.version": "Version: use the information shown on the install page and download entry.",
    "apk.rules.policy": "Test builds are for installation checks. Release candidates are for public trial use. Verify version, source, and checksum before installing.",
    "apk.metadata.loading": "Loading version information.",
    "apk.metadata.unavailable": "Version information is not available yet.",
    "apk.metadata.copy": "Copy checksum",
    "apk.metadata.copied": "Copied",
    "apk.metadata.prototype": "Prototype limitation",
    "apk.metadata.versionName": "Version",
    "apk.metadata.versionCode": "Version code",
    "apk.metadata.packageKind": "Package kind",
    "apk.metadata.releaseChannel": "Release channel",
    "apk.metadata.fileSizeBytes": "File size",
    "apk.metadata.checksum": "SHA-256",
    "userGuide.kicker": "User Guide",
    "userGuide.title": "How regular users use Yian",
    "userGuide.use.title": "Get Started",
    "userGuide.use.copy": "Download and install the Yian app first, then return to Yian and open the binding entry. After binding, the private assistant receives requests that need local confirmation on your local device.",
    "userGuide.approve.title": "Local Approval",
    "userGuide.approve.copy": "When a signature or password fill is requested, review it on your local device and complete the question confirmation plus biometric or device-credential approval. Requests do not complete without your local approval.",
    "userGuide.notice.title": "Notes",
    "userGuide.notice.copy": "Only download the app from a trusted page; do not forward binding links; keep your local device online after installation; if the status is offline, reopen the app or bind again.",
    "flow.kicker": "Use Flow",
    "flow.title": "From download to one local StoryLock approval",
    "flow.item1": "Click download on Yian to get the Yian app package.",
    "flow.item2": "Install the app on your local device and allow installation from the trusted current source.",
    "flow.item3": "Return to Yian, open the binding entry, and follow the local device prompts.",
    "flow.item4": "After binding, use the request status page to check whether the local device is online and whether requests are pending.",
    "flow.item5": "When an approval request appears later, the private assistant explains the source, content, and risk first.",
    "flow.item6": "If it looks right, StoryLock Local Core completes approval with local device unlock, biometrics, or device credential.",
    "flowGallery.kicker": "Diagrams",
    "flowGallery.title": "Put download, setup, approval, and saving on one route map",
    "flowGallery.copy1": "These diagrams explain the full StoryLock package path: download locally, choose the current package, enter empty mode, pass the challenge, load content, run learning checks, save the current package, and keep applying it.",
    "flowGallery.copy2": "The main rule is to stay on one package root. Download, setup, unlock, save, and Host application should all point to the directory that contains the same vault.stlk. Open any image to inspect the full-size diagram.",
    "flowGallery.completeAlt": "Complete StoryLock operation flow diagram",
    "flowGallery.completeCaption": "Complete operation flow: from getting a StoryLock package to saving and applying the current package.",
    "flowGallery.overviewAlt": "StoryLock operation overview diagram",
    "flowGallery.overviewTitle": "Overview",
    "flowGallery.overviewCopy": "Understand how one package directory carries download, unlock, save, and application.",
    "flowGallery.downloadAlt": "StoryLock download flow diagram",
    "flowGallery.downloadTitle": "Download",
    "flowGallery.downloadCopy": "After download, check the package structure, local folder, and required files.",
    "flowGallery.setupAlt": "StoryLock setup flow diagram",
    "flowGallery.setupTitle": "Setup",
    "flowGallery.setupCopy": "Choose the package directory or vault.stlk, normalize the path, and check conflicts.",
    "flowGallery.approvalAlt": "StoryLock use and approval flow diagram",
    "flowGallery.approvalTitle": "Approval and Save",
    "flowGallery.approvalCopy": "Start in empty mode, pass the challenge, then load, learn, and save the current package.",
    "flowGallery.boundaryTitle": "Security Boundary",
    "flowGallery.boundaryCloudTitle": "Cloud Entry",
    "flowGallery.boundaryCloudCopy": "Shows pages, serves downloads, relays requests, and returns minimal status.",
    "flowGallery.boundaryAssistantTitle": "Private Assistant",
    "flowGallery.boundaryAssistantCopy": "Explains source, content, and risk without directly reading story storage.",
    "flowGallery.boundaryCoreTitle": "StoryLock Core",
    "flowGallery.boundaryCoreCopy": "Handles sensitive content offline and hides answers and protected objects before approval.",
    "flowGallery.troubleTitle": "Check This First",
    "flowGallery.troubleItem1": "After download, verify file name, size, and checksum.",
    "flowGallery.troubleItem2": "When choosing a package, point to one package root or one vault.stlk.",
    "flowGallery.troubleItem3": "Seeing no content in empty mode is expected protection; content loads only after the challenge passes.",
    "flowGallery.troubleItem4": "After saving, Host should keep using the same path. Saving as a new package requires a manual switch.",
    "faq.kicker": "FAQ",
    "faq.title": "Start here when binding or approval does not work",
    "faq.offline.title": "What if the status is offline?",
    "faq.offline.copy": "First confirm that the local device network is available, then open the Yian app and wait a few seconds. If it is still offline, return to Yian and bind again.",
    "faq.bind.title": "Which device should the binding link connect to?",
    "faq.bind.copy": "The binding link connects the current environment to a trusted local device. The important point is that the device is trusted, clearly identified, and intended for handling later requests.",
    "faq.install.title": "What if Android warns during installation?",
    "faq.install.copy": "Only install from a Yian page you trust. Before installing, check the version, file size, and checksum. If the source is unclear, do not install it.",
    "faq.changeDevice.title": "What should I do after changing local devices?",
    "faq.changeDevice.copy": "Install the app on the new local device and bind again from Yian. If the old local device is no longer used, remove the old connection from the available management entry or stop using that device.",
    "runtime.kicker": "Request Status",
    "runtime.title": "Inspect current requests, source, and local device status",
    "runtime.loadGateway": "Load Request Status",
    "runtime.loadRegistrations": "View Device Connections",
    "runtime.refresh": "Refresh Section",
    "runtime.autorefreshOn": "Auto Refresh: On",
    "runtime.autorefreshOff": "Auto Refresh: Off",
    "runtime.requestQueue": "Pending Requests",
    "runtime.requestSource": "Request Source",
    "runtime.currentMode": "Communication",
    "runtime.activeHosts": "Online Devices",
    "runtime.pending": "Pending",
    "runtime.relayTransport": "Relay Transport",
    "runtime.relayWait": "Default Wait",
    "runtime.relayCoordination": "Coordination",
    "runtime.relayReadiness": "Production Readiness",
    "runtime.waitingPolls": "Waiting Polls",
    "runtime.pendingResponses": "Pending Responses",
    "runtime.group.current": "Current Policy",
    "runtime.group.recent": "Recent Events",
    "runtime.group.total": "Lifetime Stats",
    "runtime.summaryWaiting": "Waiting to generate a runtime summary.",
    "runtime.summaryUnavailable": "Not enough runtime status information is available yet.",
    "runtime.refreshedWaiting": "Last refreshed: waiting",
    "runtime.lastResolved": "Last Resolved",
    "runtime.lastTimeout": "Last Timeout",
    "runtime.lastIdleTimeout": "Last Idle Timeout",
    "runtime.lastReplacedPoll": "Last Replaced Poll",
    "runtime.lastClientClosed": "Last Client Closed Poll",
    "runtime.totalRequests": "Total Requests",
    "runtime.resolvedResponses": "Resolved Responses",
    "runtime.timeoutCount": "Timeout Count",
    "runtime.idleTimeoutCount": "Idle Timeout Count",
    "runtime.replacedPollCount": "Replaced Poll Count",
    "runtime.clientClosedPollCount": "Client Closed Poll Count",
    "runtime.responseTitle": "View Raw JSON",
    "runtime.responseNote": "This area shows request, binding, and online status. For normal use, focus on pending requests, trusted source, and whether the device is online.",
    "help.kicker": "Help",
    "help.title": "Use the same page flow for viewing, downloading, binding, and approving",
    "help.copy1": "Yian sends important approval requests back to the user's own local device. After downloading the app and completing one binding step, users can review the request source, understand the content, and decide whether to continue.",
    "help.copy2": "Help now uses the same navigation, language switch, and previous/next behavior as Product, Safety, Download, and the other pages instead of opening a separate page.",
    "help.imageAlt": "Diagram of the Yian and StoryLock local approval relationship",
    "help.start.title": "Quick Start",
    "help.start.step1": "Choose the Windows local host prototype, Android package, or Linux prototype package for this device.",
    "help.start.step2": "Check the displayed version, file size, and checksum before installing.",
    "help.start.step3": "After installing the Yian app, return to Yian and open the binding entry.",
    "help.start.step4": "After binding, use Request Status to confirm that the local device is online.",
    "help.approve.title": "Approve Requests",
    "help.approve.step1": "Review the request source, operation, and risk prompt on the local device.",
    "help.approve.step2": "Continue only when the request matches the operation you are performing.",
    "help.approve.step3": "Complete approval with local unlock, biometrics, or device credentials.",
    "help.notice.title": "Notes",
    "help.notice.item1": "Only download packages from a trusted Yian page.",
    "help.notice.item2": "Do not forward binding links to other people.",
    "help.notice.item3": "If you do not recognize a request, do not approve it until you verify the source.",
    "pager.prev": "Previous",
    "pager.next": "Next",
    "footer.brand": "Yian",
    "footer.copy": "Download the Yian app, bind your local device, and inspect current request status.",
    "footer.androidHost": "Yian App",
    "footer.registrations": "Device Connections",
    "footer.help": "Help",
    "footer.observeDebug": "Observe & Debug",
    "observeDebug.kicker": "Observe & Debug",
    "observeDebug.title": "Keep host and web test entry points here",
    "observeDebug.subnav.runtime": "Runtime",
    "observeDebug.subnav.webTests": "Web Integration",
    "observeDebug.subnav.hostTests": "Host Integration",
    "observeDebug.runtimeTitle": "Connection and Runtime Status",
    "observeDebug.runtimeCopy": "Use this area to inspect gateway, relay, device connections, and current request status.",
    "observeDebug.webTestsTitle": "Web Test Entry Points",
    "observeDebug.webTestsCopy": "Keep the StoryLock workbench and Agent console available here for demos and integration checks.",
    "observeDebug.storylockWorkbench": "StoryLock Workbench",
    "observeDebug.agentConsole": "Agent Console",
    "observeDebug.hostTestsTitle": "Host Debug Notes",
    "observeDebug.hostTestsCopy": "Windows Host, local relay, and device-state observation now live here so the main website stays focused.",
    "observeDebug.hostActions.health": "Load Host Health",
    "observeDebug.hostActions.status": "Load Host Status",
    "observeDebug.hostActions.diagnostics": "Load Host Diagnostics",
    "observeDebug.hostActions.refresh": "Refresh Section",
    "observeDebug.hostActions.autorefreshOn": "Auto Refresh: On",
    "observeDebug.hostActions.autorefreshOff": "Auto Refresh: Off",
    "observeDebug.hostSummaryWaiting": "Waiting to generate a Host summary.",
    "observeDebug.hostRefreshedWaiting": "Last refreshed: waiting",
    "observeDebug.hostGroup.endpoints": "Local Endpoints",
    "observeDebug.hostGroup.current": "Current Status",
    "observeDebug.hostGroup.notes": "Recent Errors",
    "observeDebug.hostEndpointActions.openManagement": "Open Management",
    "observeDebug.hostEndpointActions.copyStatus": "Copy Status API",
    "observeDebug.hostEndpointActions.copyDiagnostics": "Copy Diagnostics API",
    "observeDebug.hostSummary.managementUrl": "Management",
    "observeDebug.hostSummary.statusUrl": "Status API",
    "observeDebug.hostSummary.diagnosticsUrl": "Diagnostics API",
    "observeDebug.hostSummary.gatewayUrl": "Gateway",
    "observeDebug.hostSummary.online": "Local Host",
    "observeDebug.hostSummary.relay": "Relay Status",
    "observeDebug.hostSummary.mode": "Runtime Mode",
    "observeDebug.hostSummary.lastError": "Last Error",
    "observeDebug.hostTestsItem1": "Check whether devices are online before looking at pending requests.",
    "observeDebug.hostTestsItem2": "Open the workbench or console only for integration checks, not from the main hero flow.",
    "observeDebug.hostTestsItem3": "Raw JSON stays here for observation and troubleshooting, not as a normal user entry.",
    unconfigured: "Not configured",
    unavailable: "Unavailable",
    connectionRelay: "Cloud relay connection",
    connectionDeepLink: "Local binding launch",
    connectionLocal: "Local direct connection",
    siteLoaded: "The site is loaded.",
    gatewayMissing: "The live request status is not available right now. You can still use the download and binding entries.",
    siteOnly: "The page is not currently connected to the request status service, but it still shows the usage guide, download entry, and binding entry.",
    waiting: "Waiting for request status.",
  },
};

const STORAGE_KEY = "yian-site-locale";

const el = {
  output: document.querySelector("[data-region='output']"),
  runtimeSummary: document.querySelector("[data-region='runtime-summary']"),
  hostSummary: document.querySelector("[data-region='host-summary']"),
  runtimeRefreshed: document.querySelector("[data-region='runtime-refreshed']"),
  hostRefreshed: document.querySelector("[data-region='host-refreshed']"),
  connectMode: document.querySelector("[data-field='connect-mode']"),
  requestQueue: document.querySelector("[data-field='request-queue']"),
  requestSource: document.querySelector("[data-field='request-source']"),
  activeHostCount: document.querySelector("[data-field='active-host-count']"),
  relayTransport: document.querySelector("[data-field='relay-transport']"),
  relayWait: document.querySelector("[data-field='relay-wait']"),
  relayCoordination: document.querySelector("[data-field='relay-coordination']"),
  relayReadiness: document.querySelector("[data-field='relay-readiness']"),
  waitingPolls: document.querySelector("[data-field='waiting-polls']"),
  pendingResponses: document.querySelector("[data-field='pending-responses']"),
  lastResolved: document.querySelector("[data-field='last-resolved']"),
  lastTimeout: document.querySelector("[data-field='last-timeout']"),
  lastIdleTimeout: document.querySelector("[data-field='last-idle-timeout']"),
  lastReplacedPoll: document.querySelector("[data-field='last-replaced-poll']"),
  lastClientClosed: document.querySelector("[data-field='last-client-closed']"),
  totalRequests: document.querySelector("[data-field='total-requests']"),
  resolvedResponses: document.querySelector("[data-field='resolved-responses']"),
  timeoutCount: document.querySelector("[data-field='timeout-count']"),
  idleTimeoutCount: document.querySelector("[data-field='idle-timeout-count']"),
  replacedPollCount: document.querySelector("[data-field='replaced-poll-count']"),
  clientClosedPollCount: document.querySelector("[data-field='client-closed-poll-count']"),
  hostOnline: document.querySelector("[data-field='host-online']"),
  hostRelay: document.querySelector("[data-field='host-relay']"),
  hostMode: document.querySelector("[data-field='host-mode']"),
  hostLastError: document.querySelector("[data-field='host-last-error']"),
  hostManagementUrl: document.querySelector("[data-field='host-management-url']"),
  hostStatusUrl: document.querySelector("[data-field='host-status-url']"),
  hostDiagnosticsUrl: document.querySelector("[data-field='host-diagnostics-url']"),
  hostGatewayUrl: document.querySelector("[data-field='host-gateway-url']"),
  downloadMetadata: document.querySelector("[data-region='download-metadata']"),
  localeButtons: Array.from(document.querySelectorAll("[data-locale]")),
  metaDescription: document.querySelector("meta[name='description']"),
  pageSections: Array.from(document.querySelectorAll("[data-page-section]")),
  pageLinks: Array.from(document.querySelectorAll("[data-page-link]")),
  pageTitle: document.querySelector("[data-region='page-title']"),
  subpageSections: Array.from(document.querySelectorAll("[data-subpage-section]")),
  subpageLinks: Array.from(document.querySelectorAll("[data-subpage-link]")),
};

let currentLocale = resolveInitialLocale();
let currentPageIndex = 0;
let currentSubpageId = "runtime";
let runtimeAutoRefreshEnabled = false;
let hostAutoRefreshEnabled = false;
let runtimeAutoRefreshTimer = null;
let hostAutoRefreshTimer = null;
let touchStartX = null;
let touchStartY = null;
let currentDownloadPlatforms = null;

function resolveInitialLocale() {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved && I18N[saved]) {
    return saved;
  }
  return document.documentElement.lang === "zh-CN" ? "zh" : "en";
}

function t(key) {
  return I18N[currentLocale][key] ?? I18N.en[key] ?? key;
}

function applyLocale(locale) {
  currentLocale = I18N[locale] ? locale : "zh";
  localStorage.setItem(STORAGE_KEY, currentLocale);
  document.documentElement.lang = currentLocale === "zh" ? "zh-CN" : "en";
  document.title = t("pageTitle");
  if (el.metaDescription) {
    el.metaDescription.setAttribute("content", t("pageDescription"));
  }

  document.querySelectorAll("[data-i18n]").forEach((node) => {
    node.textContent = t(node.dataset.i18n);
  });
  document.querySelectorAll("[data-i18n-alt]").forEach((node) => {
    node.setAttribute("alt", t(node.dataset.i18nAlt));
  });
  el.localeButtons.forEach((button) => {
    button.classList.toggle("is-active", button.dataset.locale === currentLocale);
  });
  if (currentDownloadPlatforms) {
    renderDownloadMetadata(currentDownloadPlatforms);
  }
  updatePagerTitle();
  updateAutoRefreshButtons();
}

function pageIdFromHash() {
  const raw = window.location.hash.replace(/^#/, "");
  return raw || "home";
}

function pageIndexById(pageId) {
  const index = el.pageSections.findIndex((section) => section.id === pageId);
  return index >= 0 ? index : 0;
}

function updatePagerTitle() {
  const section = el.pageSections[currentPageIndex];
  if (!section || !el.pageTitle) {
    return;
  }
  el.pageTitle.textContent = t(section.dataset.pageTitle ?? "nav.home");
}

function setActivePage(pageId, options = {}) {
  const nextIndex = pageIndexById(pageId);
  currentPageIndex = nextIndex;
  const activeSection = el.pageSections[currentPageIndex];

  el.pageSections.forEach((section, index) => {
    section.classList.toggle("is-active", index === currentPageIndex);
  });
  el.pageLinks.forEach((link) => {
    link.classList.toggle("is-active", link.dataset.pageLink === activeSection?.id);
  });
  updatePagerTitle();

  if (!options.skipHash && activeSection && window.location.hash !== `#${activeSection.id}`) {
    history.pushState(null, "", `#${activeSection.id}`);
  }
  if (!options.skipScroll) {
    window.scrollTo({ top: 0, behavior: "smooth" });
  }
}

function setActiveSubpage(subpageId) {
  const nextId = el.subpageSections.some((section) => section.dataset.subpageSection === subpageId)
    ? subpageId
    : "runtime";
  currentSubpageId = nextId;
  el.subpageSections.forEach((section) => {
    section.classList.toggle("is-active", section.dataset.subpageSection === nextId);
  });
  el.subpageLinks.forEach((button) => {
    button.classList.toggle("is-active", button.dataset.subpageLink === nextId);
  });
  syncAutoRefreshTimers();
}

function movePage(step) {
  if (!el.pageSections.length) {
    return;
  }
  const nextIndex = (currentPageIndex + step + el.pageSections.length) % el.pageSections.length;
  setActivePage(el.pageSections[nextIndex].id);
}

function setOutput(value) {
  if (!el.output) {
    return;
  }
  el.output.textContent = typeof value === "string" ? value : JSON.stringify(value, null, 2);
}

function setSummaryBanner(node, value, state = "neutral") {
  if (!node) {
    return;
  }
  node.textContent = value && String(value).trim() ? String(value) : t("runtime.summaryUnavailable");
  node.classList.remove("is-good", "is-warn", "is-bad");
  if (state === "good") {
    node.classList.add("is-good");
  } else if (state === "warn") {
    node.classList.add("is-warn");
  } else if (state === "bad") {
    node.classList.add("is-bad");
  }
}

function setRefreshedMeta(node, date = new Date()) {
  if (!node) {
    return;
  }
  const locale = currentLocale === "zh" ? "zh-CN" : "en-US";
  const label = currentLocale === "zh" ? "最近刷新：" : "Last refreshed: ";
  node.textContent = `${label}${new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(date)}`;
}

function updateAutoRefreshButtons() {
  const runtimeButton = document.querySelector("[data-action='toggle-runtime-autorefresh']");
  const hostButton = document.querySelector("[data-action='toggle-host-autorefresh']");
  if (runtimeButton) {
    runtimeButton.textContent = t(runtimeAutoRefreshEnabled ? "runtime.autorefreshOn" : "runtime.autorefreshOff");
  }
  if (hostButton) {
    hostButton.textContent = t(hostAutoRefreshEnabled ? "observeDebug.hostActions.autorefreshOn" : "observeDebug.hostActions.autorefreshOff");
  }
}

function syncAutoRefreshTimers() {
  if (runtimeAutoRefreshTimer) {
    clearInterval(runtimeAutoRefreshTimer);
    runtimeAutoRefreshTimer = null;
  }
  if (hostAutoRefreshTimer) {
    clearInterval(hostAutoRefreshTimer);
    hostAutoRefreshTimer = null;
  }
  const activePageId = el.pageSections[currentPageIndex]?.id;
  if (document.hidden || activePageId !== "observe-debug") {
    return;
  }
  if (runtimeAutoRefreshEnabled && currentSubpageId === "runtime") {
    runtimeAutoRefreshTimer = setInterval(() => {
      refreshRuntimeSection().catch(() => {});
    }, 5000);
  }
  if (hostAutoRefreshEnabled && currentSubpageId === "host-tests") {
    hostAutoRefreshTimer = setInterval(() => {
      refreshHostSection().catch(() => {});
    }, 5000);
  }
}

function textValue(node) {
  return String(node?.textContent ?? "").trim();
}

function formatBytes(value) {
  const bytes = Number(value);
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return t("unavailable");
  }
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

function platformTitle(platform, metadata) {
  if (platform === "windows") {
    return currentLocale === "zh" ? "Windows 本地宿主原型" : "Windows Host Prototype";
  }
  if (platform === "android") {
    return currentLocale === "zh" ? "Android 手机版本" : "Android Phone Version";
  }
  if (platform === "linux") {
    return currentLocale === "zh" ? "Linux 原型版本" : "Linux Prototype";
  }
  return metadata?.label ?? platform;
}

function releaseBadge(metadata) {
  const channel = String(metadata?.releaseChannel ?? "").toLowerCase();
  if (channel === "prototype" || channel === "internal") {
    return t("apk.metadata.prototype");
  }
  return metadata?.releaseChannel ?? t("unavailable");
}

function metadataRows(metadata) {
  return [
    ["apk.metadata.versionName", metadata?.versionName],
    ["apk.metadata.versionCode", metadata?.versionCode],
    ["apk.metadata.packageKind", metadata?.packageKind],
    ["apk.metadata.releaseChannel", metadata?.releaseChannel],
    ["apk.metadata.fileSizeBytes", formatBytes(metadata?.fileSizeBytes)],
    ["apk.metadata.checksum", metadata?.checksum],
  ];
}

function renderDownloadMetadata(platforms) {
  if (!el.downloadMetadata) {
    return;
  }
  if (!platforms || !Object.keys(platforms).length) {
    el.downloadMetadata.textContent = t("apk.metadata.unavailable");
    return;
  }

  el.downloadMetadata.replaceChildren(...["windows", "android", "linux"].map((platform) => {
    const metadata = platforms[platform] ?? {};
    const card = document.createElement("article");
    card.className = "download-card";

    const header = document.createElement("div");
    header.className = "download-card-head";
    const title = document.createElement("h5");
    title.textContent = platformTitle(platform, metadata);
    const badge = document.createElement("span");
    badge.textContent = releaseBadge(metadata);
    header.append(title, badge);

    const list = document.createElement("dl");
    list.className = "metadata-list";
    metadataRows(metadata).forEach(([labelKey, value]) => {
      const row = document.createElement("div");
      const dt = document.createElement("dt");
      const dd = document.createElement("dd");
      dt.textContent = t(labelKey);
      dd.textContent = value && String(value).trim() ? String(value) : t("unavailable");
      row.append(dt, dd);
      list.append(row);
    });

    const actions = document.createElement("div");
    actions.className = "download-card-actions";
    if (metadata?.downloadUrl) {
      const link = document.createElement("a");
      link.className = "button";
      link.href = metadata.downloadUrl;
      link.textContent = platformTitle(platform, metadata);
      actions.append(link);
    }
    if (metadata?.checksum) {
      const copy = document.createElement("button");
      copy.className = "button";
      copy.type = "button";
      copy.dataset.action = "copy-checksum";
      copy.dataset.checksum = metadata.checksum;
      copy.textContent = t("apk.metadata.copy");
      actions.append(copy);
    }

    card.append(header, list, actions);
    return card;
  }));
}

async function fetchJsonWithFallback(paths) {
  let lastError = new Error("no endpoint succeeded");
  for (const path of paths) {
    try {
      const response = await fetch(path, {
        headers: {
          accept: "application/json",
        },
      });
      const contentType = response.headers.get("content-type") ?? "";
      if (!response.ok || !contentType.includes("application/json")) {
        lastError = new Error(`${path} returned ${response.status}`);
        continue;
      }
      const body = await response.json();
      if (body?.status === "error") {
        lastError = new Error(body.message ?? `${path} returned error`);
        continue;
      }
      return body;
    } catch (error) {
      lastError = error;
    }
  }
  throw lastError;
}

function setText(node, value) {
  if (!node) {
    return;
  }
  node.textContent = value && String(value).trim() ? String(value) : t("unconfigured");
}

function setFieldState(node, state = "neutral") {
  const card = node?.closest("div");
  if (!(card instanceof HTMLElement)) {
    return;
  }
  card.classList.remove("is-good", "is-warn", "is-bad");
  if (state === "good") {
    card.classList.add("is-good");
  } else if (state === "warn") {
    card.classList.add("is-warn");
  } else if (state === "bad") {
    card.classList.add("is-bad");
  }
}

function formatConnectionMode(value) {
  const raw = String(value ?? "").trim();
  if (!raw) {
    return t("unavailable");
  }
  if (raw.includes("relay")) {
    return t("connectionRelay");
  }
  if (raw.includes("deep") || raw.includes("bind")) {
    return t("connectionDeepLink");
  }
  if (raw.includes("local")) {
    return t("connectionLocal");
  }
  return raw;
}

function formatDurationMs(value) {
  const number = Number(value);
  if (!Number.isFinite(number) || number < 0) {
    return t("unavailable");
  }
  return `${number} ms`;
}

function formatTimestamp(value) {
  const raw = String(value ?? "").trim();
  if (!raw) {
    return t("unavailable");
  }
  const parsed = Date.parse(raw);
  if (!Number.isFinite(parsed)) {
    return raw;
  }
  const locale = currentLocale === "zh" ? "zh-CN" : "en-US";
  return new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(new Date(parsed));
}

function formatRelayReadiness(readiness) {
  if (!readiness || typeof readiness !== "object") {
    return t("unavailable");
  }
  if (readiness.singleInstance && readiness.multiInstance) {
    return currentLocale === "zh" ? "单实例/多实例可用" : "Single and multi-instance ready";
  }
  if (readiness.singleInstance && readiness.missingExternalCoordinator) {
    return currentLocale === "zh" ? "单实例可用，生产多实例需外部协调" : "Single-instance ready, multi-instance needs external coordination";
  }
  if (readiness.singleInstance) {
    return currentLocale === "zh" ? "单实例可用" : "Single-instance ready";
  }
  return t("unavailable");
}

function relayReadinessSummary(readiness) {
  if (!readiness || typeof readiness !== "object") {
    return currentLocale === "zh" ? "生产就绪信息未返回" : "readiness data is unavailable";
  }
  if (readiness.singleInstance && readiness.multiInstance) {
    return currentLocale === "zh" ? "单实例和多实例都可用" : "single-instance and multi-instance are ready";
  }
  if (readiness.singleInstance && readiness.missingExternalCoordinator) {
    return currentLocale === "zh" ? "单实例可用，多实例仍需外部协调" : "single-instance is ready, multi-instance still needs external coordination";
  }
  if (readiness.singleInstance) {
    return currentLocale === "zh" ? "当前仅确认单实例可用" : "only single-instance readiness is currently confirmed";
  }
  return currentLocale === "zh" ? "当前未确认生产可用性" : "production readiness is not yet confirmed";
}

function runtimeSummaryText({
  transport,
  readiness,
  waitingPollCount,
  pendingResponseCount,
  lastTimeoutAt,
}) {
  if (!transport && !readiness) {
    return {
      text: t("runtime.summaryUnavailable"),
      state: "neutral",
    };
  }
  const parts = [];
  const rawTransport = String(transport ?? "").trim() || t("unavailable");
  if (currentLocale === "zh") {
    parts.push(`当前 ${rawTransport}`);
    parts.push(relayReadinessSummary(readiness));
    parts.push(`waiting poll ${Number.isFinite(waitingPollCount) ? waitingPollCount : 0}`);
    parts.push(`pending response ${Number.isFinite(pendingResponseCount) ? pendingResponseCount : 0}`);
    parts.push(lastTimeoutAt ? "最近发生过 timeout" : "最近未记录 timeout");
  } else {
    parts.push(`Current transport is ${rawTransport}`);
    parts.push(relayReadinessSummary(readiness));
    parts.push(`waiting polls ${Number.isFinite(waitingPollCount) ? waitingPollCount : 0}`);
    parts.push(`pending responses ${Number.isFinite(pendingResponseCount) ? pendingResponseCount : 0}`);
    parts.push(lastTimeoutAt ? "a timeout was recorded recently" : "no recent timeout was recorded");
  }
  let state = "good";
  if (lastTimeoutAt || Number(waitingPollCount) > 0 || Number(pendingResponseCount) > 0) {
    state = "warn";
  }
  if (relayReadinessState(readiness) === "bad") {
    state = "bad";
  }
  return {
    text: parts.join(currentLocale === "zh" ? "，" : "; "),
    state,
  };
}

function relayReadinessState(readiness) {
  if (!readiness || typeof readiness !== "object") {
    return "neutral";
  }
  if (readiness.singleInstance && readiness.multiInstance) {
    return "good";
  }
  if (readiness.singleInstance && readiness.missingExternalCoordinator) {
    return "warn";
  }
  return readiness.singleInstance ? "warn" : "bad";
}

function relayCoordinationState(value) {
  const raw = String(value ?? "").toLowerCase();
  if (!raw) {
    return "neutral";
  }
  if (raw.includes("redis") || raw.includes("kv") || raw.includes("pubsub")) {
    return "good";
  }
  if (raw.includes("memory") || raw.includes("in_process") || raw.includes("volatile")) {
    return "warn";
  }
  return "neutral";
}

function relayTransportState(value) {
  const raw = String(value ?? "").toLowerCase();
  if (!raw) {
    return "neutral";
  }
  if (raw.includes("long_poll")) {
    return "good";
  }
  if (raw.includes("short_poll")) {
    return "warn";
  }
  return "neutral";
}

function queueCountState(value) {
  const count = Number(value);
  if (!Number.isFinite(count)) {
    return "neutral";
  }
  if (count === 0) {
    return "good";
  }
  if (count <= 2) {
    return "warn";
  }
  return "bad";
}

function eventCountState(value, {
  warnAt = 1,
  badAt = 3,
  zeroState = "good",
} = {}) {
  const count = Number(value);
  if (!Number.isFinite(count)) {
    return "neutral";
  }
  if (count === 0) {
    return zeroState;
  }
  if (count < warnAt) {
    return zeroState;
  }
  if (count < badAt) {
    return "warn";
  }
  return "bad";
}

function timestampState(value, emphasis = "recent_good") {
  const raw = String(value ?? "").trim();
  if (!raw) {
    return emphasis === "recent_bad" ? "good" : "neutral";
  }
  return emphasis === "recent_bad" ? "bad" : "good";
}

function hostOnlineState(value) {
  const raw = String(value ?? "").toLowerCase();
  if (!raw) {
    return "neutral";
  }
  if (raw.includes("ok") || raw.includes("success") || raw.includes("healthy")) {
    return "good";
  }
  return "warn";
}

function hostRelayState(value) {
  const raw = String(value ?? "").toLowerCase();
  if (!raw) {
    return "neutral";
  }
  if (raw.includes("online") || raw.includes("handled_request") || raw.includes("idle")) {
    return "good";
  }
  if (raw.includes("starting") || raw.includes("local_only")) {
    return "warn";
  }
  if (raw.includes("error")) {
    return "bad";
  }
  return "neutral";
}

function hostLastErrorState(value) {
  const raw = String(value ?? "").trim().toLowerCase();
  if (!raw || raw === "none" || raw === "null" || raw === "unavailable") {
    return "good";
  }
  return "bad";
}

function hostSummaryText({
  online,
  relayStatus,
  mode,
  lastError,
}) {
  const hasError = Boolean(String(lastError ?? "").trim()) && !["none", "null", "unavailable"].includes(String(lastError ?? "").trim().toLowerCase());
  const rawMode = String(mode ?? "").trim() || t("unavailable");
  const rawRelay = String(relayStatus ?? "").trim() || t("unavailable");
  const rawOnline = String(online ?? "").trim() || t("unavailable");
  const parts = [];
  if (currentLocale === "zh") {
    parts.push(`本地 Host ${rawOnline}`);
    parts.push(`relay ${rawRelay}`);
    parts.push(`模式 ${rawMode}`);
    parts.push(hasError ? "最近记录到错误" : "最近未记录错误");
  } else {
    parts.push(`Local Host ${rawOnline}`);
    parts.push(`relay ${rawRelay}`);
    parts.push(`mode ${rawMode}`);
    parts.push(hasError ? "an error was recorded recently" : "no recent error was recorded");
  }
  let state = "good";
  if (hasError) {
    state = "bad";
  } else if (hostRelayState(relayStatus) === "warn" || rawMode === "local_only") {
    state = "warn";
  }
  return {
    text: parts.join(currentLocale === "zh" ? "，" : "; "),
    state,
  };
}

function updateGatewaySummary(payload) {
  const pendingRequests = payload?.requestStatus?.pendingCount
    ?? payload?.requests?.pendingCount
    ?? payload?.onlineStatus?.pendingRequestCount
    ?? payload?.pendingRequestCount;
  const requestSource = payload?.requestStatus?.latestSource
    ?? payload?.requests?.latestSource
    ?? payload?.lastRequest?.source
    ?? payload?.latestRequest?.source
    ?? payload?.request?.source;
  const mode = payload?.secondLayerConnection?.activeOption?.mode
    ?? payload?.secondLayerConnection?.preferredMode
    ?? payload?.connection?.activeMode
    ?? payload?.onlineStatus?.activeConnectionMode
    ?? t("unavailable");
  const hostCount = payload?.hostRegistry?.activeHostCount
    ?? payload?.onlineStatus?.activeHostCount
    ?? payload?.activeHostCount;
  const relayPolicy = payload?.relayPolicy ?? {};
  const failureTracking = relayPolicy?.failureTracking ?? {};
  setText(el.connectMode, formatConnectionMode(mode));
  setFieldState(el.connectMode, "good");
  setText(el.requestQueue, Number.isFinite(pendingRequests) ? pendingRequests : t("unavailable"));
  setFieldState(el.requestQueue, queueCountState(pendingRequests));
  setText(el.requestSource, requestSource ?? t("unavailable"));
  setFieldState(el.requestSource, requestSource ? "good" : "neutral");
  setText(el.activeHostCount, Number.isFinite(hostCount) ? hostCount : "0");
  setFieldState(el.activeHostCount, Number(hostCount) > 0 ? "good" : "warn");
  setText(el.relayTransport, relayPolicy?.transport ?? t("unavailable"));
  setFieldState(el.relayTransport, relayTransportState(relayPolicy?.transport));
  setText(
    el.relayWait,
    `${formatDurationMs(relayPolicy?.defaultWaitMs)} / ${formatDurationMs(relayPolicy?.clientTimeoutMs)}`,
  );
  setFieldState(el.relayWait, "good");
  setText(el.relayCoordination, relayPolicy?.coordination ?? t("unavailable"));
  setFieldState(el.relayCoordination, relayCoordinationState(relayPolicy?.coordination));
  setText(el.relayReadiness, formatRelayReadiness(relayPolicy?.readiness));
  setFieldState(el.relayReadiness, relayReadinessState(relayPolicy?.readiness));
  setText(el.waitingPolls, Number.isFinite(failureTracking?.waitingPollCount) ? failureTracking.waitingPollCount : "0");
  setFieldState(el.waitingPolls, queueCountState(failureTracking?.waitingPollCount));
  setText(el.pendingResponses, Number.isFinite(failureTracking?.pendingResponseCount) ? failureTracking.pendingResponseCount : "0");
  setFieldState(el.pendingResponses, queueCountState(failureTracking?.pendingResponseCount));
  setText(el.lastResolved, formatTimestamp(failureTracking?.lastResolvedAt));
  setFieldState(el.lastResolved, timestampState(failureTracking?.lastResolvedAt, "recent_good"));
  setText(el.lastTimeout, formatTimestamp(failureTracking?.lastTimeoutAt));
  setFieldState(el.lastTimeout, timestampState(failureTracking?.lastTimeoutAt, "recent_bad"));
  setText(el.lastIdleTimeout, formatTimestamp(failureTracking?.lastIdleTimeoutAt));
  setFieldState(el.lastIdleTimeout, timestampState(failureTracking?.lastIdleTimeoutAt, "recent_bad"));
  setText(el.lastReplacedPoll, formatTimestamp(failureTracking?.lastReplacedPollAt));
  setFieldState(el.lastReplacedPoll, timestampState(failureTracking?.lastReplacedPollAt, "recent_bad"));
  setText(el.lastClientClosed, formatTimestamp(failureTracking?.lastClientClosedPollAt));
  setFieldState(el.lastClientClosed, timestampState(failureTracking?.lastClientClosedPollAt, "recent_bad"));
  setText(el.totalRequests, Number.isFinite(failureTracking?.totalRequests) ? failureTracking.totalRequests : "0");
  setFieldState(el.totalRequests, Number(failureTracking?.totalRequests) > 0 ? "good" : "neutral");
  setText(el.resolvedResponses, Number.isFinite(failureTracking?.resolvedResponses) ? failureTracking.resolvedResponses : "0");
  setFieldState(el.resolvedResponses, Number(failureTracking?.resolvedResponses) > 0 ? "good" : "neutral");
  setText(el.timeoutCount, Number.isFinite(failureTracking?.timeoutCount) ? failureTracking.timeoutCount : "0");
  setFieldState(el.timeoutCount, eventCountState(failureTracking?.timeoutCount));
  setText(el.idleTimeoutCount, Number.isFinite(failureTracking?.idleTimeoutCount) ? failureTracking.idleTimeoutCount : "0");
  setFieldState(el.idleTimeoutCount, eventCountState(failureTracking?.idleTimeoutCount));
  setText(el.replacedPollCount, Number.isFinite(failureTracking?.replacedPollCount) ? failureTracking.replacedPollCount : "0");
  setFieldState(el.replacedPollCount, eventCountState(failureTracking?.replacedPollCount));
  setText(el.clientClosedPollCount, Number.isFinite(failureTracking?.clientClosedPollCount) ? failureTracking.clientClosedPollCount : "0");
  setFieldState(el.clientClosedPollCount, eventCountState(failureTracking?.clientClosedPollCount));
  const runtimeSummary = runtimeSummaryText({
    transport: relayPolicy?.transport,
    readiness: relayPolicy?.readiness,
    waitingPollCount: failureTracking?.waitingPollCount,
    pendingResponseCount: failureTracking?.pendingResponseCount,
    lastTimeoutAt: failureTracking?.lastTimeoutAt,
  });
  setSummaryBanner(el.runtimeSummary, runtimeSummary.text, runtimeSummary.state);
  setRefreshedMeta(el.runtimeRefreshed);
}

function updateHostSummary(payload) {
  const result = payload?.result ?? payload ?? {};
  const host = result?.host ?? {};
  const relay = result?.relay ?? result?.ui?.relay ?? {};
  const remote = result?.remote ?? result?.ui?.remote ?? {};
  const ui = result?.ui ?? {};
  const localEndpoints = result?.localEndpoints ?? {};
  const online = host?.status ?? payload?.status ?? null;
  const relayStatus = relay?.status ?? t("unavailable");
  const mode = remote?.mode ?? t("unavailable");
  const lastError = relay?.lastError ?? t("unavailable");
  setText(el.hostOnline, online ?? t("unavailable"));
  setFieldState(el.hostOnline, hostOnlineState(online));
  setText(el.hostRelay, relayStatus);
  setFieldState(el.hostRelay, hostRelayState(relayStatus));
  setText(el.hostMode, formatConnectionMode(mode));
  setFieldState(el.hostMode, remote?.mode === "relay_url" ? "good" : "warn");
  setText(el.hostLastError, lastError);
  setFieldState(el.hostLastError, hostLastErrorState(lastError));
  setText(el.hostManagementUrl, ui?.managementUrl ?? localEndpoints?.management ?? t("unavailable"));
  setFieldState(el.hostManagementUrl, ui?.managementUrl || localEndpoints?.management ? "good" : "neutral");
  setText(el.hostStatusUrl, ui?.statusUrl ?? t("unavailable"));
  setFieldState(el.hostStatusUrl, ui?.statusUrl ? "good" : "neutral");
  setText(el.hostDiagnosticsUrl, localEndpoints?.diagnostics ?? t("unavailable"));
  setFieldState(el.hostDiagnosticsUrl, localEndpoints?.diagnostics ? "good" : "neutral");
  setText(el.hostGatewayUrl, remote?.gatewayUrl ?? t("unavailable"));
  setFieldState(el.hostGatewayUrl, remote?.gatewayUrl ? "good" : "warn");
  const hostSummary = hostSummaryText({
    online,
    relayStatus,
    mode: remote?.mode ?? mode,
    lastError,
  });
  setSummaryBanner(el.hostSummary, hostSummary.text, hostSummary.state);
  setRefreshedMeta(el.hostRefreshed);
}

async function loadGatewayStatus() {
  const payload = await fetchJsonWithFallback([
    "/api/site/gateway-status",
    "/api/storylock-gateway",
  ]);
  updateGatewaySummary(payload);
  setOutput(payload);
}

async function loadRegistrations() {
  const payload = await fetchJsonWithFallback([
    "/app/registrations",
    "/android-host/registrations",
    "/api/site/registrations",
  ]);
  if (Array.isArray(payload?.hosts)) {
    setText(el.activeHostCount, payload.hosts.length);
  }
  setOutput(payload);
}

async function loadHostHealth() {
  const payload = await fetchJsonWithFallback([
    "/api/host-observe/health",
  ]);
  updateHostSummary(payload);
  setOutput(payload);
}

async function loadHostStatus() {
  const payload = await fetchJsonWithFallback([
    "/api/host-observe/status",
  ]);
  updateHostSummary(payload);
  setOutput(payload);
}

async function loadHostDiagnostics() {
  const payload = await fetchJsonWithFallback([
    "/api/host-observe/diagnostics",
  ]);
  updateHostSummary(payload);
  setOutput(payload);
}

async function refreshRuntimeSection() {
  await loadGatewayStatus();
  try {
    await loadRegistrations();
  } catch {
    // Keep the gateway summary visible even if registrations temporarily fail.
  }
}

async function refreshHostSection() {
  await loadHostStatus();
  try {
    await loadHostDiagnostics();
  } catch {
    // Keep the latest host status visible even if diagnostics temporarily fail.
  }
}

async function loadDownloadMetadata() {
  if (!el.downloadMetadata) {
    return;
  }
  el.downloadMetadata.textContent = t("apk.metadata.loading");
  const payload = await fetchJsonWithFallback([
    "/app/download",
  ]);
  currentDownloadPlatforms = payload?.platforms ?? {};
  renderDownloadMetadata(currentDownloadPlatforms);
}

document.addEventListener("click", async (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const locale = target.dataset.locale;
  if (locale) {
    applyLocale(locale);
    return;
  }

  const pageLink = target.closest("[data-page-link]");
  if (pageLink instanceof HTMLElement) {
    event.preventDefault();
    setActivePage(pageLink.dataset.pageLink ?? "home");
    return;
  }

  if (target.matches("[data-action='prev-page']")) {
    movePage(-1);
    return;
  }

  if (target.matches("[data-action='next-page']")) {
    movePage(1);
    return;
  }

  const subpageLink = target.closest("[data-subpage-link]");
  if (subpageLink instanceof HTMLElement) {
    setActiveSubpage(subpageLink.dataset.subpageLink ?? "runtime");
    return;
  }

  try {
    if (target.matches("[data-action='copy-checksum']")) {
      await navigator.clipboard.writeText(target.dataset.checksum ?? "");
      target.textContent = t("apk.metadata.copied");
      return;
    }
    if (target.matches("[data-action='open-host-management']")) {
      const url = textValue(el.hostManagementUrl);
      if (url && url !== t("unavailable") && /^https?:\/\//i.test(url)) {
        window.open(url, "_blank", "noopener");
      }
      return;
    }
    if (target.matches("[data-action='copy-host-status-url']")) {
      const url = textValue(el.hostStatusUrl);
      if (url && url !== t("unavailable")) {
        await navigator.clipboard.writeText(url);
      }
      return;
    }
    if (target.matches("[data-action='copy-host-diagnostics-url']")) {
      const url = textValue(el.hostDiagnosticsUrl);
      if (url && url !== t("unavailable")) {
        await navigator.clipboard.writeText(url);
      }
      return;
    }
    if (target.matches("[data-action='gateway-status']")) {
      await loadGatewayStatus();
    }
    if (target.matches("[data-action='registrations']")) {
      await loadRegistrations();
    }
    if (target.matches("[data-action='refresh-runtime']")) {
      await refreshRuntimeSection();
    }
    if (target.matches("[data-action='toggle-runtime-autorefresh']")) {
      runtimeAutoRefreshEnabled = !runtimeAutoRefreshEnabled;
      updateAutoRefreshButtons();
      syncAutoRefreshTimers();
      return;
    }
    if (target.matches("[data-action='host-health']")) {
      await loadHostHealth();
    }
    if (target.matches("[data-action='host-status']")) {
      await loadHostStatus();
    }
    if (target.matches("[data-action='host-diagnostics']")) {
      await loadHostDiagnostics();
    }
    if (target.matches("[data-action='refresh-host']")) {
      await refreshHostSection();
    }
    if (target.matches("[data-action='toggle-host-autorefresh']")) {
      hostAutoRefreshEnabled = !hostAutoRefreshEnabled;
      updateAutoRefreshButtons();
      syncAutoRefreshTimers();
      return;
    }
  } catch (error) {
    setOutput([
      t("siteOnly"),
      "",
      error?.message ?? String(error),
    ].join("\n"));
  }
});

window.addEventListener("hashchange", () => {
  setActivePage(pageIdFromHash(), { skipHash: true });
});

document.addEventListener("visibilitychange", () => {
  syncAutoRefreshTimers();
});

document.addEventListener("keydown", (event) => {
  if (event.key === "ArrowLeft") {
    movePage(-1);
  }
  if (event.key === "ArrowRight") {
    movePage(1);
  }
});

document.addEventListener("touchstart", (event) => {
  if (event.touches.length !== 1) {
    return;
  }
  touchStartX = event.touches[0].clientX;
  touchStartY = event.touches[0].clientY;
}, { passive: true });

document.addEventListener("touchend", (event) => {
  if (touchStartX === null || touchStartY === null || event.changedTouches.length !== 1) {
    return;
  }
  const dx = event.changedTouches[0].clientX - touchStartX;
  const dy = event.changedTouches[0].clientY - touchStartY;
  touchStartX = null;
  touchStartY = null;

  if (Math.abs(dx) < 56 || Math.abs(dx) < Math.abs(dy) * 1.4) {
    return;
  }
  movePage(dx < 0 ? 1 : -1);
}, { passive: true });

applyLocale(currentLocale);
setActivePage(pageIdFromHash(), { skipHash: true, skipScroll: true });
setActiveSubpage(currentSubpageId);
setOutput(t("waiting"));
updateAutoRefreshButtons();

loadDownloadMetadata().catch(() => {
  if (el.downloadMetadata) {
    el.downloadMetadata.textContent = t("apk.metadata.unavailable");
  }
});
