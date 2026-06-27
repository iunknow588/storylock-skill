pub(crate) fn windows_host_management_ui_html() -> &'static str {
    r##"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Yian Windows Host</title>
    <style>
      :root { color-scheme: light; --bg:#f4f7f8; --surface:#fff; --line:#d5e0e5; --text:#12202b; --muted:#5c6d79; --accent:#0d6d77; }
      * { box-sizing: border-box; }
      body { margin: 0; background: var(--bg); color: var(--text); font-family: "Segoe UI", "Microsoft YaHei", sans-serif; }
      main { width: min(1180px, calc(100vw - 32px)); margin: 0 auto; padding: 28px 0 44px; }
      header { display: flex; align-items: end; justify-content: space-between; gap: 16px; margin-bottom: 20px; }
      h1, h2, p { margin: 0; }
      h1 { font-size: 28px; line-height: 1.2; }
      .muted { color: var(--muted); line-height: 1.65; }
      .grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px; }
      .wide { grid-column: span 2; }
      section { min-width: 0; padding: 18px; border: 1px solid var(--line); border-radius: 8px; background: var(--surface); }
      h2 { margin-bottom: 12px; font-size: 18px; }
      dl { display: grid; gap: 10px; margin: 0; }
      dt { color: var(--muted); font-size: 12px; }
      dd { margin: 3px 0 0; overflow-wrap: anywhere; }
      .pill { display: inline-flex; padding: 3px 9px; border: 1px solid var(--line); border-radius: 999px; color: var(--accent); font-size: 12px; }
      .ok { color: #0a6a38; }
      .warn { color: #9b5a00; }
      pre { margin: 0; max-height: 260px; overflow: auto; padding: 12px; border-radius: 8px; background: #101820; color: #d9e6ee; white-space: pre-wrap; overflow-wrap: anywhere; }
      button, a { min-height: 38px; display: inline-flex; align-items: center; justify-content: center; padding: 0 12px; border: 1px solid var(--line); border-radius: 8px; background: #fff; color: var(--text); text-decoration: none; cursor: pointer; }
      .actions { display: flex; flex-wrap: wrap; gap: 10px; }
      @media (max-width: 900px) { header, .grid { grid-template-columns: 1fr; display: grid; } .wide { grid-column: auto; } }
    </style>
  </head>
  <body>
    <main>
      <header>
        <div>
          <p class="muted">Yian Host audit and authorization surface</p>
          <h1>Yian Windows Host Management</h1>
        </div>
        <div class="actions">
          <button type="button" id="refresh">Refresh</button>
          <a href="/health">Health JSON</a>
          <a href="/diagnostics">Diagnostics</a>
        </div>
      </header>
      <div class="grid">
        <section><h2>Host</h2><dl id="host"></dl></section>
        <section><h2>Relay</h2><dl id="relay"></dl></section>
        <section><h2>Authorization Modes</h2><dl id="authorization-modes"></dl></section>
        <section><h2>Managed Objects</h2><dl id="managed-objects"></dl></section>
        <section><h2>Agents</h2><dl id="agents"></dl></section>
        <section><h2>Remote Interfaces</h2><dl id="remote-interfaces"></dl></section>
        <section><h2>Error Calls</h2><dl id="error-calls"></dl></section>
        <section><h2>Story Template Queue</h2><dl id="story-template"></dl></section>
        <section class="wide"><h2>Last Confirmation</h2><dl id="last-confirmation"></dl></section>
        <section><h2>Last Execution</h2><dl id="last-execution"></dl></section>
        <section><h2>Boundaries</h2><dl id="boundaries"></dl></section>
        <section class="wide"><h2>Raw Redacted Status JSON</h2><pre id="raw">loading...</pre></section>
      </div>
    </main>
    <script>
      const fields = (target, rows) => {
        document.querySelector(target).innerHTML = rows.map(([k, v]) => `<div><dt>${k}</dt><dd>${v ?? "not configured"}</dd></div>`).join("");
      };
      const emptyRows = (label) => [[label, "No calls recorded yet"]];
      async function loadStatus() {
        const response = await fetch("/ui/status", { headers: { accept: "application/json" } });
        const payload = await response.json();
        const result = payload.result || {};
        const host = result.host || {};
        const relay = result.relay || {};
        const remote = result.remote || {};
        const management = result.managementStats || {};
        const templateGenerator = result.storyTemplateGenerator || {};
        const confirmation = result.lastConfirmation || {};
        const last = result.lastExecution || {};
        const boundaries = result.boundaries || {};
        fields("#host", [
          ["Product", `${host.product || ""} ${host.version || ""}`],
          ["Status", `<span class="pill">${host.status || "unknown"}</span>`],
          ["Identity", host.identityId],
          ["Device", host.deviceId],
          ["Local API", host.executeUrl],
          ["Storage Visibility", host.storage?.visibility || "host_internal_only"],
        ]);
        fields("#relay", [
          ["Remote", remote.enabled ? `enabled: ${remote.gatewayUrl || ""}` : "local only"],
          ["Status", `<span class="${relay.status === "online" ? "ok" : "warn"}">${relay.status || "unknown"}</span>`],
          ["Last Poll", relay.lastPollAt],
          ["Last Error", relay.lastError || "none"],
        ]);
        fields("#authorization-modes", (management.authorizationModes || []).map(mode => [
          mode.channel,
          `${mode.requiredCells}/${mode.gridSize} grid cells, ${mode.requiredStrength}, ${mode.remoteAllowed ? "remote allowed" : "local only"}`
        ]));
        fields("#managed-objects", (management.objects || []).length ? management.objects.map(item => [
          item.objectRef,
          `${item.calls} calls, ${item.successes} ok, ${item.failures} errors, last ${item.lastSeenAt || "never"}`
        ]) : emptyRows("Managed objects"));
        fields("#agents", (management.agents || []).length ? management.agents.map(item => [item.name, `${item.calls} calls`]) : emptyRows("Agents"));
        fields("#remote-interfaces", (management.remoteInterfaces || []).length ? management.remoteInterfaces.map(item => [item.name, `${item.calls} calls`]) : emptyRows("Remote interfaces"));
        fields("#error-calls", (management.errors || []).length ? management.errors.map(item => [item.name, `${item.calls} calls`]) : [["No errors", "0 calls"]]);
        fields("#story-template", [
          ["Mode", templateGenerator.mode || "local_template_fallback"],
          ["LLM Key", templateGenerator.llmKey || "missing"],
          ["Candidate Count", templateGenerator.candidateCount ?? 0],
          ["Pull Rule", "StoryLock must pull; Host never invokes StoryLock"],
        ]);
        fields("#last-confirmation", [
          ["Request", confirmation.requestId || "none"],
          ["Status", confirmation.status || "none"],
          ["Capability", confirmation.capability || "none"],
          ["Object", confirmation.objectRef || "none"],
          ["Requester", confirmation.requester || "none"],
          ["Origin", confirmation.origin || "none"],
          ["Strength", confirmation.requiredStrength || "none"],
          ["Expiry", confirmation.expiry || "none"],
          ["Risk", confirmation.risk || "none"],
        ]);
        fields("#last-execution", [
          ["Request", last.requestId || "none"],
          ["Status", last.status || "none"],
          ["Capability", last.capability || "none"],
          ["Object", last.objectRef || "none"],
          ["Authorization", last.authorizationId || "none"],
          ["Strength", last.requiredStrength || "none"],
          ["Redaction", last.redactionLevel || "audit_meta_only"],
        ]);
        fields("#boundaries", [
          ["Remote Capabilities", (boundaries.remoteCapabilities || []).join(", ")],
          ["Hidden From UI", (boundaries.hiddenFromUi || []).join(", ")],
          ["Local Call Chain", (boundaries.localCoreCallChain || []).join(" -> ")],
        ]);
        document.querySelector("#raw").textContent = JSON.stringify(payload, null, 2);
      }
      document.querySelector("#refresh").addEventListener("click", loadStatus);
      loadStatus();
      setInterval(loadStatus, 5000);
    </script>
  </body>
</html>"##
}

#[allow(dead_code)]
pub(crate) fn windows_host_ui_html() -> &'static str {
    r##"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Yian Windows Host</title>
    <style>
      :root { color-scheme: light; --bg:#f4f7f8; --surface:#fff; --line:#d5e0e5; --text:#12202b; --muted:#5c6d79; --accent:#0d6d77; }
      * { box-sizing: border-box; }
      body { margin: 0; background: var(--bg); color: var(--text); font-family: "Segoe UI", "Microsoft YaHei", sans-serif; }
      main { width: min(1120px, calc(100vw - 32px)); margin: 0 auto; padding: 28px 0 44px; }
      header { display: flex; align-items: end; justify-content: space-between; gap: 16px; margin-bottom: 20px; }
      h1, h2, p { margin: 0; }
      h1 { font-size: 28px; line-height: 1.2; }
      .muted { color: var(--muted); line-height: 1.65; }
      .grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px; }
      .wide { grid-column: span 2; }
      section { min-width: 0; padding: 18px; border: 1px solid var(--line); border-radius: 8px; background: var(--surface); }
      h2 { margin-bottom: 12px; font-size: 18px; }
      dl { display: grid; gap: 10px; margin: 0; }
      dt { color: var(--muted); font-size: 12px; }
      dd { margin: 3px 0 0; overflow-wrap: anywhere; }
      .pill { display: inline-flex; padding: 3px 9px; border: 1px solid var(--line); border-radius: 999px; color: var(--accent); font-size: 12px; }
      .ok { color: #0a6a38; }
      .warn { color: #9b5a00; }
      pre { margin: 0; max-height: 260px; overflow: auto; padding: 12px; border-radius: 8px; background: #101820; color: #d9e6ee; white-space: pre-wrap; overflow-wrap: anywhere; }
      button, a { min-height: 38px; display: inline-flex; align-items: center; justify-content: center; padding: 0 12px; border: 1px solid var(--line); border-radius: 8px; background: #fff; color: var(--text); text-decoration: none; cursor: pointer; }
      .actions { display: flex; flex-wrap: wrap; gap: 10px; }
      @media (max-width: 840px) { header, .grid { grid-template-columns: 1fr; display: grid; } .wide { grid-column: auto; } }
    </style>
  </head>
  <body>
    <main>
      <header>
        <div>
          <p class="muted">StoryLock Local Core</p>
          <h1>Yian Windows Host 鏈湴绠＄悊椤?/h1>
        </div>
        <div class="actions">
          <button type="button" id="refresh">鍒锋柊</button>
          <a href="/health">Health JSON</a>
          <a href="/diagnostics">璇婃柇淇℃伅</a>
        </div>
      </header>
      <div class="grid">
        <section>
          <h2>瀹夸富鐘舵€?/h2>
          <dl id="host"></dl>
        </section>
        <section>
          <h2>Relay</h2>
          <dl id="relay"></dl>
        </section>
        <section>
          <h2>棰樺簱</h2>
          <dl id="question-bank"></dl>
        </section>
        <section class="wide">
          <h2>鏈€杩戠‘璁よ姹?/h2>
          <dl id="last-confirmation"></dl>
        </section>
        <section>
          <h2>鏈€杩戞墽琛屾憳瑕?/h2>
          <dl id="last-execution"></dl>
        </section>
        <section>
          <h2>鑳藉姏杈圭晫</h2>
          <dl id="boundaries"></dl>
        </section>
        <section class="wide">
          <h2>鍘熷鐘舵€?JSON</h2>
          <pre id="raw">loading...</pre>
        </section>
      </div>
    </main>
    <script>
      const fields = (target, rows) => {
        document.querySelector(target).innerHTML = rows.map(([k, v]) => `<div><dt>${k}</dt><dd>${v ?? "鏈厤缃?}</dd></div>`).join("");
      };
      async function loadStatus() {
        const response = await fetch("/ui/status", { headers: { accept: "application/json" } });
        const payload = await response.json();
        const result = payload.result || {};
        const host = result.host || {};
        const relay = result.relay || {};
        const remote = result.remote || {};
        const bank = result.questionBank || {};
        const confirmation = result.lastConfirmation || {};
        const last = result.lastExecution || {};
        const boundaries = result.boundaries || {};
        fields("#host", [
          ["浜у搧", `${host.product || ""} ${host.version || ""}`],
          ["鐘舵€?, `<span class="pill">${host.status || "unknown"}</span>`],
          ["identityId", host.identityId],
          ["deviceId", host.deviceId],
          ["鏈湴 API", host.executeUrl],
          ["Storage", host.storage?.visibility || "host_internal_only"],
        ]);
        fields("#relay", [
          ["Remote", remote.enabled ? `enabled: ${remote.gatewayUrl || ""}` : "local only"],
          ["鐘舵€?, `<span class="${relay.status === "online" ? "ok" : "warn"}">${relay.status || "unknown"}</span>`],
          ["鏈€杩戣疆璇?, relay.lastPollAt],
          ["鏈€杩戦敊璇?, relay.lastError || "鏃?],
        ]);
        fields("#question-bank", [
          ["鐗堟湰", bank.questionSetVersion],
          ["瑙勮寖鍖?, bank.normalizationVersion],
          ["棰樼洰鏁伴噺", bank.questionCount],
          ["Visibility", bank.visibility || "host_internal_only"],
        ]);
        fields("#last-confirmation", [
          ["璇锋眰", confirmation.requestId || "鏆傛棤"],
          ["鐘舵€?, confirmation.status || "鏆傛棤"],
          ["鑳藉姏", confirmation.capability || "鏆傛棤"],
          ["瀵硅薄", confirmation.objectRef || "鏆傛棤"],
          ["璇锋眰鏂?, confirmation.requester || "鏆傛棤"],
          ["鏉ユ簮", confirmation.origin || "鏆傛棤"],
          ["寮哄害", confirmation.requiredStrength || "鏆傛棤"],
          ["杩囨湡", confirmation.expiry || "鏆傛棤"],
          ["椋庨櫓", confirmation.risk || "鏆傛棤"],
        ]);
        fields("#last-execution", [
          ["璇锋眰", last.requestId || "鏆傛棤"],
          ["鐘舵€?, last.status || "鏆傛棤"],
          ["鑳藉姏", last.capability || "鏆傛棤"],
          ["瀵硅薄", last.objectRef || "鏆傛棤"],
          ["鎺堟潈", last.authorizationId || "鏆傛棤"],
          ["寮哄害", last.requiredStrength || "鏆傛棤"],
          ["鑴辨晱绛夌骇", last.redactionLevel || "audit_meta_only"],
        ]);
        fields("#boundaries", [
          ["杩滅▼鑳藉姏", (boundaries.remoteCapabilities || []).join(", ")],
          ["UI 闅愯棌瀛楁", (boundaries.hiddenFromUi || []).join(", ")],
          ["鏈湴璋冪敤閾?, (boundaries.localCoreCallChain || []).join(" -> ")],
        ]);
        document.querySelector("#raw").textContent = JSON.stringify(payload, null, 2);
      }
      document.querySelector("#refresh").addEventListener("click", loadStatus);
      loadStatus();
      setInterval(loadStatus, 5000);
    </script>
  </body>
</html>"##
}
