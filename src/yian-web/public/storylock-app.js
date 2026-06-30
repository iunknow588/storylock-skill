const storyTemplates = [
  {
    id: "shouzhudaitu-zh",
    title: "守株待兔（示例草稿）",
    summary: "农夫偶然捡到撞死在树桩上的兔子，从此放弃耕作，天天守在树桩旁等待下一次侥幸。",
    plot: "宋国有个农夫，正在田里劳作。忽然一只兔子慌慌张张地奔跑，撞上田边的树桩死了。农夫捡到兔子后，觉得不必再辛苦耕作，只要守着树桩就能再次得到兔子。于是他把锄头丢在一边，天天坐在树桩旁等待。日子一天天过去，兔子没有再撞上树桩，田地却因为无人耕作而荒芜。这个故事提醒人们，偶然得到的收获不能当作长期可靠的方法，更不能因为一次侥幸就放弃真正需要持续努力的事情。",
    promptNote: "示例原则：先用熟悉故事演示格式，再继续手工改写成更私人、更奇怪、更难猜的版本。不要直接依赖 AI 润色成品。",
    questions: [
      ["时代", "故事发生在什么时代背景？", "宋国"],
      ["地点", "农夫最初在哪里劳作？", "田里"],
      ["人物", "故事里的主要人物是谁？", "农夫"],
      ["物件", "兔子撞到的是什么？", "树桩"],
      ["事件", "农夫遇到了什么意外收获？", "捡到撞死的兔子"],
      ["动作", "农夫后来放下了什么工具？", "锄头"],
      ["选择", "农夫放弃了什么日常事务？", "耕作"],
      ["结果", "田地最后变成了什么样？", "荒芜"],
      ["判断", "农夫为什么误以为还能再得兔子？", "把偶然当成常法"],
      ["位置", "树桩在什么地方？", "田边"],
      ["状态", "兔子奔跑时是什么样子？", "慌慌张张"],
      ["动作细节", "农夫后来天天坐在哪里？", "树桩旁"],
      ["时间推进", "什么在一天天过去？", "日子"],
      ["失败", "后来兔子有没有再撞上树桩？", "没有"],
      ["代价", "农夫等待的代价是什么？", "荒废农田"],
      ["教训", "这个故事提醒人们不要怎样？", "不要因侥幸放弃努力"],
      ["认知", "偶然得到的收获不能被当作什么？", "长期可靠的方法"],
      ["心理", "农夫捡到兔子后产生了什么心态？", "侥幸"],
      ["反差", "真正需要持续做的事是什么？", "辛苦耕作"],
      ["环境", "树桩附近属于什么场景？", "农田场景"],
      ["误区", "农夫把哪种事情误认为可重复？", "偶然好运"],
      ["节奏", "这个故事更像突发好运还是长期经营？", "突发好运"],
      ["结局用途", "这个故事最终常被用来说明什么？", "不能守着侥幸过日子"],
      ["边界", "这个示例故事最该继续怎么处理？", "手工改写成私人版本"],
    ],
  },
  {
    id: "caochong-weighs-elephant-zh",
    title: "曹冲称象（示例草稿）",
    summary: "曹冲利用船、水线和石头，把大象的重量转化成可以称量的石头重量。",
    plot: "有人送来一头大象，大家都想知道它有多重，却没有足够大的秤。曹冲让人把大象牵到船上，在船身下沉到水面的地方刻下记号。随后把大象牵下船，再把石头一块块搬上船，直到船又沉到同一个记号处。最后称出这些石头的重量，就得到了大象的重量。这个故事展示了观察、替换和分步解决问题的智慧。",
    promptNote: "示例原则：真正安全的故事不要只停留在经典文本。请继续加入只属于你自己的细节、错位线索和不合常理的连接。",
    questions: [
      ["人物", "提出称象办法的是谁？", "曹冲"],
      ["对象", "需要称量的动物是什么？", "大象"],
      ["地点", "称象办法主要依赖什么交通工具？", "船"],
      ["标记", "曹冲先在什么位置做记号？", "船身水线处"],
      ["步骤", "大象下船后搬上船的是什么？", "石头"],
      ["原则", "石头需要搬到什么状态才可以停止？", "船沉到同一记号"],
      ["结果", "最后称出什么就等于大象重量？", "石头重量"],
      ["困难", "大家最初为什么不能直接称象？", "没有足够大的秤"],
      ["方法", "这个办法属于哪种解题思路？", "等量替换"],
      ["动作", "大象一开始被牵到哪里？", "船上"],
      ["比较", "记号前后需要保持什么一致？", "船体下沉位置一致"],
      ["观察", "曹冲观察的关键自然参照是什么？", "水面"],
      ["顺序", "是先搬石头还是先牵大象？", "先牵大象"],
      ["目的", "做记号的直接目的是什么？", "记录下沉深度"],
      ["拆解", "曹冲把难题拆成了哪类小问题？", "可逐步称量的石头问题"],
      ["智慧", "这个故事常被用来说明什么能力？", "巧妙解决问题的能力"],
      ["边界", "故事里有没有真的把大象放上秤？", "没有"],
      ["替身", "在这个办法里谁充当了大象的重量替身？", "石头"],
      ["验证", "如何确认石头数量已经足够？", "看船是否回到记号"],
      ["难点", "这个办法绕开的核心难点是什么？", "直接称量超大物体"],
      ["场景", "这个故事更像岸边实验还是宫廷宴会？", "岸边实验"],
      ["结构", "这个示例的叙事骨架是什么？", "观察记号再做替换"],
      ["提醒", "把经典故事变成个人草稿时应加入什么？", "私人且反常的细节"],
      ["用途", "这个示例最终只应该被当作什么？", "示例起稿"],
    ],
  },
  {
    id: "kongrong-shares-pears-zh",
    title: "孔融让梨（中文示例草稿）",
    summary: "年幼的孔融主动选择较小的梨，把较大的梨让给哥哥们，用行动表现礼让。",
    plot: "孔融年纪还小的时候，家中分梨，盘里有大有小。轮到他挑选时，他没有伸手拿最大的那个，而是主动拿起一个较小的梨。大人问他为什么这样选，他回答说自己年纪小，应该拿小的，把大的让给哥哥们。这个故事常被用来讲礼让、克制和先替别人着想的习惯。",
    promptNote: "示例原则：模板只演示统一文件格式。正式使用时请把人物、物件、顺序和暗号全都改成你自己的奇特版本。",
    questions: [
      ["年龄", "故事发生时孔融处在什么阶段？", "年幼时"],
      ["地点", "分梨这件事发生在哪里？", "家中"],
      ["人物", "主动挑小梨的人是谁？", "孔融"],
      ["物件", "大家分的水果是什么？", "梨"],
      ["选择", "孔融选择了哪种梨？", "较小的梨"],
      ["让给谁", "较大的梨被让给了谁？", "哥哥们"],
      ["原因", "孔融为什么说自己该拿小梨？", "因为自己年纪小"],
      ["提问者", "是谁问孔融为什么这样选？", "大人"],
      ["品质", "这个故事通常体现什么品德？", "礼让"],
      ["动作", "轮到挑选时孔融没有去拿什么？", "最大的梨"],
      ["场景细节", "盘里的梨有什么差别？", "有大有小"],
      ["顺序", "是先问原因还是先选梨？", "先选梨"],
      ["态度", "孔融的选择更像争抢还是克制？", "克制"],
      ["对象关系", "孔融让梨的对象与他是什么关系？", "兄弟关系"],
      ["回答核心", "孔融回答里最核心的一句话是什么？", "我小，应拿小的"],
      ["教育意义", "这个故事常被用来教什么习惯？", "先替别人着想"],
      ["反差", "孔融明明可以选大梨却没有这样做，说明了什么？", "主动谦让"],
      ["结构", "这个故事的冲突点来自什么？", "可拿大梨但主动不拿"],
      ["判断", "这里强调的是年龄优势还是礼让意识？", "礼让意识"],
      ["结果", "孔融最终拿走了哪一个？", "小梨"],
      ["情境", "这是公开比赛还是家庭分水果？", "家庭分水果"],
      ["原则", "这个示例若用于真实系统最该增加什么？", "个人专属怪异细节"],
      ["安全", "为什么不能直接照抄这个模板？", "太常见太容易猜"],
      ["用途", "这第三个模板在系统中的定位是什么？", "中文示例模板"],
    ],
  },
];

const genericDistractors = [
  "河岸边的石阶", "旧木箱", "门口的铜铃", "夜半三更", "仓房角落", "并不是这样", "旁观者误记", "临时记号", "意外插曲",
  "多年以后", "清晨薄雾", "屋檐下方", "完全相反", "翻过一页", "错位顺序", "陌生人猜不到", "公开资料查不到", "只在本地保存",
  "蓝色纸片", "反常细节", "私人暗号", "奇怪转折", "不合常理的搭配", "手工改写", "错误线索", "旧抽屉", "窗边影子",
  "一段停顿", "隐蔽位置", "重复检查", "临时替身", "并非最大那个", "不再出现", "严格本地化", "不导出明文", "演示用起稿",
];

const resourceGroups = [
  { id: "normal", label: "普通授权对象", chip: "普通" },
  { id: "private", label: "私密对象", chip: "私密" },
  { id: "secret", label: "机密对象", chip: "机密" },
];

let resources = [
  { id: "blog-read", group: "normal", name: "博客草稿读取授权", kind: "document", action: "requestPasswordFill", strength: "普通", description: "允许本地确认后读取低风险草稿内容。" },
  { id: "mail-login", group: "normal", name: "邮箱登录密码", kind: "password", action: "requestPasswordFill", strength: "普通", description: "用于演示网页凭据填充。" },
  { id: "api-token", group: "private", name: "演示 API Token", kind: "token", action: "requestSignature", strength: "私密", description: "只返回签名结果，不暴露 token 本体。" },
  { id: "cert-local", group: "private", name: "本地证书凭据", kind: "certificate", action: "requestSignature", strength: "私密", description: "用于本机证书操作授权。" },
  { id: "wallet-main", group: "secret", name: "主钱包签名密钥", kind: "credential", action: "requestSignature", strength: "机密", description: "高风险签名对象，需要更严格确认。" },
  { id: "vault-root", group: "secret", name: "本地 Vault 根密钥", kind: "private_key", action: "requestSignature", strength: "机密", description: "只用于展示对象边界，绝不导出密钥本体。" },
];

const retentionLearningPolicy = {
  purpose: "防止用户长期不用后忘记问题答案。导出后由本地 Host 按周期强制复习。",
  requiredQuestionCount: 22,
  requiredQuestionCountMeaning: "每次保留学习固定回答 22 个问题，用来确认用户仍然记得自己的故事锁。",
  frequencyDesign: "先密后疏：先按天，再按周、按月、按年逐步降频。",
  phaseParameterMeaning: "duration 表示阶段持续多久，frequency 表示该阶段隔多久触发一次复习。",
  phases: [
    { phase: "initial", label: "初始期", duration: "3 天", frequency: "每 1 天" },
    { phase: "consolidation", label: "巩固期", duration: "4 天", frequency: "每 2 天" },
    { phase: "adaptation", label: "适应期", duration: "3 周", frequency: "每 1 周" },
    { phase: "stable", label: "稳定期", duration: "4 个月", frequency: "每 1 个月" },
    { phase: "long_term", label: "长期期", duration: "1 年", frequency: "每 1 年" },
  ],
};

const CONFIG_SCHEMA_VERSION = "storylock-local-core-config-v1";
const EXPORT_PACKAGE_KIND = "configuration_summary";

const state = {
  activeChallenge: [],
  selectedAnswerId: null,
  selectedQuestionId: null,
  selectedResourceId: resources[0].id,
  selectedResourceGroup: "normal",
  verified: false,
  audit: [],
  templateIndex: 0,
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

function uniqueDistractors(correctText, index) {
  const picked = [];
  for (let offset = 0; offset < genericDistractors.length && picked.length < 8; offset += 1) {
    const candidate = genericDistractors[(index + offset) % genericDistractors.length];
    if (candidate !== correctText && !picked.includes(candidate)) {
      picked.push(candidate);
    }
  }
  while (picked.length < 8) {
    picked.push(`干扰项 ${index + 1}-${picked.length + 1}`);
  }
  return picked;
}

function makeAnswers(correctText, questionIndex) {
  const wrong = uniqueDistractors(correctText, questionIndex);
  return [
    { id: "a1", text: correctText, correct: true },
    ...wrong.map((text, index) => ({ id: `a${index + 2}`, text, correct: false })),
  ];
}

function buildNodesFromTemplate(template) {
  return template.questions.map(([type, prompt, correctAnswer], index) => ({
    id: `q-${String(index + 1).padStart(2, "0")}`,
    type,
    prompt,
    correctAnswer,
    answers: makeAnswers(correctAnswer, index),
  }));
}

function getActiveTemplate() {
  return storyTemplates[state.templateIndex] ?? storyTemplates[0];
}

let nodes = [];

function applyTemplate(index, reason = "切换示例模板") {
  state.templateIndex = (index + storyTemplates.length) % storyTemplates.length;
  const template = getActiveTemplate();
  nodes = buildNodesFromTemplate(template);
  state.selectedQuestionId = nodes[0]?.id ?? null;
  $('[data-field="story-title"]').value = template.title;
  $('[data-field="story-summary"]').value = template.summary;
  $('[data-field="story-plot"]').value = template.plot;
  $('[data-region="story-status"]').textContent = `示例模板 ${state.templateIndex + 1}/3`;
  renderNodes();
  makeChallenge(false);
  exportBundle();
  addAudit(`${reason}：${template.title}`);
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
  if (!question) return;
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
        <span class="answer-state-icon">${answer.correct ? "对" : "错"}</span>
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
  const anyOpen = $$('[data-region="answer-modal"], [data-region="resource-modal"], [data-region="story-modal"]')
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
  makeChallenge(false);
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
  $$('[data-resource-nav-group]').forEach((item) => {
    item.classList.toggle("is-active", item.dataset.resourceNavGroup === state.selectedResourceGroup);
  });
}

function renderResourceEditor() {
  const item = getSelectedResource();
  $('[data-region="resource-editor-note"]').textContent = `${item.name} | ${item.strength}`;
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
      <strong>${answer.correct ? "候选正确答案" : "干扰答案"}</strong>
      <p>${escapeHtml(answer.text)}</p>
    </button>`;
  }).join("");
  $('[data-region="challenge-note"]').textContent = `当前问题：${question.prompt}。请选择 1 个答案。`;
}

function makeChallenge(logIt = true) {
  const question = getSelectedQuestion();
  state.activeChallenge = [...question.answers];
  state.selectedAnswerId = null;
  state.verified = false;
  $('[data-region="session-status"]').textContent = "未授权";
  $('[data-region="session-status"]').className = "chip warn";
  renderChallenge();
  if (logIt) {
    addAudit(`生成 ${question.id} 的九宫格答案挑战`);
  }
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

function makeStableDigest(value) {
  let hash = 2166136261;
  const input = JSON.stringify(value);
  for (let index = 0; index < input.length; index += 1) {
    hash ^= input.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return `fnv1a32:${(hash >>> 0).toString(16).padStart(8, "0")}`;
}

function summarizeQuestionsForExport() {
  return nodes.map((question) => ({
    questionId: question.id,
    type: question.type,
    prompt: question.prompt,
    answerCandidateCount: question.answers.length,
    correctAnswerCount: question.answers.filter((answer) => answer.correct).length,
    optionDigest: makeStableDigest(question.answers.map((answer) => ({
      id: answer.id,
      text: answer.text,
      correct: answer.correct,
    }))),
    status: "active",
  }));
}

function summarizeResourcesForExport() {
  return resources.map((resource) => ({
    resourceId: resource.id,
    group: resource.group,
    name: resource.name,
    kind: resource.kind,
    requiredCapability: resource.action,
    strength: resource.strength,
    description: resource.description,
    secretMaterialExported: false,
  }));
}

function exportBundle() {
  const template = getActiveTemplate();
  const storyTitle = $('[data-field="story-title"]').value.trim();
  const storySummary = $('[data-field="story-summary"]').value.trim();
  const storyPlot = $('[data-field="story-plot"]').value.trim();
  const questionSet = summarizeQuestionsForExport();
  const protectedResources = summarizeResourcesForExport();
  const bundle = {
    schemaVersion: CONFIG_SCHEMA_VERSION,
    product: "StoryLock Local Core",
    packageKind: EXPORT_PACKAGE_KIND,
    exportedAt: new Date().toISOString(),
    deploymentTarget: "Yian prototype web deployment",
    templateCatalog: {
      builtinTemplateCount: storyTemplates.length,
      activeTemplateId: template.id,
      note: "Only three example templates are built in. Rewrite by hand before real use.",
      securityPrompt: "Prefer private, strange, and less reasonable story details so outsiders cannot easily infer answers.",
    },
    localCoreBoundary: {
      core: "storylock_local_core",
      hostUiRuntimeSeparated: true,
      hostMayReadConfigPackage: true,
      hostMayExportStoryPlaintext: false,
      hostMayExportSecretMaterial: false,
    },
    story: {
      title: storyTitle,
      summary: storySummary,
      plotDigest: makeStableDigest(storyPlot),
      plaintextExported: false,
      plaintextRedaction: "story plot is kept in local StoryLock draft only",
      authorRewriteRequired: true,
    },
    questionSetVersion: "draft-web-prototype-v2",
    questionCount: questionSet.length,
    answerCandidatesPerQuestion: 9,
    questions: questionSet,
    resourceCount: protectedResources.length,
    resourceGroups: resourceGroups.map((group) => ({
      id: group.id,
      label: group.label,
      count: resources.filter((item) => item.group === group.id).length,
    })),
    protectedResources,
    containsStoryPlaintext: false,
    containsSecrets: false,
    capabilities: ["requestSignature", "requestPasswordFill"],
    retentionLearningPolicy,
    promptGuidance: [
      template.promptNote,
      "不要把模板直接当成最终成品。",
      "正式版本应加入私人记忆、错位细节和反常结构。",
    ],
    auditSummary: {
      latestEvents: state.audit.slice(0, 8),
      totalEvents: state.audit.length,
    },
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
  $$('[data-view-target]').forEach((item) => {
    const isResourceLevel = Boolean(item.dataset.resourceNavGroup);
    const isActiveView = item.dataset.viewTarget === normalizedViewId && !isResourceLevel;
    item.classList.toggle("is-active", isActiveView);
  });
  $$(".view").forEach((item) => item.classList.toggle("is-active", item.id === normalizedViewId));
  if (normalizedViewId === "resources") updateResourceNavState();
  if (updateHash) history.replaceState(null, "", `#${normalizedViewId}`);
}

function bindNavigation() {
  $$('[data-view-target]').forEach((button) => {
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
    const questionId = event.target.closest('[data-question-id]')?.dataset.questionId;
    const resourceGroup = event.target.closest('[data-resource-nav-group]')?.dataset.resourceNavGroup;
    const resourceId = event.target.closest('[data-resource-id]')?.dataset.resourceId;
    const challengeAnswer = event.target.closest('[data-challenge-answer]')?.dataset.challengeAnswer;
    const answerState = event.target.closest('[data-answer-state]')?.dataset.answerState;
    const action = event.target.closest('[data-action]')?.dataset.action;

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
      const button = event.target.closest('[data-answer-state]');
      button.classList.toggle("is-correct", checkbox.checked);
      button.classList.toggle("is-wrong", !checkbox.checked);
      button.setAttribute("aria-pressed", checkbox.checked ? "true" : "false");
      button.querySelector(".answer-state-icon").textContent = checkbox.checked ? "对" : "错";
      button.querySelector(".answer-state-text").textContent = checkbox.checked ? "正确" : "错误";
      return;
    }
    if (!action) return;

    if (action === "open-story-editor") openStoryEditor();
    if (action === "shuffle-nodes") {
      applyTemplate(state.templateIndex + 1, "切换示例模板");
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
      const group = resourceGroups.some((item) => item.id === state.selectedResourceGroup)
        ? state.selectedResourceGroup
        : "normal";
      const groupMeta = resourceGroups.find((item) => item.id === group) ?? resourceGroups[0];
      const resource = {
        id: `resource-${next}`,
        group,
        name: `本地保护对象 ${next}`,
        kind: "secret",
        action: "requestSignature",
        strength: groupMeta.chip,
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
renderResourceGroups();
applyTemplate(0, "载入默认示例模板");
issueSession();
activateView(location.hash.slice(1) || "draft", false);
addAudit("StoryLock 工作台已启动");
