const defaultConfig = {
  product: "Yian Windows Host",
  version: "0.1.0",
  gateway_base_url: "https://yian.cdao.online",
  identity_id: "windows-demo-001",
  device_id: "windows-demo-local",
  preferred_mode: "relay_url",
  host_port: 4510,
  health_url: "http://127.0.0.1:4510/health",
  execute_url: "http://127.0.0.1:4510/execute",
  register_path: "/android-host/register",
  relay_poll_path: "/local-host/relay/poll",
  relay_respond_path: "/local-host/relay/respond",
};

let requests = [
  {
    id: "req-sign-001",
    capability: "requestSignature",
    source: "pharos-agent.demo",
    object: "主钱包签名密钥",
    risk: "中",
    status: "pending",
  },
  {
    id: "req-fill-002",
    capability: "requestPasswordFill",
    source: "browser.local",
    object: "邮箱登录密码",
    risk: "低",
    status: "pending",
  },
];

let selectedRequestId = requests[0].id;
const logs = [];

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => [...document.querySelectorAll(selector)];

function log(message) {
  logs.unshift(`[${new Date().toLocaleTimeString("zh-CN", { hour12: false })}] ${message}`);
  $('[data-region="agent-log"]').innerHTML = logs.map((line) => `<div class="log-line">${line}</div>`).join("");
}

function setChip(region, text, tone) {
  const chip = $(`[data-region="${region}"]`);
  chip.textContent = text;
  chip.className = `chip ${tone}`;
}

function renderConfig(extra = {}) {
  $('[data-region="config-json"]').textContent = JSON.stringify({ ...defaultConfig, ...extra }, null, 2);
}

function renderRequests() {
  $('[data-region="pending-count"]').textContent = requests.filter((item) => item.status === "pending").length;
  $('[data-region="request-list"]').innerHTML = requests.map((item) => {
    const active = item.id === selectedRequestId ? " is-active" : "";
    const tone = item.status === "approved" ? "good" : item.status === "denied" ? "danger" : "warn";
    return `<div class="request-item${active}" data-request-id="${item.id}">
      <strong>${item.capability}</strong>
      <span class="chip ${tone}">${item.status}</span>
      <p>${item.source} -> ${item.object}</p>
    </div>`;
  }).join("");
  renderSelectedRequest();
}

function renderSelectedRequest() {
  const item = requests.find((request) => request.id === selectedRequestId) ?? requests[0];
  $('[data-region="request-detail"]').textContent = JSON.stringify(item ?? {}, null, 2);
}

function renderQuestionSet(imported = false) {
  const items = [
    ["故事节点", imported ? "24 / 24" : "未导入"],
    ["九宫格挑战", imported ? "可生成" : "等待题库"],
    ["本地会话", imported ? "180 秒上限" : "不可签发"],
    ["敏感内容", "不出本机"],
  ];
  $('[data-region="question-set-grid"]').innerHTML = items.map(([name, value]) => `
    <div class="resource-item">
      <strong>${name}</strong>
      <span class="muted">${value}</span>
    </div>
  `).join("");
}

async function refreshStatus() {
  try {
    const [gateway, registrations] = await Promise.all([
      fetch("/api/storylock-gateway").then((response) => response.json()),
      fetch("/app/registrations").then((response) => response.json()),
    ]);
    $('[data-region="gateway-state"]').textContent = gateway.status ?? "online";
    $('[data-region="host-count"]').textContent = String(registrations.registrations?.length ?? registrations.hostCount ?? 0);
    setChip("agent-status", "网关可达", "good");
    renderConfig({
      gatewayStatus: gateway.status ?? "available",
      registrationStatus: registrations.status ?? "loaded",
    });
    log("已读取网关和设备注册状态");
  } catch (error) {
    $('[data-region="gateway-state"]').textContent = "离线";
    setChip("agent-status", "网关不可达", "danger");
    renderConfig({ gatewayError: error.message });
    log(`网关状态读取失败：${error.message}`);
  }
}

async function checkLocalHealth() {
  try {
    const health = await fetch(defaultConfig.health_url, { mode: "cors" }).then((response) => response.json());
    $('[data-region="host-state"]').textContent = health.status ?? "online";
    setChip("agent-status", "本机在线", "good");
    renderConfig({ localHealth: health });
    log("本地宿主健康检查通过");
  } catch (error) {
    $('[data-region="host-state"]').textContent = "未连接";
    setChip("agent-status", "本机未连接", "warn");
    renderConfig({ localHealthError: "无法从浏览器直接访问 127.0.0.1:4510，可能是宿主未启动或 CORS 未开放。" });
    log(`本地健康检查未通过：${error.message}`);
  }
}

function bindNavigation() {
  $$("[data-view-target]").forEach((button) => {
    button.addEventListener("click", () => {
      $$("[data-view-target]").forEach((item) => item.classList.remove("is-active"));
      $$(".view").forEach((item) => item.classList.remove("is-active"));
      button.classList.add("is-active");
      $(`#${button.dataset.viewTarget}`).classList.add("is-active");
    });
  });
}

function bindActions() {
  document.addEventListener("click", (event) => {
    const requestId = event.target.closest("[data-request-id]")?.dataset.requestId;
    if (requestId) {
      selectedRequestId = requestId;
      renderRequests();
      return;
    }
    const action = event.target.closest("[data-action]")?.dataset.action;
    if (!action) return;
    if (action === "refresh-status") refreshStatus();
    if (action === "check-local-health") checkLocalHealth();
    if (action === "simulate-request") {
      const id = `req-demo-${Date.now().toString(36)}`;
      requests.unshift({
        id,
        capability: requests.length % 2 ? "requestSignature" : "requestPasswordFill",
        source: "remote-entry.demo",
        object: "新增演示对象",
        risk: "中",
        status: "pending",
      });
      selectedRequestId = id;
      renderRequests();
      log("加入一条模拟远程请求");
    }
    if (action === "approve-request" || action === "deny-request") {
      const item = requests.find((request) => request.id === selectedRequestId);
      if (item) {
        item.status = action === "approve-request" ? "approved" : "denied";
        renderRequests();
        log(`${item.id} 已${item.status === "approved" ? "批准" : "拒绝"}`);
      }
    }
    if (action === "import-question-set") {
      renderQuestionSet(true);
      log("导入 StoryLock 演示题库状态");
    }
    if (action === "clear-log") {
      logs.length = 0;
      $('[data-region="agent-log"]').innerHTML = "";
    }
  });
}

bindNavigation();
bindActions();
renderConfig();
renderRequests();
renderQuestionSet(false);
log("本地 Agent 控制台已启动");
refreshStatus();
