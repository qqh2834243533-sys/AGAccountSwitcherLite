# 架构总览 (Architecture Overview)

## 模块分层

AG Account Switcher Lite (`0.1.2`) 是面向 Windows 11 x64 的单可执行文件 Tauri 2 应用，由四个高内聚层组成：

1. **WebView2 表现层 (`ui/index.html`, `ui/app.js`, `ui/style.css`)**
   - 原生 HTML/CSS/JS 实现，零第三方前端框架依赖，启用严格 CSP 且 `capabilities/main.json` 权限集为空（`permissions: []`）。
   - 仅接收脱敏后的槽位编号（`1..=4`）、备注名称、当前活动槽位标识、恢复状态及自启/热键壳层状态；支持自定义热键录制、数字键 `1`–`4` 切号与 `Esc` 隐藏回托盘，客户端正在运行时弹出轻量切换确认对话框。
2. **IPC、桌面壳层与互斥入口层 (`src/main.rs`, `src/shell.rs`, `src/switch_lock.rs`)**
   - 注册 `tauri-plugin-single-instance` 保证同一会话内单窗口实例，二次运行自动唤醒托盘隐藏窗口（`--autostart` 开机静默入驻托盘）；
   - `shell.rs` 统一管理系统托盘常驻、当前用户注册表 `HKCU\...\Run` 开机自启（含便携目录迁移路径自愈）与独立 Win32 消息循环全局热键线程（`shell.json` 原子持久化自定义热键）；
   - 所有凭据命令统一在 `with_vault` 阻塞池任务中持有 `SwitchLock`（`account-switch.lock`, `share_mode(0)`）与活跃事务计数保护，保证进程内与跨进程串行互斥且防止切号中途托盘强退。
3. **切换与恢复事务层 (`src/service.rs`)**
   - 管理最多 4 个槽位登记、移除、切号、回读验证与失败自动回滚；
   - 切出或恢复前，按 JWT `sub` 或未变化的 `refresh_token` 找到唯一匹配槽位并回写活动凭据。缺少共同身份信息时，用户可在官方客户端核对账号并退出，再确认更新原槽位。
   - 首次加载及后续刷新状态时，在切换锁内核对挂起的 `Journal`（`p1`/`p0`）。当前凭据与切换前状态完全一致时清理记录。只有仍能匹配目标槽位时才显示挂起事务的恢复入口。已就绪记录（`r1`/`r0`）仍可手动处理。
4. **平台集成层 (`src/vault.rs`, `src/client.rs`, `src/client/graceful.rs`, `src/client/terminate.rs`)**
   - `vault.rs`：封装 Win32 `CredReadW` / `CredWriteW` / `CredDeleteW`，原地替换凭据消除“先删后写”断档窗口，敏感内存使用 `Zeroizing` 并在身份提取析构时显式 `zeroize`。
   - `client.rs`：核实同会话进程归属（排除 Node Sidecar 与无关第三方 IDE 的 `language_server.exe`），执行两阶段安全关闭（`WM_CLOSE` 可见窗口关闭门禁 + 后台托盘驻留进程收尾），清理失效 `DevToolsActivePort`，并按安全过滤规则重放启动标志与 Antigravity 亲和伴生进程。
   - 切号更新 `gemini:antigravity` 系统凭据，`credential-user-scope` 由官方客户端管理。启动后若本地凭据无法匹配目标槽位，界面提示到原生 App 核对账号。
