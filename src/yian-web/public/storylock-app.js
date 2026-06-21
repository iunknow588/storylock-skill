const nodes = [
  ["地点", "大学城旧书店二楼库房"],
  ["物件", "蓝色电影票根"],
  ["书名", "《小王子》旧书"],
  ["人物", "旧书店店主"],
  ["关系", "多年未见的好友"],
  ["动作", "整理二楼库房"],
  ["季节", "雨后的六月傍晚"],
  ["颜色", "票根边缘是蓝色"],
  ["细节", "票根夹在第 21 页"],
  ["原因", "担心票根被雨水打湿"],
  ["暗号", "星星落在票根背面"],
  ["地点线索", "靠窗的木梯旁"],
  ["回忆", "店里播放老电影配乐"],
  ["时间", "闭店前十分钟"],
  ["触发", "好友说起旧书店"],
  ["验证", "问票根夹在哪本书"],
  ["误导", "不是借书卡"],
  ["补充", "票根背面有铅笔字"],
  ["习惯", "你会把纸片夹进书里"],
  ["风险", "公开信息猜不到页码"],
  ["口令", "蓝票根"],
  ["场景", "书架顶层落灰"],
  ["结果", "重逢时确认身份"],
  ["边界", "不记录真实姓名"],
];

let resources = [
  { id: "wallet-main", name: "主钱包签名密钥", kind: "credential", action: "requestSignature", strength: "高" },
  { id: "mail-login", name: "邮箱登录密码", kind: "password", action: "requestPasswordFill", strength: "中高" },
  { id: "api-token", name: "演示 API Token", kind: "token", action: "requestSignature", strength: "中" },
  { id: "cert-local", name: "本地证书凭据", kind: "certificate", action: "requestSignature", strength: "高" },
];

const state = {
  activeChallenge: [],
  selectedAnswers: new Set(),
  requiredAnswers: new Set(["蓝色电影票根", "《小王子》旧书", "票根夹在第 21 页"]),
  verified: false,
  audit: [],
};

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => [...document.querySelectorAll(selector)];

function addAudit(message) {
  state.audit.unshift({ time: new Date().toLocaleTimeString("zh-CN", { hour12: false }), message });
  renderAudit();
}

function renderAudit() {
  const box = $('[data-region="audit-log"]');
  box.innerHTML = state.audit.map((item) => `<div class="log-line">[${item.time}] ${item.message}</div>`).join("");
}

function renderNodes() {
  const grid = $('[data-region="node-grid"]');
  grid.innerHTML = nodes.map(([type, value], index) => `
    <div class="node-tile">
      <strong>${String(index + 1).padStart(2, "0")} ${type}</strong>
      <span class="muted">${value}</span>
    </div>
  `).join("");
}

function renderResources() {
  const grid = $('[data-region="resource-grid"]');
  grid.innerHTML = resources.map((item) => `
    <div class="resource-item">
      <strong>${item.name}</strong>
      <span class="chip">${item.kind}</span>
      <span class="chip good">${item.action}</span>
      <p>故事锁强度：${item.strength}</p>
    </div>
  `).join("");

  const select = $('[data-field="challenge-resource"]');
  select.innerHTML = resources.map((item) => `<option value="${item.id}">${item.name}</option>`).join("");
}

function renderChallenge() {
  const grid = $('[data-region="challenge-grid"]');
  grid.innerHTML = state.activeChallenge.map((item) => {
    const selected = state.selectedAnswers.has(item.value) ? " is-selected" : "";
    return `<button class="challenge-cell${selected}" type="button" data-answer="${item.value}">
      <strong>${item.type}</strong>
      <p>${item.value}</p>
    </button>`;
  }).join("");
}

function makeChallenge() {
  const fixed = nodes.filter(([, value]) => state.requiredAnswers.has(value));
  const distractors = nodes.filter(([, value]) => !state.requiredAnswers.has(value)).slice(0, 6);
  state.activeChallenge = [...fixed, ...distractors]
    .map(([type, value]) => ({ type, value }))
    .sort((left, right) => left.value.localeCompare(right.value, "zh-CN"));
  state.selectedAnswers.clear();
  state.verified = false;
  $('[data-region="challenge-note"]').textContent = "请选择：票根、书名、页码三个关键线索。";
  $('[data-region="session-status"]').textContent = "未授权";
  $('[data-region="session-status"]').className = "chip warn";
  renderChallenge();
  addAudit("生成九宫格挑战");
}

function verifyChallenge() {
  const ok = state.selectedAnswers.size === state.requiredAnswers.size
    && [...state.requiredAnswers].every((answer) => state.selectedAnswers.has(answer));
  state.verified = ok;
  $('[data-region="challenge-note"]').textContent = ok
    ? "挑战通过，可以签发短时授权会话。"
    : "挑战未通过，请重新核对故事线索。";
  $('[data-region="session-status"]').textContent = ok ? "挑战通过" : "未授权";
  $('[data-region="session-status"]').className = ok ? "chip good" : "chip danger";
  addAudit(ok ? "挑战验证通过" : "挑战验证失败");
}

function issueSession() {
  const resourceId = $('[data-field="challenge-resource"]').value || resources[0].id;
  const action = $('[data-field="challenge-action"]').value;
  const session = {
    status: state.verified ? "issued" : "blocked",
    sessionId: state.verified ? `sl-${Date.now().toString(36)}` : null,
    resourceId,
    action,
    expiresInSeconds: state.verified ? 180 : 0,
    returns: action === "requestSignature" ? "signature_only" : "local_fill_only",
  };
  $('[data-region="session-json"]').textContent = JSON.stringify(session, null, 2);
  addAudit(state.verified ? `签发 ${action} 会话` : "阻止未验证会话签发");
}

function exportBundle() {
  const bundle = {
    product: "StoryLock Local Core",
    exportedAt: new Date().toISOString(),
    storyTitle: $('[data-field="story-title"]').value,
    nodeCount: nodes.length,
    resourceCount: resources.length,
    containsStoryPlaintext: false,
    containsSecrets: false,
    capabilities: ["requestSignature", "requestPasswordFill"],
  };
  $('[data-region="export-json"]').textContent = JSON.stringify(bundle, null, 2);
  addAudit("生成导出包摘要");
}

function scoreStrength() {
  const summary = $('[data-field="story-summary"]').value;
  const recall = Math.min(96, 70 + Math.floor(summary.length / 18));
  const guess = Math.min(94, 68 + new Set(summary).size);
  $('[data-region="recall-score"]').textContent = recall;
  $('[data-region="guess-score"]').textContent = guess;
  $('[data-region="strength-note"]').textContent = "已根据故事长度、细节密度和非公开线索重新估算强度。";
  addAudit("完成故事强度评估");
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
    const action = event.target.closest("[data-action]")?.dataset.action;
    const answer = event.target.closest("[data-answer]")?.dataset.answer;
    if (answer) {
      state.selectedAnswers.has(answer) ? state.selectedAnswers.delete(answer) : state.selectedAnswers.add(answer);
      renderChallenge();
      return;
    }
    if (!action) return;
    if (action === "polish-story") {
      $('[data-field="story-summary"]').value += "\n\n润色建议：保留蓝色票根、旧书页码和闭店前场景，删除容易被公开资料猜到的信息。";
      addAudit("追加草稿润色建议");
    }
    if (action === "generate-nodes" || action === "shuffle-nodes") {
      nodes.push(nodes.shift());
      renderNodes();
      addAudit("刷新 24 节点题集");
    }
    if (action === "score-strength") scoreStrength();
    if (action === "add-resource") {
      resources.push({ id: `resource-${resources.length + 1}`, name: `本地保护对象 ${resources.length + 1}`, kind: "secret", action: "requestSignature", strength: "中" });
      renderResources();
      addAudit("添加保护对象");
    }
    if (action === "new-challenge") makeChallenge();
    if (action === "demo-answer") {
      state.selectedAnswers = new Set(state.requiredAnswers);
      renderChallenge();
      addAudit("填入演示答案");
    }
    if (action === "verify-challenge") verifyChallenge();
    if (action === "issue-session") issueSession();
    if (action === "export-bundle") exportBundle();
    if (action === "clear-log") {
      state.audit = [];
      renderAudit();
    }
  });
}

bindNavigation();
bindActions();
renderNodes();
renderResources();
makeChallenge();
issueSession();
exportBundle();
addAudit("StoryLock 工作台已启动");
