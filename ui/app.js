const chinese = navigator.language.toLowerCase().startsWith("zh");
const strings = {
  title: ["Antigravity 账号", "Antigravity accounts"],
  intro: ["点击账号或按数字键 1–4 切换，关闭窗口自动驻留系统托盘。", "Click an account or press 1–4 to switch; closing hides to the system tray."],
  register: ["登记当前账号", "Save current account"],
  restore: ["恢复切换前会话", "Restore previous session"],
  forget: ["清除恢复记录", "Clear recovery record"],
  reload: ["重试", "Retry"],
  save: ["保存", "Save"],
  cancel: ["取消", "Cancel"],
  yes: ["确认", "Confirm"],
  remove: ["移除", "Remove"],
  updateCredential: ["更新凭据", "Update credential"],
  alias: ["账号名称", "Account label"],
  activeBadge: ["当前", "Active"],
  autostartLabel: ["开机自启", "Launch at startup"],
  hotkeyTitle: ["快速唤出", "Quick summon"],
  editHotkey: ["修改", "Change"],
  disableHotkey: ["禁用", "Disable"],
  hotkeyDisabled: ["已禁用", "Disabled"],
  recordingBadge: ["请按键…", "Press keys…"],
  recordingPrompt: [
    "请按下包含 Ctrl / Alt / Win 的组合键，按 Esc 取消。",
    "Press a shortcut with Ctrl, Alt, or Win (Esc to cancel)."
  ],
  autostartEnabled: ["已开启开机自启（开机静默驻留托盘）。", "Launch at startup enabled (starts silently in tray)."],
  autostartDisabled: ["已关闭开机自启。", "Launch at startup disabled."],
  autostartBlockedNotice: [
    "开机自启已在 Windows 系统启动项设置中被禁用，请在任务管理器或系统设置中开启。",
    "Startup is disabled by Windows Startup settings. Re-enable it in Task Manager or Settings."
  ],
  hotkeySaved: ["已更新快速唤出热键。", "Quick summon shortcut updated."],
  hotkeyCleared: ["已禁用快速唤出热键。", "Quick summon shortcut disabled."],
  hotkey_conflict: [
    "该热键已被其他程序占用，请点击「修改」更换组合键。",
    "That shortcut is already in use by another app. Click “Change” to pick another."
  ],
  hotkey_unavailable: ["无法向系统注册该热键，请更换其他组合键。", "Windows could not register that shortcut. Try another combination."],
  invalid_hotkey: [
    "无效热键：须包含 Ctrl / Alt / Win 修饰键且不能占用系统保留热键。",
    "Invalid shortcut: include Ctrl, Alt, or Win and avoid reserved system shortcuts."
  ],
  autostart_unavailable: ["无法写入当前用户的开机自启注册表项。", "Could not update the current user’s startup registry entry."],
  loading: ["正在读取账号…", "Loading accounts…"],
  busy: ["正在处理…", "Working…"],
  ready: ["选择一个账号。", "Choose an account."],
  empty: ["还没有保存的账号。", "No accounts saved."],
  unregisteredStatus: [
    "检测到当前已登录未保存账号，可直接点击「登记当前账号」。",
    "An unsaved account is signed in. Click “Save current account” to register it."
  ],
  unmatchedStatus: [
    "当前凭据未匹配已保存账号。如果是原账号，请核对后更新相应槽位。",
    "The current credential matches no saved account. If you recognize the account, verify it and update its slot."
  ],
  guidance: [
    "首次在官方 Antigravity 登录后，在此登记当前账号（最多四个）。热键可随时唤出或收起窗口（窗口内按 Esc 亦可收起回托盘，真正退出请使用托盘右键菜单）。客户端运行时切换会先请求确认并发送关闭信号；若窗口未关闭则安全中止。",
    "Sign in through official Antigravity first, then save up to four accounts here. Use the global shortcut to summon or hide this window (Esc also hides to tray; use the tray menu to quit). Running switches ask for confirmation first and abort safely if any window stays open."
  ],
  opened: [
    "已打开 Antigravity。请在原生 App 核对账号。若出现登录页，请用官方流程重新登录。",
    "Antigravity opened. Check the account in the App. If sign-in is requested, use the official sign-in flow."
  ],
  opened_unmatched: [
    "Antigravity 已打开，但本地凭据未能确认目标账号。请在原生 App 核对账号。",
    "Antigravity opened, but the local credential could not confirm the target account. Check the account in the App."
  ],
  restored: [
    "已恢复切换前的本地状态。请打开原生 App 核对账号。",
    "Previous local state restored. Open Antigravity to check the account."
  ],
  saved: ["已保存当前账号。", "Current account saved."],
  credentialUpdated: ["已更新槽位凭据。", "Saved credential updated."],
  removed: ["已移除该保存项。", "Saved account removed."],
  cleared: ["已清除恢复记录。", "Recovery record cleared."],
  removePrompt: ["移除这个已保存的账号？", "Remove this saved account?"],
  updateCredentialPrompt: [
    "请先在 Antigravity 确认当前登录的是「{account}」并完全退出。确认用当前凭据更新该槽位？",
    "Verify in Antigravity that you are signed in as “{account}”, then quit it completely. Update this slot with the current credential?"
  ],
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
  different_account: ["当前凭据与该槽位属于不同账号。", "The current credential and the slot belong to different accounts."],
  active_changed: [
    "原生账号状态已被外部修改。已保留当前状态，请核对 App 中的账号。",
    "The active session changed externally. It was left intact. Check the account in Antigravity."
  ],
  launch_failed: ["未能打开 Antigravity。切换前的状态已自动恢复。", "Antigravity could not open. The previous state was restored."],
  open_failed_without_switch: ["未能打开 Antigravity，原有授权状态仍在。", "Antigravity could not open. The existing authorization is intact."],
  switch_unverified: ["无法确认凭据写入。切换前的状态已自动恢复。", "The switch could not be verified. The previous state was restored."],
  restore_unverified: ["无法确认恢复结果。已停止操作。", "Recovery could not be verified."],
  recovery_required: ["上次切换尚未完成。退出原生 App 后，请先恢复会话。", "The previous switch is incomplete. Quit Antigravity and restore the previous session."],
  recovery_unmatched: [
    "当前凭据无法匹配上次的目标账号。请核对原生 App，完全退出后可选择清除恢复记录。",
    "The current credential could not be matched to the previous target. Check Antigravity, then quit it completely to clear the recovery record."
  ],
  restore_before_discard: ["请先恢复切换前的会话。", "Restore the previous session first."],
  clear_recovery_before_update: ["此槽位关联了恢复记录。请先处理该记录，再更新凭据。", "This slot has a recovery record. Resolve it before updating the credential."],
  no_recovery: ["当前没有恢复记录。", "There is no recovery record."],
  journal_invalid: ["无法读取恢复记录。切换已停止。", "The recovery record could not be read. Switching stopped."],
  switch_in_progress: ["另一个账号操作尚未完成。", "An account operation is already in progress."],
  vault_unavailable: ["无法访问当前用户的 Windows 凭据管理器。", "Windows Credential Manager is unavailable."],
  data_folder_unavailable: ["无法访问本工具的本地目录。", "The app’s local folder is unavailable."],
  lock_unavailable: ["无法锁定切换操作。", "Could not lock the account operation."],
  operation_failed: ["操作没有完成。请重试。", "The operation did not complete. Try again."],
};

const CODE_KEY_MAP = {
  Space: "Space", Tab: "Tab", Enter: "Enter", Backspace: "Backspace",
  Insert: "Insert", Delete: "Delete", Home: "Home", End: "End",
  PageUp: "PageUp", PageDown: "PageDown",
  ArrowUp: "Up", ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right",
  Backquote: "`", Minus: "-", Equal: "=", BracketLeft: "[", BracketRight: "]",
  Backslash: "\\", Semicolon: ";", Quote: "'", Comma: ",", Period: ".", Slash: "/",
};

const t = key => (strings[key] ?? strings.operation_failed)[chinese ? 0 : 1];
const byId = id => document.getElementById(id);
const api = window.__TAURI__?.core;
let current;
let working = false;
let registration = false;
let recording = false;
let recordingPreview = "";

document.documentElement.lang = chinese ? "zh-CN" : "en";
for (const [id, key] of Object.entries({
  title: "title", intro: "intro", register: "register", restore: "restore", forget: "forget",
  reload: "reload", save: "save", "cancel-registration": "cancel", "label-title": "alias",
  "autostart-label": "autostartLabel", "hotkey-title": "hotkeyTitle",
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

function mapEventMainKey(event) {
  const { code, key } = event;
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-2])$/.test(code)) return code;
  if (CODE_KEY_MAP[code]) return CODE_KEY_MAP[code];
  if (key && key.length === 1 && /[a-z0-9]/i.test(key)) return key.toUpperCase();
  return null;
}

function formatModifiers(event) {
  const parts = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Win");
  return parts;
}

async function requestSwitch(slot) {
  if (working || recording) return;
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
    notice(current.canRestore ? "recovery_required" : "recovery_unmatched", true);
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

    const left = document.createElement("span");
    left.className = "account-main";
    const keyHint = document.createElement("kbd");
    keyHint.className = "slot-key";
    keyHint.textContent = String(slot);
    const nameSpan = document.createElement("span");
    nameSpan.className = "account-label";
    nameSpan.textContent = label;
    left.append(keyHint, nameSpan);
    open.append(left);

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
    open.disabled = working || recording || needsRecovery || !current.appInstalled;
    remove.disabled = working || recording || needsRecovery;
    row.append(open);
    if (current.unregisteredActive && current.appInstalled && !needsRecovery) {
      const update = button(t("updateCredential"), "compact", async () => {
        if (await confirmAction("updateCredentialPrompt", label)) {
          await operate("update_saved_account", { slot }, "credentialUpdated");
        }
      });
      update.setAttribute("aria-label", t("updateCredential") + " " + label);
      update.disabled = working || recording;
      row.append(update);
    }
    row.append(remove);
    list.append(row);
  }
  byId("count").hidden = !current;
  byId("count").textContent = accounts.length + " / 4";
  byId("register").disabled = working || recording || needsRecovery || !current?.appInstalled || accounts.length >= 4;
  byId("register").hidden = registration;
  byId("registration").hidden = !registration;
  byId("save").disabled = working;
  byId("label").disabled = working;
  byId("cancel-registration").disabled = working;
  for (const id of ["restore", "forget"]) {
    byId(id).hidden = id === "restore" ? !current?.canRestore : !current?.recovery;
    byId(id).disabled = working || recording;
  }
  byId("reload").hidden = Boolean(current);
  byId("reload").disabled = working || recording;

  const autostartBox = byId("autostart");
  autostartBox.checked = Boolean(current?.autostart);
  autostartBox.disabled = working || recording || !current;

  const hotkeyDisplay = byId("hotkey-display");
  const hasHotkey = Boolean(current?.hotkey);
  const isConflict = current?.hotkeyStatus === "conflict";
  if (recording) {
    hotkeyDisplay.textContent = recordingPreview || t("recordingBadge");
  } else {
    hotkeyDisplay.textContent = hasHotkey ? current.hotkey : t("hotkeyDisabled");
  }
  hotkeyDisplay.classList.toggle("is-recording", recording);
  hotkeyDisplay.classList.toggle("is-conflict", !recording && isConflict);
  hotkeyDisplay.classList.toggle("is-muted", !recording && !hasHotkey);

  const editBtn = byId("hotkey-edit");
  const clearBtn = byId("hotkey-clear");
  editBtn.textContent = t("editHotkey");
  editBtn.hidden = recording;
  editBtn.disabled = working || !current;

  clearBtn.textContent = recording ? t("cancel") : t("disableHotkey");
  clearBtn.hidden = !recording && !hasHotkey;
  clearBtn.disabled = working || !current;
}

function confirmAction(key, accountLabel = "") {
  const dialog = byId("confirm");
  byId("confirm-text").textContent = t(key).replace("{account}", () => accountLabel);
  dialog.returnValue = "cancel";
  dialog.showModal();
  return new Promise(resolve => {
    dialog.addEventListener("close", () => resolve(dialog.returnValue === "confirm"), { once: true });
  });
}

async function load() {
  current = await api.invoke("overview");
}

function applyShellStatus(shell) {
  if (!current || !shell) return;
  current.autostart = shell.autostart;
  current.autostartBlocked = shell.autostartBlocked;
  current.hotkey = shell.hotkey;
  current.hotkeyStatus = shell.hotkeyStatus;
}

function showStatus() {
  if (!current) return;
  if (!current.appInstalled) notice("app_missing", true);
  else if (current.recovery === "pending") notice(current.canRestore ? "recovery_required" : "recovery_unmatched", true);
  else if (current.hotkeyStatus === "conflict") notice("hotkey_conflict", true);
  else if (current.autostart && current.autostartBlocked) notice("autostartBlockedNotice", true);
  else if (current.unregisteredActive) notice(current.accounts.length ? "unmatchedStatus" : "unregisteredStatus");
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
    if (command === "switch_account" && current.activeSlot !== arguments_.slot) {
      notice("opened_unmatched", true);
    } else {
      notice(success);
    }
  } catch (error) {
    try { await load(); } catch { current = undefined; }
    if (error === "recovery_required" && current?.recovery === "pending" && !current.canRestore) {
      notice("recovery_unmatched", true);
    } else {
      notice(typeof error === "string" ? error : "operation_failed", true);
    }
  } finally {
    working = false;
    render();
  }
}

async function startRecording() {
  if (working || recording || !current) return;
  recording = true;
  recordingPreview = "";
  render();
  notice("recordingPrompt");
  try {
    await api.invoke("set_hotkey_suspended", { suspended: true });
  } catch {
    // Best effort suspension while capturing keystrokes.
  }
}

async function cancelRecording(restoreNotice = true) {
  if (!recording) return;
  recording = false;
  recordingPreview = "";
  try {
    await api.invoke("set_hotkey_suspended", { suspended: false });
  } catch {
    // Ignore
  }
  render();
  if (restoreNotice) showStatus();
}

async function submitHotkey(hotkey, successKey) {
  working = true;
  recording = false;
  recordingPreview = "";
  render();
  try {
    const shell = await api.invoke("set_hotkey", { hotkey });
    applyShellStatus(shell);
    notice(successKey);
  } catch (error) {
    try { await load(); } catch { /* keep current */ }
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

byId("autostart").addEventListener("change", async event => {
  const enabled = event.target.checked;
  working = true;
  render();
  try {
    const shell = await api.invoke("set_autostart", { enabled });
    applyShellStatus(shell);
    if (shell.autostart && shell.autostartBlocked) {
      notice("autostartBlockedNotice", true);
    } else {
      notice(shell.autostart ? "autostartEnabled" : "autostartDisabled");
    }
  } catch (error) {
    try { await load(); } catch { /* ignore */ }
    notice(typeof error === "string" ? error : "operation_failed", true);
  } finally {
    working = false;
    render();
  }
});

byId("hotkey-edit").addEventListener("click", () => void startRecording());
byId("hotkey-clear").addEventListener("click", () => {
  if (recording) void cancelRecording(true);
  else void submitHotkey(null, "hotkeyCleared");
});

window.addEventListener("blur", () => {
  if (recording) void cancelRecording(true);
});

window.addEventListener("focus", async () => {
  if (working || registration || recording || byId("confirm").open) return;
  try {
    await load();
    render();
    showStatus();
  } catch {
    // Keep last rendered state if background refresh fails.
  }
});

document.addEventListener("keydown", event => {
  if (recording) {
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape" && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
      void cancelRecording(true);
      return;
    }
    const mods = formatModifiers(event);
    const mainKey = mapEventMainKey(event);
    if (!mainKey) {
      recordingPreview = mods.length ? mods.join("+") + "+…" : "";
      render();
      return;
    }
    void submitHotkey([...mods, mainKey].join("+"), "hotkeySaved");
    return;
  }

  if (working || registration || byId("confirm").open) return;
  const activeTag = document.activeElement?.tagName;
  if (activeTag === "INPUT" || activeTag === "TEXTAREA") return;

  if (event.key === "Escape" && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    event.preventDefault();
    void api.invoke("hide_window");
    return;
  }

  if (!event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey && /^[1-4]$/.test(event.key)) {
    const slot = Number(event.key);
    if (current?.accounts?.some(item => item.slot === slot)) {
      event.preventDefault();
      void requestSwitch(slot);
    }
  }
});

document.addEventListener("keyup", event => {
  if (!recording) return;
  const mods = formatModifiers(event);
  recordingPreview = mods.length ? mods.join("+") + "+…" : "";
  render();
});

void initialize();
