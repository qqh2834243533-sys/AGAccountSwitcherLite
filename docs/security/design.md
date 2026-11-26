# 安全设计 (Security Design)

| 安全域 | Rust + Tauri 2 实现机制 |
| --- | --- |
| **额度采集网络边界** | Rust 后端在用户确认切号后，仅向已核实归属的 Antigravity `127.0.0.1` 监听端口调用状态和额度汇总接口。HTTP 客户端禁用代理、重定向和重试，采集共享时间预算。 |
| **凭据存储与内存保护** | 槽位凭据（`ag-account-switcher-lite:v1:account:1..=4`）与恢复日志存放在当前 Windows 用户的 Credential Manager（`CRED_TYPE_GENERIC`）；`CredWriteW` 原地替换避免先删后写断档；原始字节装入 `Zeroizing<Vec<u8>>`，JWT `sub` 与 `refresh_token` 解析结构体在 `Drop` 时显式执行 `zeroize()` 清零。 |
| **额度缓存** | `quota-cache.json` 按槽位保存快照及单向凭据摘要。采集前后检查当前 Windows 凭据是否仍唯一匹配用户确认的切出槽位，槽位换绑后旧快照不再显示。额度归属依赖活动凭据与运行中的客户端会话一致。采集只请求额度汇总，RPC 认证信息保留在后端内存。 |
| **WebView2 与 IPC 隔离** | `tauri.conf.json` 配置严格 CSP（仅允许 `'self'` 与本地 IPC），`capabilities/main.json` 权限集为空（`permissions: []`）。后端为非当前槽位提供脱敏额度，缓存键保留在后端。 |
| **并发互斥** | `tauri-plugin-single-instance` 保证单窗口实例，`SwitchLock` 通过 `OpenOptionsExt::share_mode(0)` 独占锁定 `%LOCALAPPDATA%\org.ag-account-switcher.lite\account-switch.lock` 串行化全部凭据与切换事务，并在切号事务进行时阻止托盘中途退出。 |
| **进程核实与窗口门禁** | 仅操作当前会话内经 `QueryFullProcessImageNameW` 与 `GetProcessTimes` 双重核实的进程；运行态切换先提示确认并向可见窗口投递 `WM_CLOSE`，若可见窗口超时未关（如未保存提示）立即中止不强杀；排除 Node Sidecar 与无关第三方 IDE 的同名 `language_server.exe`。 |
| **启动参数与伴生进程** | `%LOCALAPPDATA%\org.ag-account-switcher.lite\launch-context.json` 只保存通过取值校验的主程序参数，内容变化时原子替换。磁盘配置不能指定伴生程序、参数或工作目录。运行态退出时只考虑同会话的实时父链或带显式亲和性标记的附加进程，只传递其生命周期参数，并以观察到的映像目录启动。 |
| **开机自启与全局热键** | 开机自启默认关闭，仅在用户显式开启时向当前用户 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\AGAccountSwitcherLite` 写入双引号包裹的可执行文件路径与 `--autostart`（无需管理员提权，不修改 `StartupApproved\Run`）；自定义唤出热键保存在 `shell.json`，拦截系统保留组合键并在冲突时安全回退。 |
| **卸载与清理** | 在界面关闭「开机自启」（或删除注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\AGAccountSwitcherLite`），删除便携版目录、`%LOCALAPPDATA%\org.ag-account-switcher.lite` 目录，并在 Windows 凭据管理器中移除 `ag-account-switcher-lite:v1:*` 条目即可完整清理，不影响官方客户端安装文件与项目数据。 |
