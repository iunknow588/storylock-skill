const questionSeeds = [
  ["地点", "票根最初被你夹进了哪家店的哪一层？", "大学城旧书店二楼库房"],
  ["物件", "你夹进旧书里的关键物件是什么？", "蓝色电影票根"],
  ["书名", "票根被夹进了哪本旧书？", "《小王子》旧书"],
  ["人物", "当时和你一起整理库房的人是谁？", "旧书店店主"],
  ["关系", "多年后用这个故事确认身份的人是谁？", "多年未见的好友"],
  ["动作", "你把票根夹进书里前正在做什么？", "整理二楼库房"],
  ["季节", "故事发生在什么季节和天气后？", "雨后的六月傍晚"],
  ["颜色", "票根最容易被回忆起来的颜色是什么？", "票根边缘是蓝色"],
  ["细节", "票根夹在旧书的第几页？", "票根夹在第 21 页"],
  ["原因", "你为什么把票根临时夹进书里？", "担心票根被雨水打湿"],
  ["暗号", "票根背面的私人暗号是什么？", "星星落在票根背面"],
  ["地点线索", "库房里你站在什么位置附近？", "靠窗的木梯旁"],
  ["回忆", "店里当时播放的声音线索是什么？", "店里播放老电影配乐"],
  ["时间", "这件事发生在店里什么时间点？", "闭店前十分钟"],
  ["触发", "多年后是什么话题触发了这段回忆？", "好友说起旧书店"],
  ["验证", "身份确认时最核心的问题是什么？", "问票根夹在哪本书"],
  ["误导", "这个物件明确不是什么？", "不是借书卡"],
  ["补充", "票根背面还有什么附加痕迹？", "票根背面有铅笔字"],
  ["习惯", "这个故事对应你的哪种习惯？", "你会把纸片夹进书里"],
  ["风险", "为什么陌生人难以猜中答案？", "公开信息猜不到页码"],
  ["口令", "你们后来给这个线索起的短口令是什么？", "蓝票根"],
  ["场景", "库房里另一个视觉细节是什么？", "书架顶层落灰"],
  ["结果", "这个故事最终用于什么结果？", "重逢时确认身份"],
  ["边界", "这个故事锁刻意不记录什么？", "不记录真实姓名"],
];

const distractorPool = [
  "一楼收银台", "红色借书卡", "《边城》新版", "咖啡店老板", "刚认识的同事",
  "打包新书", "冬天的清晨", "绿色封面", "第 12 页", "避免被风吹走",
  "月亮贴纸", "门口雨伞架", "流行歌曲", "开店后半小时", "朋友提起电影",
  "问书店地址", "不是电影票", "背面没有字", "你会折纸角", "公开资料能猜到",
  "旧木梯", "玻璃柜台反光", "临时借书", "真实身份证号", "黑色钥匙扣",
  "三楼走廊", "蓝色书签", "《海边的卡夫卡》", "社团学长", "整理明信片",
  "秋天夜里", "银色边框", "第 8 页", "怕弄丢钥匙", "太阳图案",
  "靠门的沙发", "电台新闻", "午饭前", "问电影名字", "不是会员卡",
  "背面有印章", "你会写日期", "网上能搜到", "蓝书签",
];

function makeAnswers(correctText, questionIndex) {
  const wrong = distractorPool
    .filter((item) => item !== correctText)
    .slice(questionIndex, questionIndex + 8);
  while (wrong.length < 8) {
    wrong.push(distractorPool[(wrong.length + questionIndex) % distractorPool.length]);
  }
  const answers = [
    { id: "a1", text: correctText, correct: true },
    ...wrong.slice(0, 8).map((text, index) => ({ id: `a${index + 2}`, text, correct: false })),
  ];
  return answers.sort((left, right) => left.text.localeCompare(right.text, "zh-CN"));
}

let nodes = questionSeeds.map(([type, prompt, correctAnswer], index) => ({
  id: `q-${String(index + 1).padStart(2, "0")}`,
  type,
  prompt,
  correctAnswer,
  answers: makeAnswers(correctAnswer, index),
}));

let resources = [
  { id: "blog-read", group: "normal", name: "博客草稿读取授权", kind: "document", action: "requestPasswordFill", strength: "普通", description: "允许本地确认后读取低风险草稿内容。" },
  { id: "mail-login", group: "normal", name: "邮箱登录密码", kind: "password", action: "requestPasswordFill", strength: "普通", description: "用于演示网页凭据填充。" },
  { id: "api-token", group: "private", name: "演示 API Token", kind: "token", action: "requestSignature", strength: "私密", description: "只返回签名或调用结果，不暴露 token。" },
  { id: "cert-local", group: "private", name: "本地证书凭据", kind: "certificate", action: "requestSignature", strength: "私密", description: "用于本机证书操作授权。" },
  { id: "wallet-main", group: "secret", name: "主钱包签名密钥", kind: "credential", action: "requestSignature", strength: "机密", description: "高风险签名对象，需要更严格确认。" },
  { id: "vault-root", group: "secret", name: "本地 Vault 根密钥", kind: "private_key", action: "requestSignature", strength: "机密", description: "仅用于演示对象边界，绝不导出密钥本体。" },
];

const resourceGroups = [
  { id: "normal", label: "普通授权对象", chip: "普通" },
  { id: "private", label: "私密对象", chip: "私密" },
  { id: "secret", label: "机密对象", chip: "机密" },
];

const retentionLearningPolicy = {
  purpose: "防止用户长期不用后忘记问题答案，导出后由本地 Host 按周期强制复习。",
  requiredQuestionCount: 22,
  requiredQuestionCountMeaning: "每次保留学习固定回答 22 个问题，确认用户仍然记得故事锁。",
  frequencyDesign: "先密后疏：按天开始，逐步降低到每周、每月、每年。",
  phaseParameterMeaning: "duration 表示阶段持续多久，frequency 表示该阶段隔多久触发一次复习。",
  phases: [
    { phase: "initial", label: "初始期", duration: "3 天", frequency: "每 1 天" },
    { phase: "consolidation", label: "巩固期", duration: "4 天", frequency: "每 2 天" },
    { phase: "adaptation", label: "适应期", duration: "3 周", frequency: "每 1 周" },
    { phase: "stable", label: "稳定期", duration: "4 个月", frequency: "每 1 个月" },
    { phase: "long_term", label: "长期期", duration: "1 年", frequency: "每 1 年" },
  ],
};

const state = {
  activeChallenge: [],
  selectedAnswerId: null,
  selectedQuestionId: nodes[0].id,
  selectedResourceId: resources[0].id,
  selectedResourceGroup: "normal",
  verified: false,
  audit: [],
};

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => [...document.querySelectorAll(selector)];

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function getSelectedQuestion() {
  return nodes.find((item) => item.id === state.selectedQuestionId) ?? nodes[0];
}

function getSelectedResource() {
  return resources.find((item) => item.id === state.selectedResourceId) ?? resources[0];
}

function addAudit(message) {
  state.audit.unshift({ time: new Date().toLocaleTimeString("zh-CN", { hour12: false }), message });
  renderAudit();
}

function renderAudit() {
  const box = $('[data-region="audit-log"]');
  box.innerHTML = state.audit.map((item) => `<div class="log-line">[${item.time}] ${escapeHtml(item.message)}</div>`).join("");
}

function renderNodes() {
  const grid = $('[data-region="node-grid"]');
  grid.innerHTML = nodes.map((item, index) => {
    const correctCount = item.answers.filter((answer) => answer.correct).length;
    const selected = item.id === state.selectedQuestionId ? " is-active" : "";
    return `
      <button class="node-tile${selected}" type="button" data-question-id="${item.id}">
        <strong>${String(index + 1).padStart(2, "0")} ${escapeHtml(item.type)}</strong>
        <span>${escapeHtml(item.prompt)}</span>
        <small>${correctCount} 个正确答案 / 9 个候选</small>
      </button>
    `;
  }).join("");
  renderQuestionEditor();
  renderQuestionSelect();
}

function renderQuestionEditor() {
  const question = getSelectedQuestion();
  $('[data-region="question-editor-title"]').textContent = `${question.id} ${question.type}`;
  $('[data-region="question-editor-note"]').textContent = question.prompt;
  $('[data-region="answer-editor"]').innerHTML = question.answers.map((answer, index) => `
    <div class="answer-row">
      <label class="answer-text">
        <span>答案 ${index + 1}</span>
        <input data-answer-text="${answer.id}" value="${escapeHtml(answer.text)}">
      </label>
      <button class="answer-state-toggle ${answer.correct ? "is-correct" : "is-wrong"}" type="button" data-answer-state="${answer.id}" aria-pressed="${answer.correct ? "true" : "false"}">
        <input type="checkbox" data-answer-correct="${answer.id}" ${answer.correct ? "checked" : ""} tabindex="-1" aria-hidden="true">
        <span class="answer-state-icon">${answer.correct ? "✓" : "×"}</span>
        <span class="answer-state-text">${answer.correct ? "正确" : "错误"}</span>
      </button>
    </div>
  `).join("");
}

function openAnswerEditor() {
  renderQuestionEditor();
  $('[data-region="answer-modal"]').hidden = false;
  updateModalOpenState();
}

function closeAnswerEditor() {
  $('[data-region="answer-modal"]').hidden = true;
  updateModalOpenState();
}

function openStoryEditor() {
  $('[data-region="story-modal-text"]').value = $('[data-field="story-plot"]').value;
  $('[data-region="story-modal"]').hidden = false;
  updateModalOpenState();
}

function closeStoryEditor() {
  $('[data-region="story-modal"]').hidden = true;
  updateModalOpenState();
}

function saveStoryEditor() {
  const value = $('[data-region="story-modal-text"]').value.trim();
  if (value) {
    $('[data-field="story-plot"]').value = value;
  }
  closeStoryEditor();
  exportBundle();
  addAudit("保存故事内容");
}

function updateModalOpenState() {
  const anyOpen = $$("[data-region='answer-modal'], [data-region='resource-modal'], [data-region='story-modal']")
    .some((modal) => !modal.hidden);
  document.body.classList.toggle("modal-open", anyOpen);
}

function saveQuestionAnswers() {
  const question = getSelectedQuestion();
  question.answers = question.answers.map((answer) => ({
    ...answer,
    text: $(`[data-answer-text="${answer.id}"]`).value.trim() || answer.text,
    correct: $(`[data-answer-correct="${answer.id}"]`).checked,
  }));
  question.correctAnswer = question.answers.find((answer) => answer.correct)?.text ?? question.answers[0].text;
  renderNodes();
  makeChallenge();
  exportBundle();
  closeAnswerEditor();
  addAudit(`保存 ${question.id} 的 9 个答案配置`);
}

function renderResourceGroups() {
  const grid = $('[data-region="resource-grid"]');
  const currentGroup = resourceGroups.find((group) => group.id === state.selectedResourceGroup) ?? resourceGroups[0];
  grid.innerHTML = resourceGroups.map((group) => {
    const items = resources.filter((item) => item.group === group.id);
    const active = group.id === currentGroup.id ? " is-active" : "";
    return `
      <section class="resource-list-panel${active}">
        <div class="resource-list-head">
          <div>
            <h3>${escapeHtml(group.label)}</h3>
            <p>${items.length} 个受保护对象</p>
          </div>
        </div>
        <div class="resource-list">
          ${items.map((item) => {
            const selected = item.id === state.selectedResourceId ? " is-active" : "";
            return `
              <button class="resource-row${selected}" type="button" data-resource-id="${item.id}">
                <span class="resource-row-main">
                  <strong>${escapeHtml(item.name)}</strong>
                  <small>${escapeHtml(item.description)}</small>
                </span>
                <span class="chip">${escapeHtml(item.kind)}</span>
                <span class="chip good">${escapeHtml(item.action)}</span>
                <span class="chip warn">${escapeHtml(item.strength)}</span>
              </button>
            `;
          }).join("")}
        </div>
      </section>
    `;
  }).join("");

  const select = $('[data-field="challenge-resource"]');
  select.innerHTML = resources.map((item) => `<option value="${item.id}">${escapeHtml(item.name)}</option>`).join("");
  select.value = state.selectedResourceId;
  updateResourceNavState();
}

function updateResourceNavState() {
  $$("[data-resource-nav-group]").forEach((item) => {
    item.classList.toggle("is-active", item.dataset.resourceNavGroup === state.selectedResourceGroup);
  });
}

function renderResourceEditor() {
  const item = getSelectedResource();
  $('[data-region="resource-editor-note"]').textContent = `${item.name} · ${item.strength}`;
  $('[data-region="resource-editor"]').innerHTML = `
    <label class="field">
      <span>显示名称</span>
      <input data-resource-field="name" value="${escapeHtml(item.name)}">
    </label>
    <label class="field">
      <span>对象分类</span>
      <select data-resource-field="group">
        ${resourceGroups.map((group) => `<option value="${group.id}" ${group.id === item.group ? "selected" : ""}>${group.label}</option>`).join("")}
      </select>
    </label>
    <label class="field">
      <span>对象类型</span>
      <input data-resource-field="kind" value="${escapeHtml(item.kind)}">
    </label>
    <label class="field">
      <span>授权能力</span>
      <select data-resource-field="action">
        <option value="requestSignature" ${item.action === "requestSignature" ? "selected" : ""}>requestSignature</option>
        <option value="requestPasswordFill" ${item.action === "requestPasswordFill" ? "selected" : ""}>requestPasswordFill</option>
      </select>
    </label>
    <label class="field">
      <span>保护级别</span>
      <input data-resource-field="strength" value="${escapeHtml(item.strength)}">
    </label>
    <label class="field">
      <span>保护说明</span>
      <textarea data-resource-field="description">${escapeHtml(item.description)}</textarea>
    </label>
  `;
}

function openResourceEditor() {
  renderResourceEditor();
  $('[data-region="resource-modal"]').hidden = false;
  updateModalOpenState();
}

function closeResourceEditor() {
  $('[data-region="resource-modal"]').hidden = true;
  updateModalOpenState();
}

function saveResource() {
  const item = getSelectedResource();
  for (const field of ["name", "group", "kind", "action", "strength", "description"]) {
    item[field] = $(`[data-resource-field="${field}"]`).value.trim() || item[field];
  }
  state.selectedResourceGroup = item.group;
  renderResourceGroups();
  exportBundle();
  closeResourceEditor();
  addAudit(`保存保护对象：${item.name}`);
}

function renderQuestionSelect() {
  const select = $('[data-field="challenge-question"]');
  select.innerHTML = nodes.map((item) => `<option value="${item.id}">${item.id} ${escapeHtml(item.type)} - ${escapeHtml(item.prompt)}</option>`).join("");
  select.value = state.selectedQuestionId;
}

function renderChallenge() {
  const grid = $('[data-region="challenge-grid"]');
  const question = getSelectedQuestion();
  grid.innerHTML = state.activeChallenge.map((answer) => {
    const selected = state.selectedAnswerId === answer.id ? " is-selected" : "";
    return `<button class="challenge-cell${selected}" type="button" data-challenge-answer="${answer.id}">
      <strong>${answer.correct ? "候选答案" : "干扰答案"}</strong>
      <p>${escapeHtml(answer.text)}</p>
    </button>`;
  }).join("");
  $('[data-region="challenge-note"]').textContent = `当前问题：${question.prompt}。请选择 1 个答案。`;
}

function makeChallenge() {
  const question = getSelectedQuestion();
  state.activeChallenge = [...question.answers];
  state.selectedAnswerId = null;
  state.verified = false;
  $('[data-region="session-status"]').textContent = "未授权";
  $('[data-region="session-status"]').className = "chip warn";
  renderChallenge();
  addAudit(`生成 ${question.id} 的九宫格答案挑战`);
}

function verifyChallenge() {
  const question = getSelectedQuestion();
  const selected = question.answers.find((answer) => answer.id === state.selectedAnswerId);
  const ok = Boolean(selected?.correct);
  state.verified = ok;
  $('[data-region="challenge-note"]').textContent = ok
    ? `挑战通过：${selected.text}`
    : "挑战未通过，请在这个问题的 9 个答案中选择正确答案。";
  $('[data-region="session-status"]').textContent = ok ? "挑战通过" : "未授权";
  $('[data-region="session-status"]').className = ok ? "chip good" : "chip danger";
  addAudit(ok ? `${question.id} 答案挑战通过` : `${question.id} 答案挑战失败`);
}

function issueSession() {
  const resourceId = $('[data-field="challenge-resource"]').value || resources[0].id;
  const action = $('[data-field="challenge-action"]').value;
  const question = getSelectedQuestion();
  const selected = question.answers.find((answer) => answer.id === state.selectedAnswerId);
  const session = {
    status: state.verified ? "issued" : "blocked",
    sessionId: state.verified ? `sl-${Date.now().toString(36)}` : null,
    resourceId,
    action,
    questionId: question.id,
    selectedAnswer: selected?.text ?? null,
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
    storySummary: $('[data-field="story-summary"]').value,
    storyPlot: $('[data-field="story-plot"]').value,
    questionCount: nodes.length,
    answerCandidatesPerQuestion: 9,
    resourceCount: resources.length,
    resourceGroups: resourceGroups.map((group) => ({
      id: group.id,
      label: group.label,
      count: resources.filter((item) => item.group === group.id).length,
    })),
    containsStoryPlaintext: false,
    containsSecrets: false,
    capabilities: ["requestSignature", "requestPasswordFill"],
    retentionLearningPolicy,
  };
  $('[data-region="export-json"]').textContent = JSON.stringify(bundle, null, 2);
}

function activateView(viewId, updateHash = true) {
  const viewAliases = {
    challenge: "host-challenge",
    session: "host-session",
  };
  const normalizedViewId = viewAliases[viewId] ?? viewId;
  const target = $(`#${normalizedViewId}`);
  if (!target) return;
  $$("[data-view-target]").forEach((item) => {
    const isResourceLevel = Boolean(item.dataset.resourceNavGroup);
    const isActiveView = item.dataset.viewTarget === normalizedViewId && !isResourceLevel;
    item.classList.toggle("is-active", isActiveView);
  });
  $$(".view").forEach((item) => item.classList.toggle("is-active", item.id === normalizedViewId));
  if (normalizedViewId === "resources") updateResourceNavState();
  if (updateHash) history.replaceState(null, "", `#${normalizedViewId}`);
}

function bindNavigation() {
  $$("[data-view-target]").forEach((button) => {
    button.addEventListener("click", () => activateView(button.dataset.viewTarget));
  });
  window.addEventListener("hashchange", () => activateView(location.hash.slice(1) || "draft", false));
}

function bindActions() {
  document.addEventListener("change", (event) => {
    if (event.target.matches('[data-field="challenge-question"]')) {
      state.selectedQuestionId = event.target.value;
      renderNodes();
      makeChallenge();
    }
    if (event.target.matches('[data-field="challenge-resource"]')) {
      state.selectedResourceId = event.target.value;
      state.selectedResourceGroup = getSelectedResource().group;
      renderResourceGroups();
    }
  });

  document.addEventListener("click", (event) => {
    const questionId = event.target.closest("[data-question-id]")?.dataset.questionId;
    const resourceGroup = event.target.closest("[data-resource-nav-group]")?.dataset.resourceNavGroup;
    const resourceId = event.target.closest("[data-resource-id]")?.dataset.resourceId;
    const challengeAnswer = event.target.closest("[data-challenge-answer]")?.dataset.challengeAnswer;
    const answerState = event.target.closest("[data-answer-state]")?.dataset.answerState;
    const action = event.target.closest("[data-action]")?.dataset.action;

    if (questionId) {
      state.selectedQuestionId = questionId;
      renderNodes();
      makeChallenge();
      openAnswerEditor();
      return;
    }
    if (resourceGroup) {
      state.selectedResourceGroup = resourceGroup;
      const firstInGroup = resources.find((item) => item.group === resourceGroup);
      if (firstInGroup) state.selectedResourceId = firstInGroup.id;
      renderResourceGroups();
      activateView("resources");
      return;
    }
    if (resourceId) {
      state.selectedResourceId = resourceId;
      state.selectedResourceGroup = getSelectedResource().group;
      renderResourceGroups();
      openResourceEditor();
      return;
    }
    if (challengeAnswer) {
      state.selectedAnswerId = challengeAnswer;
      renderChallenge();
      return;
    }
    if (answerState) {
      const checkbox = $(`[data-answer-correct="${answerState}"]`);
      checkbox.checked = !checkbox.checked;
      const button = event.target.closest("[data-answer-state]");
      button.classList.toggle("is-correct", checkbox.checked);
      button.classList.toggle("is-wrong", !checkbox.checked);
      button.setAttribute("aria-pressed", checkbox.checked ? "true" : "false");
      button.querySelector(".answer-state-icon").textContent = checkbox.checked ? "✓" : "×";
      button.querySelector(".answer-state-text").textContent = checkbox.checked ? "正确" : "错误";
      return;
    }
    if (!action) return;

    if (action === "polish-story") {
      $('[data-field="story-plot"]').value += "\n\n润色建议：保留蓝色票根、旧书页码和闭店前场景，把 24 个问题的答案自然串进完整情节里，同时删除容易被公开资料猜到的信息。";
      addAudit("追加草稿润色建议");
    }
    if (action === "open-story-editor") openStoryEditor();
    if (action === "generate-nodes" || action === "shuffle-nodes") {
      nodes.push(nodes.shift());
      state.selectedQuestionId = nodes[0].id;
      renderNodes();
      makeChallenge();
      addAudit("刷新 24 节点题集");
    }
    if (action === "save-question-answers") saveQuestionAnswers();
    if (action === "close-answer-editor") closeAnswerEditor();
    if (action === "save-story-editor") saveStoryEditor();
    if (action === "close-story-editor") closeStoryEditor();
    if (action === "close-resource-editor") closeResourceEditor();
    if (action === "use-question-challenge") {
      closeAnswerEditor();
      activateView("host-challenge");
      makeChallenge();
    }
    if (action === "add-resource") {
      const next = resources.length + 1;
      const resource = {
        id: `resource-${next}`,
        group: "normal",
        name: `本地保护对象 ${next}`,
        kind: "secret",
        action: "requestSignature",
        strength: "普通",
        description: "新建的本地保护对象，请补充用途和授权边界。",
      };
      resources.push(resource);
      state.selectedResourceId = resource.id;
      state.selectedResourceGroup = resource.group;
      renderResourceGroups();
      openResourceEditor();
      addAudit("添加保护对象");
    }
    if (action === "save-resource") saveResource();
    if (action === "new-challenge") makeChallenge();
    if (action === "demo-answer") {
      const correct = getSelectedQuestion().answers.find((answer) => answer.correct);
      state.selectedAnswerId = correct?.id ?? null;
      renderChallenge();
      addAudit("填入演示答案");
    }
    if (action === "verify-challenge") verifyChallenge();
    if (action === "issue-session") issueSession();
    if (action === "export-bundle") {
      exportBundle();
      addAudit("生成导出包摘要");
    }
    if (action === "clear-log") {
      state.audit = [];
      renderAudit();
    }
  });
}

bindNavigation();
bindActions();
renderNodes();
renderResourceGroups();
makeChallenge();
issueSession();
exportBundle();
activateView(location.hash.slice(1) || "draft", false);
addAudit("StoryLock 工作台已启动");
