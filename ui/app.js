const chinese = navigator.language.toLowerCase().startsWith("zh");
const strings = {
  title: ["Antigravity 账号", "Antigravity accounts"],
  intro: ["点击已保存的账号，在原生 App 中打开。", "Click a saved account to open Antigravity."],
  register: ["登记当前账号", "Save current account"],
  restore: ["恢复切换前会话", "Restore previous session"],
  forget: ["清除恢复记录", "Clear recovery record"],
  reload: ["重试", "Retry"],
  save: ["保存", "Save"],
  cancel: ["取消", "Cancel"],
  yes: ["确认", "Confirm"],
  remove: ["移除", "Remove"],
  alias: ["账号名称", "Account label"],
  activeBadge: ["当前", "Active"],
  loading: ["正在读取账号…", "Loading accounts…"],
  busy: ["正在处理…", "Working…"],
  ready: ["选择一个账号。", "Choose an account."],
  empty: ["还没有保存的账号。", "No accounts saved."],
  unregisteredStatus: [
    "检测到当前已登录未保存账号，可直接点击「登记当前账号」。",
    "An unsaved account is signed in. Click “Save current account” to register it."
  ],
  guidance: [
    "首次在官方 Antigravity 登录后，在此登记当前账号（最多四个）。客户端运行时切换会先请求确认并向可见窗口发送关闭信号；若窗口未关闭则安全中止，仅在窗口退入后台后收尾驻留进程并保留合法启动参数重开 App。",
    "Sign in through official Antigravity first, then save up to four accounts here. When Antigravity is running, Lite asks for confirmation and sends a close request to visible windows; if any window stays open, switching aborts safely, and tray-resident processes are cleaned up only after windows close before reopening with sanitized launch flags."
  ],
  opened: [
    "已打开 Antigravity。请在原生 App 核对账号。若出现登录页，请用官方流程重新登录。",
    "Antigravity opened. Check the account in the App. If sign-in is requested, use the official sign-in flow."
  ],
  restored: [
    "已恢复切换前的本地状态。请打开原生 App 核对账号。",
    "Previous local state restored. Open Antigravity to check the account."
  ],
  saved: ["已保存当前账号。", "Current account saved."],
  removed: ["已移除该保存项。", "Saved account removed."],
  cleared: ["已清除恢复记录。", "Recovery record cleared."],
  removePrompt: ["移除这个已保存的账号？", "Remove this saved account?"],
  forgetPrompt: ["清除本工具保存的恢复记录？", "Clear this app’s recovery record?"],
  switchRunningPrompt: [
    "Antigravity 正在运行。请确认没有正在执行的 Agent 且已保存工作，继续切换并重启客户端？",
    "Antigravity is running. Confirm no agents are active and your work is saved before switching and restarting?"
  ],
  reopenSameRunningPrompt: [
    "此账号已是当前活动账号，且 Antigravity 正在运行。仍要重启客户端吗？",
    "This account is already active and Antigravity is running. Restart the app anyway?"
  ],
  runningStatus: ["Antigravity 或共享认证进程正在运行。", "Antigravity or a shared sign-in process is running."],
  app_missing: ["未找到受支持的 Antigravity 安装。", "An Antigravity installation was not found."],
  account_missing: ["此账号已不存在。", "This saved account no longer exists."],
  client_running: [
    "仍有无法安全关闭的 App 或独立共享认证进程（如 agy CLI / IDE）。请完成任务并正常退出后再试。",
    "An app or independent shared sign-in process (such as agy CLI or IDE) is still running. Finish its work, quit, and try again."
  ],
  close_failed: [
    "未能向 Antigravity 发送关闭请求，账号未修改。请保存工作并手动退出 App 后重试。",
    "Could not request Antigravity to close. The account was left intact. Save your work, quit the app, and retry."
  ],
  close_timeout: [
    "窗口未在限定时间内关闭（可能存在未保存更改提示），切换已安全中止且账号未修改。",
    "A window remained open (check for unsaved prompts in Antigravity). Switching stopped safely with the account intact."
  ],
  client_state_unknown: ["无法核对 Antigravity 进程状态。已停止切换。", "Could not verify the Antigravity processes. Switching stopped."],
  client_state_changed: ["Antigravity 进程状态已改变。请重新选择账号。", "The Antigravity processes changed. Select the account again."],
  official_login_required: [
    "没有可保存的授权。请先在官方 Antigravity 登录。",
    "No saved authorization is available. Sign in through Antigravity first."
  ],
  invalid_label: ["请输入最长 48 字的账号名称。", "Enter an account label up to 48 characters."],
  slots_full: ["最多只能保存四个账号。", "Up to four accounts can be saved."],
  already_registered: ["已保存此会话，或账号名称已使用。", "This session or account label is already saved."],
  active_changed: [
    "原生账号状态已被外部修改。已保留当前状态，请核对 App 中的账号。",
    "The active session changed externally. It was left intact. Check the account in Antigravity."
  ],
  launch_failed: ["未能打开 Antigravity。切换前的状态已自动恢复。", "Antigravity could not open. The previous state was restored."],
  open_failed_without_switch: ["未能打开 Antigravity，原有授权状态仍在。", "Antigravity could not open. The existing authorization is intact."],
  switch_unverified: ["无法确认凭据写入。切换前的状态已自动恢复。", "The switch could not be verified. The previous state was restored."],
  restore_unverified: ["无法确认恢复结果。已停止操作。", "Recovery could not be verified."],
  recovery_required: ["上次切换尚未完成。退出原生 App 后，请先恢复会话。", "The previous switch is incomplete. Quit Antigravity and restore the previous session."],
  restore_before_discard: ["请先恢复切换前的会话。", "Restore the previous session first."],
  no_recovery: ["当前没有恢复记录。", "There is no recovery record."],
  journal_invalid: ["无法读取恢复记录。切换已停止。", "The recovery record could not be read. Switching stopped."],
  switch_in_progress: ["另一个账号操作尚未完成。", "An account operation is already in progress."],
  vault_unavailable: ["无法访问当前用户的 Windows 凭据管理器。", "Windows Credential Manager is unavailable."],
  data_folder_unavailable: ["无法访问本工具的本地目录。", "The app’s local folder is unavailable."],
  lock_unavailable: ["无法锁定切换操作。", "Could not lock the account operation."],
  operation_failed: ["操作没有完成。请重试。", "The operation did not complete. Try again."],
};

const t = key => (strings[key] ?? strings.operation_failed)[chinese ? 0 : 1];
const byId = id => document.getElementById(id);
const api = window.__TAURI__?.core;
let current;
let working = false;
let registration = false;

document.documentElement.lang = chinese ? "zh-CN" : "en";
for (const [id, key] of Object.entries({
  title: "title", intro: "intro", register: "register", restore: "restore", forget: "forget",
  reload: "reload", save: "save", "cancel-registration": "cancel", "label-title": "alias",
  guidance: "guidance", no: "cancel", yes: "yes",
})) {
  byId(id).textContent = t(key);
}
byId("accounts").setAttribute("aria-label", t("title"));
byId("label").placeholder = t("alias");

function notice(key, isError = false) {
  byId("notice").textContent = t(key);
  byId("notice").classList.toggle("error", isError);
}

function button(label, cssClass, action) {
  const element = document.createElement("button");
  element.type = "button";
  element.textContent = label;
  element.className = cssClass;
  element.addEventListener("click", action);
  return element;
}

async function requestSwitch(slot) {
  if (working) return;
  try {
    await load();
    render();
  } catch (error) {
    notice(typeof error === "string" ? error : "operation_failed", true);
    return;
  }
  if (!current?.appInstalled) {
    notice("app_missing", true);
    return;
  }
  if (current.recovery === "pending") {
    notice("recovery_required", true);
    return;
  }
  if (current.clientRunning) {
    const promptKey = current.activeSlot === slot ? "reopenSameRunningPrompt" : "switchRunningPrompt";
    if (!(await confirmAction(promptKey))) {
      showStatus();
      return;
    }
  }
  await operate("switch_account", { slot }, "opened");
}

function render() {
  const accounts = current?.accounts ?? [];
  const needsRecovery = current?.recovery === "pending";
  const list = byId("accounts");
  list.replaceChildren();
  for (const { slot, label } of accounts) {
    const isActive = current?.activeSlot === slot;
    const row = document.createElement("li");
    const open = document.createElement("button");
    open.type = "button";
    open.className = isActive ? "account is-active" : "account";
    if (isActive) open.setAttribute("aria-current", "true");

    const nameSpan = document.createElement("span");
    nameSpan.className = "account-label";
    nameSpan.textContent = label;
    open.append(nameSpan);

    if (isActive) {
      const badge = document.createElement("span");
      badge.className = "badge";
      badge.textContent = t("activeBadge");
      open.append(badge);
    }

    open.addEventListener("click", () => void requestSwitch(slot));

    const remove = button(t("remove"), "remove", async () => {
      if (await confirmAction("removePrompt")) await operate("remove_account", { slot }, "removed");
    });
    remove.setAttribute("aria-label", t("remove") + " " + label);
    open.disabled = working || needsRecovery || !current.appInstalled;
    remove.disabled = working || needsRecovery;
    row.append(open, remove);
    list.append(row);
  }
  byId("count").hidden = !current;
  byId("count").textContent = accounts.length + " / 4";
  byId("register").disabled = working || needsRecovery || !current?.appInstalled || accounts.length >= 4;
  byId("register").hidden = registration;
  byId("registration").hidden = !registration;
  byId("save").disabled = working;
  byId("label").disabled = working;
  byId("cancel-registration").disabled = working;
  for (const id of ["restore", "forget"]) {
    byId(id).hidden = !current?.recovery;
    byId(id).disabled = working;
  }
  byId("reload").hidden = Boolean(current);
  byId("reload").disabled = working;
}

function confirmAction(key) {
  const dialog = byId("confirm");
  byId("confirm-text").textContent = t(key);
  dialog.returnValue = "cancel";
  dialog.showModal();
  return new Promise(resolve => {
    dialog.addEventListener("close", () => resolve(dialog.returnValue === "confirm"), { once: true });
  });
}

async function load() {
  current = await api.invoke("overview");
}

function showStatus() {
  if (!current.appInstalled) notice("app_missing", true);
  else if (current.recovery === "pending") notice("recovery_required", true);
  else if (current.unregisteredActive && current.accounts.length < 4) notice("unregisteredStatus");
  else if (current.clientRunning) notice("runningStatus");
  else notice(current.accounts.length ? "ready" : "empty");
}

async function operate(command, arguments_, success) {
  if (working) return;
  working = true;
  render();
  notice("busy");
  try {
    await api.invoke(command, arguments_);
    if (command === "register_account") {
      registration = false;
      byId("label").value = "";
    }
    await load();
    notice(success);
  } catch (error) {
    try { await load(); } catch { current = undefined; }
    notice(typeof error === "string" ? error : "operation_failed", true);
  } finally {
    working = false;
    render();
  }
}

async function initialize() {
  working = true;
  render();
  notice("loading");
  try {
    await load();
    showStatus();
  } catch (error) {
    notice(typeof error === "string" ? error : "operation_failed", true);
  } finally {
    working = false;
    render();
  }
}

byId("register").addEventListener("click", () => {
  registration = true;
  render();
  byId("label").focus();
});
byId("cancel-registration").addEventListener("click", () => {
  registration = false;
  byId("label").value = "";
  render();
  byId("register").focus();
});
byId("registration").addEventListener("submit", event => {
  event.preventDefault();
  void operate("register_account", { label: byId("label").value }, "saved");
});
byId("restore").addEventListener("click", () => operate("restore_session", {}, "restored"));
byId("forget").addEventListener("click", async () => {
  if (await confirmAction("forgetPrompt")) await operate("discard_recovery", {}, "cleared");
});
byId("reload").addEventListener("click", initialize);
void initialize();
