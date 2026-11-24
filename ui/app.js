const chinese = navigator.language.toLowerCase().startsWith("zh");
const strings = {
  title: ["AG Switcher", "AG Switcher"],
  accountsAria: ["Antigravity 账号", "Antigravity accounts"],
  modeList: ["列表", "List"],
  modeMatrix: ["矩阵", "Matrix"],
  themeSystem: ["自动", "Auto"],
  themeLight: ["浅色", "Light"],
  themeDark: ["深色", "Dark"],
  themeToggleAria: ["切换深浅色", "Toggle theme"],
  settingsAria: ["打开设置", "Open settings"],
  dragHint: ["按住此处拖动窗口", "Drag to move window"],
  minimizeAria: ["最小化", "Minimize"],
  closeAria: ["收起至托盘 (Esc)", "Hide to tray (Esc)"],
  settingsTitle: ["视图与驻留设置", "View & tray settings"],
  done: ["完成", "Done"],
  styleLabel: ["风格切换", "Layout style"],
  appearanceLabel: ["外观主题", "Appearance"],
  particlesLabel: ["星环跟随鼠标", "Cursor halo"],
  particlesHint: ["移出窗口时全背景均匀呼吸", "Uniform breathing when idle"],
  autostartLabel: ["开机静默自启", "Launch at startup"],
  hotkeyTitle: ["全局快速唤出", "Quick summon"],
  editHotkey: ["修改", "Change"],
  disableHotkey: ["禁用", "Disable"],
  hotkeyDisabled: ["已禁用", "Disabled"],
  recordingBadge: ["请按键…", "Press keys…"],
  recordingPrompt: [
    "请按下包含 Ctrl / Alt / Win 的组合键，按 Esc 取消。",
    "Press a shortcut with Ctrl, Alt, or Win (Esc to cancel)."
  ],
  footerHint: ["数字键 1–4 速切 · Esc 收回托盘", "Keys 1–4 to switch · Esc to hide"],
  registerShort: ["＋ 登记当前账号", "+ Save current"],
  restore: ["恢复切换前会话", "Restore previous session"],
  restoreShort: ["恢复上次会话", "Restore session"],
  forget: ["清除恢复记录", "Clear recovery"],
  forgetShort: ["清除记录", "Clear record"],
  reload: ["重试", "Retry"],
  save: ["保存", "Save"],
  cancel: ["取消", "Cancel"],
  yes: ["确认", "Confirm"],
  confirmTitle: ["请确认操作", "Confirm action"],
  remove: ["移除", "Remove"],
  updateCredential: ["更新", "Update"],
  alias: ["输入备注名称（最长 48 字）", "Account label (up to 48 chars)"],
  activeMatrix: ["● 当前激活会话", "● Active session"],
  idleMatrix: ["点击安全切入", "Click to switch"],
  activePill: ["当前 · #{slot}", "Active · #{slot}"],
  emptyTitle: ["空闲槽位 #{slot}", "Empty slot #{slot}"],
  emptySub: ["点击登记当前账号", "Click to save current account"],
  emptyUnregistered: ["检测到新会话 · 点击登记", "New session · Click to save"],
  pillRunning: ["AGY 运行中", "AGY running"],
  pillStandby: ["待命", "Standby"],
  pillUnregistered: ["新会话未存", "Unsaved sign-in"],
  pillRecovery: ["待恢复", "Recovery"],
  pillConflict: ["热键冲突", "Shortcut conflict"],
  autostartEnabled: ["已开启开机自启（开机静默驻留托盘）。", "Launch at startup enabled (starts silently in tray)."],
  autostartDisabled: ["已关闭开机自启。", "Launch at startup disabled."],
  autostartBlockedNotice: [
    "开机自启已在 Windows 系统启动项设置中被禁用，请在任务管理器或系统设置中开启。",
    "Startup is disabled by Windows Startup settings. Re-enable it in Task Manager or Settings."
  ],
  hotkeySaved: ["已更新快速唤出热键。", "Quick summon shortcut updated."],
  hotkeyCleared: ["已禁用快速唤出热键。", "Quick summon shortcut disabled."],
  hotkey_conflict: [
    "该热键已被其他程序占用，请在右上角设置或点击热键更换组合键。",
    "That shortcut is already in use by another app. Pick another combination."
  ],
  hotkey_unavailable: ["无法向系统注册该热键，请更换其他组合键。", "Windows could not register that shortcut. Try another combination."],
  invalid_hotkey: [
    "无效热键：须包含 Ctrl / Alt / Win 修饰键且不能占用系统保留热键。",
    "Invalid shortcut: include Ctrl, Alt, or Win and avoid reserved system shortcuts."
  ],
  invalid_preferences: ["界面偏好参数无效。", "Invalid interface preference."],
  autostart_unavailable: ["无法写入当前用户的开机自启注册表项。", "Could not update the current user’s startup registry entry."],
  loading: ["正在读取账号…", "Loading accounts…"],
  busy: ["正在处理…", "Working…"],
  unregisteredStatus: [
    "检测到当前已登录未保存账号，点击任意空闲槽位即可登记。",
    "An unsaved account is signed in. Click an empty slot to save it."
  ],
  unmatchedStatus: [
    "当前凭据未匹配已保存账号。如属原账号重登，核对后可点槽位「更新」；如属新账号，可点空槽登记。",
    "Current credential matches no saved slot. Update its slot if re-authenticated, or save to an empty slot."
  ],
  opened: [
    "已打开 Antigravity。请在原生 App 核对账号；若出现登录页，请用官方流程重新登录。",
    "Antigravity opened. Verify the account in the app; if prompted, use the official sign-in flow."
  ],
  opened_unmatched: [
    "Antigravity 已打开，但本地凭据未能确认目标账号。请在原生 App 核对账号。",
    "Antigravity opened, but the local credential could not confirm the target account. Check in the app."
  ],
  restored: [
    "已恢复切换前的本地状态。请打开原生 App 核对账号。",
    "Previous local state restored. Open Antigravity to verify the account."
  ],
  saved: ["已保存当前账号。", "Current account saved."],
  credentialUpdated: ["已更新槽位凭据。", "Saved credential updated."],
  removed: ["已移除该保存项。", "Saved account removed."],
  cleared: ["已清除恢复记录。", "Recovery record cleared."],
  removePrompt: ["移除槽位「{account}」的已保存凭据？", "Remove saved account “{account}”?"],
  updateCredentialPrompt: [
    "请先在 Antigravity 确认当前登录的是「{account}」并完全退出。确认用当前凭据更新该槽位？",
    "Verify in Antigravity that you are signed in as “{account}”, then quit it completely. Update this slot?"
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
const darkMedia = window.matchMedia("(prefers-color-scheme: dark)");

let current;
let working = false;
let registration = false;
let recording = false;
let recordingPreview = "";
let settingsOpen = false;
let transientNotice = null; // { key, tone: 'info' | 'warn' | 'error' }

const uiPrefs = {
  layout: "matrix",      // 'list' | 'matrix'
  theme: "system",       // 'system' | 'light' | 'dark'
  particlesFollow: true,
};

document.documentElement.lang = chinese ? "zh-CN" : "en";
for (const [id, key] of Object.entries({
  title: "title",
  "settings-title": "settingsTitle",
  "settings-close": "done",
  "style-label": "styleLabel",
  "mode-list": "modeList",
  "mode-matrix": "modeMatrix",
  "appearance-label": "appearanceLabel",
  "theme-system": "themeSystem",
  "theme-light": "themeLight",
  "theme-dark": "themeDark",
  "particles-label": "particlesLabel",
  "particles-hint": "particlesHint",
  "autostart-label": "autostartLabel",
  "hotkey-title": "hotkeyTitle",
  "footer-hint": "footerHint",
  register: "registerShort",
  restore: "restore",
  forget: "forget",
  "restore-footer": "restoreShort",
  "forget-footer": "forgetShort",
  reload: "reload",
  save: "save",
  "cancel-registration": "cancel",
  "confirm-title": "confirmTitle",
  no: "cancel",
  yes: "yes",
})) {
  byId(id).textContent = t(key);
}
byId("accounts").setAttribute("aria-label", t("accountsAria"));
byId("theme-toggle").setAttribute("aria-label", t("themeToggleAria"));
byId("theme-toggle").title = t("themeToggleAria");
byId("settings-toggle").setAttribute("aria-label", t("settingsAria"));
byId("settings-toggle").title = t("settingsAria");
byId("win-minimize").setAttribute("aria-label", t("minimizeAria"));
byId("win-minimize").title = t("minimizeAria");
byId("win-close").setAttribute("aria-label", t("closeAria"));
byId("win-close").title = t("closeAria");
byId("drag-zone").title = t("dragHint");
byId("label").placeholder = t("alias");
byId("label").setAttribute("aria-label", t("alias"));

function effectiveTheme() {
  if (uiPrefs.theme === "light" || uiPrefs.theme === "dark") return uiPrefs.theme;
  return darkMedia.matches ? "dark" : "light";
}

function syncDocumentRootAttributes() {
  const resolved = effectiveTheme();
  document.documentElement.dataset.theme = resolved;
  document.documentElement.dataset.layout = uiPrefs.layout;
  byId("mode-badge").textContent = t(uiPrefs.layout === "list" ? "modeList" : "modeMatrix");
  ensureStarburstLoop();
}

/* --- Radial Starburst + Uniform Idle Particle Engine --- */
const canvas = byId("starburst-canvas");
const ctx = canvas.getContext("2d");
const pointer = {
  x: 260, y: 240,
  tx: 260, ty: 240,
  vx: 0, vy: 0,
  inside: false,
  hoverMix: 0,
};
let stageW = 0;
let stageH = 0;
const particles = [];

const stopsLight = [
  { deg: 0,   r: 36,  g: 99,  b: 245 },
  { deg: 75,  r: 58,  g: 92,  b: 238 },
  { deg: 132, r: 122, g: 68,  b: 214 },
  { deg: 182, r: 218, g: 68,  b: 96  },
  { deg: 232, r: 136, g: 72,  b: 210 },
  { deg: 290, r: 42,  g: 102, b: 246 },
  { deg: 360, r: 36,  g: 99,  b: 245 },
];
const stopsDark = [
  { deg: 0,   r: 78,  g: 140, b: 255 },
  { deg: 75,  r: 96,  g: 126, b: 250 },
  { deg: 132, r: 156, g: 104, b: 242 },
  { deg: 182, r: 240, g: 96,  b: 118 },
  { deg: 232, r: 168, g: 106, b: 238 },
  { deg: 290, r: 82,  g: 142, b: 255 },
  { deg: 360, r: 78,  g: 140, b: 255 },
];

function buildColorLUT(stops) {
  const lut = new Array(360);
  for (let d = 0; d < 360; d++) {
    let s0 = stops[0];
    let s1 = stops[stops.length - 1];
    for (let i = 0; i < stops.length - 1; i++) {
      if (d >= stops[i].deg && d <= stops[i + 1].deg) {
        s0 = stops[i];
        s1 = stops[i + 1];
        break;
      }
    }
    const span = Math.max(1, s1.deg - s0.deg);
    const ratio = (d - s0.deg) / span;
    const st = ratio * ratio * ratio * (ratio * (ratio * 6 - 15) + 10);
    const r = Math.round(s0.r + (s1.r - s0.r) * st);
    const g = Math.round(s0.g + (s1.g - s0.g) * st);
    const b = Math.round(s0.b + (s1.b - s0.b) * st);
    lut[d] = `rgb(${r},${g},${b})`;
  }
  return lut;
}

const lutLight = buildColorLUT(stopsLight);
const lutDark = buildColorLUT(stopsDark);

function hash(n) {
  const x = Math.sin(n * 127.1 + 311.7) * 43758.5453;
  return x - Math.floor(x);
}

function smoothstep(edge0, edge1, x) {
  const k = Math.max(0, Math.min(1, (x - edge0) / (edge1 - edge0)));
  return k * k * (3 - 2 * k);
}

function lerp(a, b, k) {
  return a + (b - a) * k;
}

function lerpAngle(a, b, k) {
  let d = b - a;
  while (d > Math.PI) d -= Math.PI * 2;
  while (d < -Math.PI) d += Math.PI * 2;
  return a + d * k;
}

function lerpDeg(d0, d1, k) {
  const diff = ((d1 - d0 + 540) % 360) - 180;
  return ((Math.round(d0 + diff * k) % 360) + 360) % 360;
}

function rebuildParticleGrid(w, h) {
  stageW = w;
  stageH = h;
  particles.length = 0;
  const pitch = 28;
  const cols = Math.max(10, Math.round(w / pitch));
  const rows = Math.max(8, Math.round(h / pitch));
  const cw = w / cols;
  const ch = h / rows;
  let idx = 1;
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const jx = 0.20 + hash(idx * 1.31) * 0.60;
      const jy = 0.20 + hash(idx * 2.63) * 0.60;
      particles.push({
        hx: (c + jx) * cw,
        hy: (r + jy) * ch,
        seed: hash(idx * 3.91) * Math.PI * 2,
        scale: 0.82 + hash(idx * 5.17) * 0.36,
        hueJitter: Math.round((hash(idx * 7.31) - 0.5) * 6),
      });
      idx++;
    }
  }
}

let rafId = 0;
function renderStarburstFrame(now) {
  if (uiPrefs.layout !== "matrix" || document.hidden) {
    rafId = 0;
    return;
  }

  const rect = canvas.getBoundingClientRect();
  const w = rect.width || window.innerWidth || 504;
  const h = rect.height || Math.max(200, (window.innerHeight || 504) - 48);
  if (Math.abs(w - stageW) > 2 || Math.abs(h - stageH) > 2 || !particles.length) {
    rebuildParticleGrid(w, h);
  }
  const dpr = window.devicePixelRatio || 1;
  const bufW = Math.round(w * dpr);
  const bufH = Math.round(h * dpr);
  if (canvas.width !== bufW || canvas.height !== bufH) {
    canvas.width = bufW;
    canvas.height = bufH;
  }

  ctx.save();
  ctx.scale(dpr, dpr);
  ctx.clearRect(0, 0, w, h);

  const wantHover = uiPrefs.particlesFollow && pointer.inside ? 1 : 0;
  pointer.hoverMix += (wantHover - pointer.hoverMix) * 0.055;
  const m = pointer.hoverMix;

  const prevX = pointer.x;
  const prevY = pointer.y;
  pointer.x += (pointer.tx - pointer.x) * 0.075;
  pointer.y += (pointer.ty - pointer.y) * 0.075;
  pointer.vx = pointer.x - prevX;
  pointer.vy = pointer.y - prevY;

  const isDark = effectiveTheme() === "dark";
  const lut = isDark ? lutDark : lutLight;

  const breath = Math.sin(now * 0.0018) * 6;
  const innerClear = 20;
  const innerPeak = 76 + breath * 0.5;
  const outerStart = 128 + breath;
  const outerLimit = 238 + breath;
  const slowTimeHue = (now * 0.0022) % 360;

  for (const p of particles) {
    const ambWave = 0.66 + 0.34 * Math.sin(p.hx * 0.012 + p.hy * 0.012 - now * 0.0034 + p.seed);
    const ambAngle = -0.52 + Math.sin(now * 0.0012 + p.hx * 0.007 + p.hy * 0.007) * 0.24;
    const ambDrift = (ambWave - 0.66) * 4.2;
    const ambX = p.hx + Math.cos(ambAngle) * ambDrift;
    const ambY = p.hy + Math.sin(ambAngle) * ambDrift;
    const ambLen = (2.5 + ambWave * 2.3) * p.scale;
    const ambThick = 1.45 + ambWave * 0.45;
    const ambAlpha = (isDark ? 0.22 : 0.20) + ambWave * 0.32;
    const ambDeg = ((p.hx / w) * 135 + (1 - p.hy / h) * 75 + slowTimeHue + p.hueJitter + 360) % 360;

    if (m < 0.005) {
      ctx.save();
      ctx.translate(ambX, ambY);
      ctx.rotate(ambAngle);
      ctx.globalAlpha = ambAlpha;
      ctx.fillStyle = lut[Math.floor(ambDeg)];
      ctx.beginPath();
      ctx.roundRect(-ambLen * 0.5, -ambThick * 0.5, ambLen, ambThick, ambThick * 0.5);
      ctx.fill();
      ctx.restore();
      continue;
    }

    const dx = p.hx - pointer.x;
    const dy = p.hy - pointer.y;
    const dist = Math.hypot(dx, dy);

    const rise = smoothstep(innerClear, innerPeak, dist);
    const fall = 1 - smoothstep(outerStart, outerLimit, dist);
    const radWave = 0.74 + 0.26 * Math.sin(dist * 0.078 - now * 0.0036 + p.seed);
    const ring = rise * fall * radWave * p.scale;

    const radialAngle = Math.atan2(dy, dx);
    const radDisp = Math.sin(dist * 0.058 - now * 0.0032 + p.seed) * 3.8 * ring;
    const hovX = p.hx + Math.cos(radialAngle) * radDisp - pointer.vx * ring * 1.3;
    const hovY = p.hy + Math.sin(radialAngle) * radDisp - pointer.vy * ring * 1.3;

    const hovLen = 1.6 + ring * 5.4;
    const hovThick = 1.3 + ring * 1.15;
    const hovAlpha = (isDark ? 0.16 : 0.15) + ring * 0.76;
    const hovDeg = (((radialAngle * 180) / Math.PI) + slowTimeHue * 0.6 + p.hueJitter + 360) % 360;

    const px = lerp(ambX, hovX, m);
    const py = lerp(ambY, hovY, m);
    const alignWeight = m * smoothstep(0.02, 0.28, ring + 0.06);
    const angle = lerpAngle(ambAngle, radialAngle, alignWeight);
    const len = lerp(ambLen, hovLen, m);
    const thick = lerp(ambThick, hovThick, m);
    const alpha = Math.min(0.94, lerp(ambAlpha, hovAlpha, m));
    const deg = lerpDeg(ambDeg, hovDeg, m);

    ctx.save();
    ctx.translate(px, py);
    ctx.rotate(angle);
    ctx.globalAlpha = alpha;
    ctx.fillStyle = lut[deg];
    ctx.beginPath();
    ctx.roundRect(-len * 0.5, -thick * 0.5, len, thick, thick * 0.5);
    ctx.fill();
    ctx.restore();
  }

  ctx.restore();
  rafId = requestAnimationFrame(renderStarburstFrame);
}

function ensureStarburstLoop() {
  if (uiPrefs.layout === "matrix" && !document.hidden && !rafId) {
    rafId = requestAnimationFrame(renderStarburstFrame);
  }
}

/* --- Status & Notices --- */
function setNotice(key, tone = "info") {
  transientNotice = key ? { key, tone } : null;
  renderNoticeAndStatus();
}

function computePersistentIssue() {
  if (!current) return null;
  if (!current.appInstalled) return { key: "app_missing", tone: "error" };
  if (current.recovery === "pending") {
    return { key: current.canRestore ? "recovery_required" : "recovery_unmatched", tone: "error" };
  }
  if (current.hotkeyStatus === "conflict") return { key: "hotkey_conflict", tone: "error" };
  if (current.autostart && current.autostartBlocked) return { key: "autostartBlockedNotice", tone: "warn" };
  if (current.unregisteredActive) {
    return { key: current.accounts.length ? "unmatchedStatus" : "unregisteredStatus", tone: "warn" };
  }
  return null;
}

function renderNoticeAndStatus() {
  const pill = byId("status-pill");
  const pillText = byId("status-pill-text");

  if (!current) {
    pill.dataset.tone = "idle";
    pillText.textContent = t("pillStandby");
  } else if (!current.appInstalled || current.recovery === "pending") {
    pill.dataset.tone = "error";
    pillText.textContent = t("pillRecovery");
  } else if (current.hotkeyStatus === "conflict") {
    pill.dataset.tone = "error";
    pillText.textContent = t("pillConflict");
  } else if (current.unregisteredActive) {
    pill.dataset.tone = "warn";
    pillText.textContent = t("pillUnregistered");
  } else if (current.clientRunning) {
    pill.dataset.tone = "ok";
    pillText.textContent = t("pillRunning");
  } else {
    pill.dataset.tone = "idle";
    pillText.textContent = t("pillStandby");
  }

  const activeIssue = transientNotice ?? computePersistentIssue();
  const bar = byId("notice-bar");
  const needsPendingRecoveryBtns = current?.recovery === "pending";
  const showReload = !current && !working;

  if (!activeIssue && !showReload) {
    bar.hidden = true;
    return;
  }

  bar.hidden = false;
  bar.classList.toggle("is-warn", activeIssue?.tone === "warn");
  bar.classList.toggle("is-error", activeIssue?.tone === "error");
  byId("notice").textContent = activeIssue ? t(activeIssue.key) : "";

  byId("restore").hidden = !needsPendingRecoveryBtns || !current?.canRestore;
  byId("forget").hidden = !needsPendingRecoveryBtns || !current?.recovery;
  byId("restore").disabled = working || recording;
  byId("forget").disabled = working || recording;

  byId("reload").hidden = !showReload;
  byId("reload").disabled = working || recording;
}

function firstAvailableSlot(accounts) {
  for (let slot = 1; slot <= 4; slot++) {
    if (!accounts.some(a => a.slot === slot)) return slot;
  }
  return null;
}

function avatarInitial(label, slot) {
  const trimmed = (label ?? "").trim();
  if (!trimmed) return String(slot);
  return Array.from(trimmed)[0].toUpperCase();
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

function openRegistration() {
  if (working || recording || !current?.appInstalled || current?.recovery === "pending") return;
  const freeSlot = firstAvailableSlot(current?.accounts ?? []);
  if (!freeSlot) return;
  settingsOpen = false;
  registration = true;
  render();
  byId("label").focus();
}

async function requestSwitch(slot) {
  if (working || recording) return;
  settingsOpen = false;
  try {
    await load();
    render();
  } catch (error) {
    setNotice(typeof error === "string" ? error : "operation_failed", "error");
    return;
  }
  if (!current?.appInstalled) {
    setNotice("app_missing", "error");
    return;
  }
  if (current.recovery === "pending") {
    setNotice(current.canRestore ? "recovery_required" : "recovery_unmatched", "error");
    return;
  }
  if (current.clientRunning) {
    const promptKey = current.activeSlot === slot ? "reopenSameRunningPrompt" : "switchRunningPrompt";
    if (!(await confirmAction(promptKey))) {
      transientNotice = null;
      renderNoticeAndStatus();
      return;
    }
  }
  await operate("switch_account", { slot }, "opened");
}

function createAuxActions(slot, label, needsRecovery) {
  const wrap = document.createElement("div");
  wrap.className = "slot-aux";

  if (current?.unregisteredActive && current?.appInstalled && !needsRecovery) {
    const upd = document.createElement("button");
    upd.type = "button";
    upd.className = "aux-btn aux-update";
    upd.textContent = t("updateCredential");
    upd.setAttribute("aria-label", `${t("updateCredential")} ${label}`);
    upd.disabled = working || recording;
    upd.addEventListener("click", async event => {
      event.stopPropagation();
      if (await confirmAction("updateCredentialPrompt", label)) {
        await operate("update_saved_account", { slot }, "credentialUpdated");
      }
    });
    wrap.append(upd);
  }

  const rem = document.createElement("button");
  rem.type = "button";
  rem.className = "aux-btn aux-remove";
  const svgNs = "http://www.w3.org/2000/svg";
  const closeSvg = document.createElementNS(svgNs, "svg");
  closeSvg.setAttribute("width", "11");
  closeSvg.setAttribute("height", "11");
  closeSvg.setAttribute("viewBox", "0 0 24 24");
  closeSvg.setAttribute("fill", "none");
  closeSvg.setAttribute("stroke", "currentColor");
  closeSvg.setAttribute("stroke-width", "2.2");
  closeSvg.setAttribute("stroke-linecap", "round");
  closeSvg.setAttribute("aria-hidden", "true");
  const closePath = document.createElementNS(svgNs, "path");
  closePath.setAttribute("d", "M6 6l12 12M18 6L6 18");
  closeSvg.append(closePath);
  rem.append(closeSvg);
  rem.title = `${t("remove")} ${label}`;
  rem.setAttribute("aria-label", `${t("remove")} ${label}`);
  rem.disabled = working || recording || needsRecovery;
  rem.addEventListener("click", async event => {
    event.stopPropagation();
    if (await confirmAction("removePrompt", label)) {
      await operate("remove_account", { slot }, "removed");
    }
  });
  wrap.append(rem);
  return wrap;
}

function renderSlots() {
  const accounts = current?.accounts ?? [];
  const needsRecovery = current?.recovery === "pending";
  const isMatrix = uiPrefs.layout === "matrix";
  const list = byId("accounts");
  list.replaceChildren();

  for (let slot = 1; slot <= 4; slot++) {
    const saved = accounts.find(a => a.slot === slot);
    const li = document.createElement("li");
    li.className = "slot-item";

    const btn = document.createElement("button");
    btn.type = "button";

    if (saved) {
      const isActive = current?.activeSlot === slot;
      btn.className = isActive ? "slot-trigger is-active" : "slot-trigger";
      if (isActive) btn.setAttribute("aria-current", "true");
      btn.disabled = working || recording || needsRecovery || !current?.appInstalled;
      btn.addEventListener("click", () => void requestSwitch(slot));

      if (isMatrix) {
        const top = document.createElement("div");
        top.className = "card-top";
        const orb = document.createElement("span");
        orb.className = "avatar-orb";
        orb.dataset.slotColor = String(slot);
        orb.textContent = avatarInitial(saved.label, slot);
        const key = document.createElement("kbd");
        key.className = "slot-key";
        key.textContent = String(slot);
        top.append(orb, key);

        const bottom = document.createElement("div");
        bottom.className = "card-bottom";
        const name = document.createElement("span");
        name.className = "account-label";
        name.textContent = saved.label;
        const sub = document.createElement("div");
        sub.className = "account-sub";
        const subText = document.createElement("span");
        subText.textContent = isActive ? t("activeMatrix") : t("idleMatrix");
        const arrow = document.createElement("span");
        arrow.textContent = "→";
        sub.append(subText, arrow);
        bottom.append(name, sub);

        btn.append(top, bottom);
      } else {
        const left = document.createElement("span");
        left.className = "row-left";
        if (isActive) {
          const bar = document.createElement("span");
          bar.className = "rainbow-bar";
          left.append(bar);
        } else {
          const key = document.createElement("kbd");
          key.className = "slot-key";
          key.textContent = String(slot);
          left.append(key);
        }
        const name = document.createElement("span");
        name.className = "account-label";
        name.textContent = saved.label;
        left.append(name);

        const right = document.createElement("span");
        right.className = "row-right";
        if (isActive) {
          const badge = document.createElement("span");
          badge.className = "spectrum-pill";
          badge.textContent = t("activePill").replace("{slot}", String(slot));
          right.append(badge);
        } else {
          const arrow = document.createElement("span");
          arrow.className = "arrow-circle";
          arrow.textContent = "→";
          right.append(arrow);
        }

        btn.append(left, right);
      }

      li.append(btn, createAuxActions(slot, saved.label, needsRecovery));
    } else {
      btn.className = "slot-trigger is-empty";
      btn.disabled = working || recording || needsRecovery || !current?.appInstalled;
      btn.addEventListener("click", openRegistration);

      const unregHint = Boolean(current?.unregisteredActive);
      if (isMatrix) {
        const top = document.createElement("div");
        top.className = "card-top";
        const plus = document.createElement("span");
        plus.className = "avatar-plus";
        plus.textContent = "＋";
        const key = document.createElement("kbd");
        key.className = "slot-key";
        key.textContent = String(slot);
        top.append(plus, key);

        const bottom = document.createElement("div");
        bottom.className = "card-bottom";
        const name = document.createElement("span");
        name.className = "account-label";
        name.textContent = t("emptyTitle").replace("{slot}", String(slot));
        const sub = document.createElement("div");
        sub.className = "account-sub";
        const subText = document.createElement("span");
        subText.textContent = unregHint ? t("emptyUnregistered") : t("emptySub");
        sub.append(subText);
        bottom.append(name, sub);

        btn.append(top, bottom);
      } else {
        const left = document.createElement("span");
        left.className = "row-left";
        const key = document.createElement("kbd");
        key.className = "slot-key";
        key.textContent = String(slot);
        const name = document.createElement("span");
        name.className = "account-label";
        name.textContent = `${t("emptyTitle").replace("{slot}", String(slot))} · ${unregHint ? t("emptyUnregistered") : t("emptySub")}`;
        left.append(key, name);

        const right = document.createElement("span");
        right.className = "row-right";
        const plus = document.createElement("span");
        plus.className = "plus-cta-circle";
        plus.textContent = "＋";
        right.append(plus);

        btn.append(left, right);
      }

      li.append(btn);
    }

    list.append(li);
  }
}

function render() {
  syncDocumentRootAttributes();
  renderSlots();

  const accounts = current?.accounts ?? [];
  const needsRecovery = current?.recovery === "pending";
  const freeSlot = firstAvailableSlot(accounts);

  byId("count").hidden = !current;
  byId("count").textContent = `${accounts.length}/4`;

  // Settings popover state
  byId("settings-popover").hidden = !settingsOpen;
  byId("settings-toggle").setAttribute("aria-expanded", String(settingsOpen));
  byId("mode-list").classList.toggle("is-selected", uiPrefs.layout === "list");
  byId("mode-matrix").classList.toggle("is-selected", uiPrefs.layout === "matrix");
  byId("theme-system").classList.toggle("is-selected", uiPrefs.theme === "system");
  byId("theme-light").classList.toggle("is-selected", uiPrefs.theme === "light");
  byId("theme-dark").classList.toggle("is-selected", uiPrefs.theme === "dark");

  const partSwitch = byId("particles-switch");
  partSwitch.dataset.on = String(uiPrefs.particlesFollow);
  partSwitch.setAttribute("aria-checked", String(uiPrefs.particlesFollow));
  partSwitch.disabled = working;

  const autoSwitch = byId("autostart-switch");
  const autoOn = Boolean(current?.autostart);
  autoSwitch.dataset.on = String(autoOn);
  autoSwitch.setAttribute("aria-checked", String(autoOn));
  autoSwitch.disabled = working || recording || !current;

  // Registration bar
  byId("registration").hidden = !registration;
  if (freeSlot) {
    byId("reg-slot-badge").textContent = `#${freeSlot}`;
  }
  byId("save").disabled = working;
  byId("label").disabled = working;
  byId("cancel-registration").disabled = working;

  // Footer controls
  const showQuickReg = !registration && Boolean(current?.unregisteredActive) && Boolean(freeSlot) && Boolean(current?.appInstalled) && !needsRecovery;
  byId("register").hidden = !showQuickReg;
  byId("register").disabled = working || recording;

  const readyRecovery = Boolean(current?.recovery) && !needsRecovery;
  byId("restore-footer").hidden = !readyRecovery || !current?.canRestore;
  byId("forget-footer").hidden = !readyRecovery;
  byId("restore-footer").disabled = working || recording;
  byId("forget-footer").disabled = working || recording;

  // Hotkey display & buttons
  const hotkeyPill = byId("hotkey-pill");
  const hasHotkey = Boolean(current?.hotkey);
  const isConflict = current?.hotkeyStatus === "conflict";
  if (recording) {
    hotkeyPill.textContent = recordingPreview || t("recordingBadge");
  } else {
    hotkeyPill.textContent = hasHotkey ? current.hotkey : t("hotkeyDisabled");
  }
  hotkeyPill.classList.toggle("is-recording", recording);
  hotkeyPill.classList.toggle("is-conflict", !recording && isConflict);
  hotkeyPill.classList.toggle("is-muted", !recording && !hasHotkey);
  hotkeyPill.disabled = working || !current;

  const editBtn = byId("hotkey-edit");
  const clearBtn = byId("hotkey-clear");
  editBtn.textContent = t("editHotkey");
  editBtn.hidden = recording;
  editBtn.disabled = working || !current;

  clearBtn.textContent = recording ? t("cancel") : t("disableHotkey");
  clearBtn.hidden = !recording && !hasHotkey;
  clearBtn.disabled = working || !current;

  renderNoticeAndStatus();
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

function applyShellStatus(shell) {
  if (!shell) return;
  if (current) {
    current.autostart = shell.autostart;
    current.autostartBlocked = shell.autostartBlocked;
    current.hotkey = shell.hotkey;
    current.hotkeyStatus = shell.hotkeyStatus;
    current.layout = shell.layout;
    current.theme = shell.theme;
    current.particlesFollow = shell.particlesFollow;
  }
  if (shell.layout === "list" || shell.layout === "matrix") {
    uiPrefs.layout = shell.layout;
  }
  if (shell.theme === "system" || shell.theme === "light" || shell.theme === "dark") {
    uiPrefs.theme = shell.theme;
  }
  if (typeof shell.particlesFollow === "boolean") {
    uiPrefs.particlesFollow = shell.particlesFollow;
  }
}

async function load() {
  current = await api.invoke("overview");
  applyShellStatus(current);
}

async function persistPreferences(patch) {
  const prev = { ...uiPrefs };
  Object.assign(uiPrefs, patch);
  render();
  if (!api) return;
  try {
    const shell = await api.invoke("set_preferences", {
      layout: uiPrefs.layout,
      theme: uiPrefs.theme,
      particlesFollow: uiPrefs.particlesFollow,
    });
    applyShellStatus(shell);
    render();
  } catch (error) {
    Object.assign(uiPrefs, prev);
    setNotice(typeof error === "string" ? error : "operation_failed", "error");
    render();
  }
}

async function operate(command, arguments_, successKey) {
  if (working) return;
  working = true;
  setNotice("busy", "info");
  render();
  try {
    await api.invoke(command, arguments_);
    if (command === "register_account") {
      registration = false;
      byId("label").value = "";
    }
    await load();
    if (command === "switch_account" && current.activeSlot !== arguments_.slot) {
      setNotice("opened_unmatched", "warn");
    } else {
      setNotice(successKey, "info");
    }
  } catch (error) {
    try { await load(); } catch { current = undefined; }
    if (error === "recovery_required" && current?.recovery === "pending" && !current.canRestore) {
      setNotice("recovery_unmatched", "error");
    } else {
      setNotice(typeof error === "string" ? error : "operation_failed", "error");
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
  setNotice("recordingPrompt", "info");
  render();
  try {
    await api.invoke("set_hotkey_suspended", { suspended: true });
  } catch {
    // Best-effort suspension while capturing keystrokes.
  }
}

async function cancelRecording(clearPrompt = true) {
  if (!recording) return;
  recording = false;
  recordingPreview = "";
  try {
    await api.invoke("set_hotkey_suspended", { suspended: false });
  } catch {
    // Ignore
  }
  if (clearPrompt) transientNotice = null;
  render();
}

async function submitHotkey(hotkey, successKey) {
  working = true;
  recording = false;
  recordingPreview = "";
  render();
  try {
    const shell = await api.invoke("set_hotkey", { hotkey });
    applyShellStatus(shell);
    setNotice(successKey, "info");
  } catch (error) {
    try { await load(); } catch { /* keep current */ }
    setNotice(typeof error === "string" ? error : "operation_failed", "error");
  } finally {
    working = false;
    render();
  }
}

async function initialize() {
  working = true;
  setNotice("loading", "info");
  render();
  try {
    await load();
    transientNotice = null;
  } catch (error) {
    setNotice(typeof error === "string" ? error : "operation_failed", "error");
  } finally {
    working = false;
    render();
  }
}

/* --- Event Wiring --- */
darkMedia.addEventListener("change", () => {
  if (uiPrefs.theme === "system") syncDocumentRootAttributes();
});

const TOPBAR_HEIGHT = 48;
window.addEventListener("mouseenter", event => {
  pointer.tx = event.clientX;
  pointer.ty = event.clientY - TOPBAR_HEIGHT;
  if (pointer.hoverMix < 0.05) {
    pointer.x = pointer.tx;
    pointer.y = pointer.ty;
  }
  pointer.inside = event.clientY > TOPBAR_HEIGHT;
});
window.addEventListener("mousemove", event => {
  pointer.tx = event.clientX;
  pointer.ty = event.clientY - TOPBAR_HEIGHT;
  pointer.inside = event.clientY > TOPBAR_HEIGHT;
});
document.addEventListener("mouseleave", () => {
  pointer.inside = false;
});
window.addEventListener("resize", () => {
  stageW = 0;
  stageH = 0;
  ensureStarburstLoop();
});
document.addEventListener("visibilitychange", () => {
  if (!document.hidden) ensureStarburstLoop();
});

byId("topbar").addEventListener("mousedown", event => {
  if (event.button !== 0) return;
  if (event.target.closest("button, input, a")) return;
  if (settingsOpen) {
    settingsOpen = false;
    render();
  }
  void api?.invoke("start_drag");
});

byId("win-minimize").addEventListener("click", () => {
  void api?.invoke("minimize_window");
});

byId("win-close").addEventListener("click", () => {
  void api?.invoke("hide_window");
});

byId("theme-toggle").addEventListener("click", () => {
  const next = effectiveTheme() === "light" ? "dark" : "light";
  void persistPreferences({ theme: next });
});

byId("settings-toggle").addEventListener("click", event => {
  event.stopPropagation();
  settingsOpen = !settingsOpen;
  render();
});
byId("settings-close").addEventListener("click", () => {
  settingsOpen = false;
  render();
});
document.addEventListener("click", event => {
  if (!settingsOpen) return;
  if (event.target.closest("#settings-popover") || event.target.closest("#settings-toggle")) return;
  settingsOpen = false;
  render();
});

byId("mode-list").addEventListener("click", () => void persistPreferences({ layout: "list" }));
byId("mode-matrix").addEventListener("click", () => void persistPreferences({ layout: "matrix" }));
byId("theme-system").addEventListener("click", () => void persistPreferences({ theme: "system" }));
byId("theme-light").addEventListener("click", () => void persistPreferences({ theme: "light" }));
byId("theme-dark").addEventListener("click", () => void persistPreferences({ theme: "dark" }));
byId("particles-switch").addEventListener("click", () => {
  void persistPreferences({ particlesFollow: !uiPrefs.particlesFollow });
});

byId("autostart-switch").addEventListener("click", async () => {
  if (working || recording || !current) return;
  const enabled = !current.autostart;
  working = true;
  render();
  try {
    const shell = await api.invoke("set_autostart", { enabled });
    applyShellStatus(shell);
    if (shell.autostart && shell.autostartBlocked) {
      setNotice("autostartBlockedNotice", "warn");
    } else {
      setNotice(shell.autostart ? "autostartEnabled" : "autostartDisabled", "info");
    }
  } catch (error) {
    try { await load(); } catch { /* ignore */ }
    setNotice(typeof error === "string" ? error : "operation_failed", "error");
  } finally {
    working = false;
    render();
  }
});

byId("register").addEventListener("click", openRegistration);
byId("cancel-registration").addEventListener("click", () => {
  registration = false;
  byId("label").value = "";
  transientNotice = null;
  render();
});
byId("registration").addEventListener("submit", event => {
  event.preventDefault();
  void operate("register_account", { label: byId("label").value }, "saved");
});

for (const id of ["restore", "restore-footer"]) {
  byId(id).addEventListener("click", () => void operate("restore_session", {}, "restored"));
}
for (const id of ["forget", "forget-footer"]) {
  byId(id).addEventListener("click", async () => {
    if (await confirmAction("forgetPrompt")) {
      await operate("discard_recovery", {}, "cleared");
    }
  });
}
byId("reload").addEventListener("click", () => void initialize());

byId("hotkey-pill").addEventListener("click", () => void startRecording());
byId("hotkey-edit").addEventListener("click", () => void startRecording());
byId("hotkey-clear").addEventListener("click", () => {
  if (recording) void cancelRecording(true);
  else void submitHotkey(null, "hotkeyCleared");
});

window.addEventListener("blur", () => {
  pointer.inside = false;
  if (recording) void cancelRecording(true);
});

window.addEventListener("focus", async () => {
  ensureStarburstLoop();
  if (working || registration || recording || byId("confirm").open) return;
  try {
    await load();
    render();
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
      recordingPreview = mods.length ? `${mods.join("+")}+…` : "";
      render();
      return;
    }
    void submitHotkey([...mods, mainKey].join("+"), "hotkeySaved");
    return;
  }

  if (working || byId("confirm").open) return;

  if (event.key === "Escape" && !event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
    if (registration) {
      event.preventDefault();
      registration = false;
      byId("label").value = "";
      render();
      return;
    }
    if (settingsOpen) {
      event.preventDefault();
      settingsOpen = false;
      render();
      return;
    }
    const activeTag = document.activeElement?.tagName;
    if (activeTag === "INPUT" || activeTag === "TEXTAREA") return;
    event.preventDefault();
    void api?.invoke("hide_window");
    return;
  }

  if (registration) return;
  const activeTag = document.activeElement?.tagName;
  if (activeTag === "INPUT" || activeTag === "TEXTAREA") return;

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
  recordingPreview = mods.length ? `${mods.join("+")}+…` : "";
  render();
});

void initialize();
