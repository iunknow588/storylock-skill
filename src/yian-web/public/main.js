const I18N = {
  zh: {
    pageTitle: "易安 | Yian",
    pageDescription: "易安帮助普通用户下载易安 App、完成本地设备绑定、查看连接状态，并在本地设备上确认需要授权的请求。",
    "header.kicker": "StoryLock 本地确认",
    "header.brand": "易安",
    "nav.overview": "产品说明",
    "nav.architecture": "安全方式",
    "nav.binding": "下载绑定",
    "nav.apk": "安装版本",
    "nav.faq": "常见问题",
    "nav.runtime": "连接状态",
    "hero.kicker": "StoryLock 本地安全确认",
    "hero.imageAlt": "第三方 Agent 通过云平台上的易安远程入口与私人智能助理双向通信，StoryLock 本地核心只与私人智能助理本地通信的示意图",
    "hero.title": "用易安把 StoryLock 绑定到你的本地设备",
    "hero.copy": "下载易安 App，完成一次绑定。之后第三方 Agent 或云服务发起的敏感请求，会先到你的私人智能助理，再由 StoryLock 本地核心确认后继续。",
    "hero.download": "下载易安 App",
    "hero.bind": "打开绑定入口",
    "hero.status": "查看连接状态",
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
    "binding.step2.copy": "点击“打开绑定入口”，系统会唤起本地设备上的私人智能助理并写入本次绑定信息。绑定链接只给本人使用，不要转发。",
    "binding.step3.title": "3. 完成注册",
    "binding.step3.copy": "私人智能助理完成首次注册后，回到易安查看连接状态。若显示离线，请打开 App、检查网络，必要时重新绑定。",
    "apk.kicker": "安装版本",
    "apk.title": "安装说明与当前版本",
    "apk.install.title": "安装流程",
    "apk.install.step1": "下载易安 App 安装包。",
    "apk.install.step2": "在 Android 设备上安装并允许来自当前来源的安装。",
    "apk.install.step3": "从易安打开绑定链接，按本地设备提示完成绑定。",
    "apk.install.step4": "完成首次注册后，回到站点查看 App 在线状态。",
    "apk.rules.title": "版本信息",
    "apk.rules.copy": "下载前请核对页面展示的版本号、文件大小和校验值。当前如果还没有正式安装包，下载入口会提示未配置或跳转到已配置的下载地址。",
    "apk.rules.path": "文件来源：以页面下载入口实际提供的安装包为准。",
    "apk.rules.name": "文件名称：优先使用带版本号的易安 App 安装包。",
    "apk.rules.version": "版本号：以本页面“连接状态”区域展示的信息为准。",
    "apk.rules.policy": "测试版只用于安装验证；正式候选版用于对外试用。安装前请核对版本、来源和校验值。",
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
    "flow.item4": "绑定完成后，在连接状态区域查看 App 是否在线、版本是否正确。",
    "flow.item5": "之后遇到需要授权的操作时，先由私人智能助理解释请求来源、内容和风险。",
    "flow.item6": "确认无误后，再由 StoryLock 本地核心配合本地设备解锁、生物识别或设备凭据完成确认。",
    "faq.kicker": "常见问题",
    "faq.title": "绑定或确认遇到问题时先看这里",
    "faq.offline.title": "状态显示离线怎么办？",
    "faq.offline.copy": "先确认本地设备网络可用，再打开易安 App 等待几秒。如果仍然离线，回到易安重新打开绑定入口完成一次新的绑定。",
    "faq.bind.title": "绑定链接可以发给别人吗？",
    "faq.bind.copy": "不可以。绑定链接只用于把当前使用环境连接到你的本地设备，转发给别人可能导致错误绑定。发现链接外泄时，请重新绑定并停止使用旧链接。",
    "faq.install.title": "安装时提示风险怎么办？",
    "faq.install.copy": "只从你确认可信的易安页面下载安装包。安装前核对版本号、文件大小和校验值；如果来源不确定，先不要安装。",
    "faq.changeDevice.title": "更换本地设备后怎么处理？",
    "faq.changeDevice.copy": "在新本地设备上重新下载并安装 App，然后从易安重新绑定。旧本地设备不再使用时，请在可用的管理入口中移除旧连接，或停止使用旧设备。",
    "runtime.kicker": "连接状态",
    "runtime.title": "查看当前 App 版本与本地设备连接情况",
    "runtime.loadGateway": "加载连接状态",
    "runtime.loadRegistrations": "查看设备连接",
    "runtime.currentMode": "连接方式",
    "runtime.activeHosts": "在线设备",
    "runtime.downloadEntry": "下载入口",
    "runtime.bindEntry": "绑定入口",
    "runtime.apkVersion": "APK 版本",
    "runtime.apkFileSize": "APK 大小",
    "runtime.apkPackage": "版本类型",
    "runtime.apkChecksum": "校验值",
    "runtime.pending": "等待加载",
    "runtime.responseTitle": "状态详情",
    "runtime.responseNote": "这里用于查看当前下载、绑定和在线状态；普通使用时只需确认版本、下载入口和在线数量。",
    "footer.brand": "易安 Yian",
    "footer.copy": "用于下载易安 App、完成本地设备绑定，并查看当前连接状态。",
    "footer.androidHost": "易安 App",
    "footer.registrations": "设备连接",
    unconfigured: "未配置",
    unavailable: "未获取",
    connectionRelay: "云端中转连接",
    connectionDeepLink: "本地唤起绑定",
    connectionLocal: "本地直连",
    siteLoaded: "站点已加载。",
    gatewayMissing: "当前未读取到实时连接状态，可稍后再试或直接使用下载与绑定入口。",
    siteOnly: "当前未连接到状态服务，页面仍可用于查看使用说明、下载和绑定入口。",
    waiting: "等待连接状态。",
  },
  en: {
    pageTitle: "Yian | StoryLock Local Approval",
    pageDescription: "Yian helps regular users download the Yian app, bind their local device, inspect connection status, and approve sensitive requests on the local device.",
    "header.kicker": "StoryLock Local Approval",
    "header.brand": "Yian",
    "nav.overview": "Product",
    "nav.architecture": "Safety",
    "nav.binding": "Download",
    "nav.apk": "Install",
    "nav.faq": "FAQ",
    "nav.runtime": "Status",
    "hero.kicker": "StoryLock Local Safety",
    "hero.imageAlt": "Diagram showing third-party agents calling Yian Remote Entry, bidirectional communication with a private assistant, and local-only communication with the offline StoryLock Local Core",
    "hero.title": "Bind StoryLock to your local device with Yian",
    "hero.copy": "Download the Yian app and complete one binding step. After that, sensitive requests from third-party agents or cloud services go to your private assistant first, then continue only after StoryLock Local Core approves them.",
    "hero.download": "Download Yian App",
    "hero.bind": "Open Binding Entry",
    "hero.status": "View Connection Status",
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
    "binding.step2.copy": "Click the binding entry. It opens the private assistant on your local device and writes this binding information. The binding link is for you only; do not forward it.",
    "binding.step3.title": "3. Complete registration",
    "binding.step3.copy": "After first registration, return to Yian and check connection status. If it shows offline, open the app, check the network, or bind again.",
    "apk.kicker": "Install Version",
    "apk.title": "Install guide and current version",
    "apk.install.title": "Install Flow",
    "apk.install.step1": "Download the Yian app package.",
    "apk.install.step2": "Install it on Android and allow installs from the current source.",
    "apk.install.step3": "Open the Yian binding link and follow the local device prompts to finish binding.",
    "apk.install.step4": "After first registration, return to the site and inspect app online status.",
    "apk.rules.title": "Version Information",
    "apk.rules.copy": "Before downloading, check the displayed version, file size, and checksum. If no public package is configured yet, the download entry will show that state or redirect to the configured download address.",
    "apk.rules.path": "File source: use the package actually provided by the download entry.",
    "apk.rules.name": "File name: prefer a Yian app package with a visible version number.",
    "apk.rules.version": "Version: use the information displayed in the Status section on this page.",
    "apk.rules.policy": "Test builds are for installation checks. Release candidates are for public trial use. Verify version, source, and checksum before installing.",
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
    "flow.item4": "After binding, check whether the app is online and whether the version is correct.",
    "flow.item5": "When an approval request appears later, the private assistant explains the source, content, and risk first.",
    "flow.item6": "If it looks right, StoryLock Local Core completes approval with local device unlock, biometrics, or device credential.",
    "faq.kicker": "FAQ",
    "faq.title": "Start here when binding or approval does not work",
    "faq.offline.title": "What if the status is offline?",
    "faq.offline.copy": "First confirm that the local device network is available, then open the Yian app and wait a few seconds. If it is still offline, return to Yian and bind again.",
    "faq.bind.title": "Can I send the binding link to someone else?",
    "faq.bind.copy": "No. The binding link connects the current environment to your local device. Forwarding it can cause the wrong local device to be bound. If the link leaks, bind again and stop using the old link.",
    "faq.install.title": "What if Android warns during installation?",
    "faq.install.copy": "Only install from a Yian page you trust. Before installing, check the version, file size, and checksum. If the source is unclear, do not install it.",
    "faq.changeDevice.title": "What should I do after changing local devices?",
    "faq.changeDevice.copy": "Install the app on the new local device and bind again from Yian. If the old local device is no longer used, remove the old connection from the available management entry or stop using that device.",
    "runtime.kicker": "Status",
    "runtime.title": "Inspect current app version and local device connection",
    "runtime.loadGateway": "Load Connection Status",
    "runtime.loadRegistrations": "View Device Connections",
    "runtime.currentMode": "Connection Type",
    "runtime.activeHosts": "Online Devices",
    "runtime.downloadEntry": "Download Entry",
    "runtime.bindEntry": "Binding Entry",
    "runtime.apkVersion": "APK Version",
    "runtime.apkFileSize": "APK Size",
    "runtime.apkPackage": "Version Type",
    "runtime.apkChecksum": "Checksum",
    "runtime.pending": "Pending",
    "runtime.responseTitle": "Status Details",
    "runtime.responseNote": "This area shows download, binding, and online status. For normal use, check the version, download entry, and online count.",
    "footer.brand": "Yian",
    "footer.copy": "Download the Yian app, bind your local device, and inspect the current connection status.",
    "footer.androidHost": "Yian App",
    "footer.registrations": "Device Connections",
    unconfigured: "Not configured",
    unavailable: "Unavailable",
    connectionRelay: "Cloud relay connection",
    connectionDeepLink: "Local binding launch",
    connectionLocal: "Local direct connection",
    siteLoaded: "The site is loaded.",
    gatewayMissing: "The live connection status is not available right now. You can still use the download and binding entries.",
    siteOnly: "The page is not currently connected to the status service, but it still shows the usage guide, download entry, and binding entry.",
    waiting: "Waiting for connection status.",
  },
};

const STORAGE_KEY = "yian-site-locale";

const el = {
  output: document.querySelector("[data-region='output']"),
  connectMode: document.querySelector("[data-field='connect-mode']"),
  activeHostCount: document.querySelector("[data-field='active-host-count']"),
  downloadUrl: document.querySelector("[data-field='download-url']"),
  bindUrl: document.querySelector("[data-field='bind-url']"),
  apkVersion: document.querySelector("[data-field='apk-version']"),
  apkFileSize: document.querySelector("[data-field='apk-file-size']"),
  apkPackage: document.querySelector("[data-field='apk-package']"),
  apkChecksum: document.querySelector("[data-field='apk-checksum']"),
  localeButtons: Array.from(document.querySelectorAll("[data-locale]")),
  metaDescription: document.querySelector("meta[name='description']"),
};

let currentLocale = resolveInitialLocale();

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
}

function setOutput(value) {
  if (!el.output) {
    return;
  }
  el.output.textContent = typeof value === "string" ? value : JSON.stringify(value, null, 2);
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

function formatBytes(value) {
  if (!Number.isFinite(value)) {
    return t("unconfigured");
  }
  const units = ["B", "KB", "MB", "GB"];
  let size = value;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  const precision = unitIndex === 0 ? 0 : 1;
  return `${size.toFixed(precision)} ${units[unitIndex]}`;
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

function updateGatewaySummary(payload) {
  const mode = payload?.secondLayerConnection?.activeOption?.mode
    ?? payload?.secondLayerConnection?.preferredMode
    ?? payload?.connection?.activeMode
    ?? payload?.onlineStatus?.activeConnectionMode
    ?? t("unavailable");
  const hostCount = payload?.hostRegistry?.activeHostCount
    ?? payload?.onlineStatus?.activeHostCount
    ?? payload?.activeHostCount;
  const downloadUrl = payload?.appDistribution?.androidAppDownloadUrl
    ?? payload?.onlineStatus?.androidDownloadUrl
    ?? payload?.downloads?.androidHost
    ?? "/app/download";
  const bindUrl = payload?.bindingEndpoint
    ?? payload?.onlineStatus?.bindingEntryUrl
    ?? payload?.endpoints?.binding
    ?? "/app/bind";
  const artifact = payload?.appDistribution?.artifact ?? {};
  const versionName = artifact.versionName ?? payload?.appDistribution?.androidApkVersion;
  const versionCode = artifact.versionCode ?? payload?.appDistribution?.androidApkVersionCode;
  const packageKind = artifact.packageKind;
  const releaseChannel = artifact.releaseChannel;

  setText(el.connectMode, formatConnectionMode(mode));
  setText(el.activeHostCount, Number.isFinite(hostCount) ? hostCount : "0");
  setText(el.downloadUrl, downloadUrl);
  setText(el.bindUrl, bindUrl);
  setText(el.apkVersion, [versionName, versionCode ? `(${versionCode})` : null].filter(Boolean).join(" "));
  setText(el.apkFileSize, formatBytes(artifact.fileSizeBytes));
  setText(el.apkPackage, [packageKind, releaseChannel].filter(Boolean).join(" / "));
  setText(el.apkChecksum, artifact.checksum);
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

  try {
    if (target.matches("[data-action='gateway-status']")) {
      await loadGatewayStatus();
    }
    if (target.matches("[data-action='registrations']")) {
      await loadRegistrations();
    }
  } catch (error) {
    setOutput([
      t("siteOnly"),
      "",
      error?.message ?? String(error),
    ].join("\n"));
  }
});

applyLocale(currentLocale);
setOutput(t("waiting"));

loadGatewayStatus().catch((error) => {
  setOutput([
    t("siteLoaded"),
    t("gatewayMissing"),
    "",
    error?.message ?? String(error),
  ].join("\n"));
});
