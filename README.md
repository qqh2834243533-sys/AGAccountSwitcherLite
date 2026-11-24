# AG Account Switcher Lite

[English](README.en.md) · [署名与许可声明 (NOTICE)](NOTICE.md) · [安全设计](docs/security/design.md)

**AG Account Switcher Lite (`ag-account-switcher-lite`)** 是面向 Windows 11 x64 的非官方 Antigravity 桌面客户端四账号手动切换工具，基于 Rust 与 Tauri 2 构建。

工具本身不发起第三方浏览器 OAuth、不内置客户端密钥、不查询额度或代理模型流量，也不读写项目文件与对话数据库；所有首次登录与失效重验均由官方 Antigravity 桌面端和官方浏览器流程完成。

## 核心特性

- **四槽位系统级凭据保管**：最多保存 4 个已在官方 Antigravity 登录的账号。槽位凭据与恢复日志仅存放在当前 Windows 用户的 Credential Manager（`CRED_TYPE_GENERIC`），前端界面仅接收槽位号与备注名称。
- **当前账号识别与刷新自动回写**：自动识别当前激活槽位并高亮标记；切号或回退前若检测到官方客户端后台刷新了当前账号 Token，自动回写至对应槽位，避免刷新令牌过期。
- **窗口级安全关闭门禁（1-A）**：
  - 客户端已完全关闭时：点击账号一键切入并启动客户端；
  - 客户端正在运行时：先弹出轻量确认框提醒确认无活跃 Agent 且已保存工作，随后向可见窗口发送 `WM_CLOSE`（不弹托盘菜单、不抢占鼠标或键盘焦点）；
  - **防丢数据硬门禁**：若任意可见窗口在超时内未关闭（例如出现未保存更改提示框），立即中止切号且不改动任何凭据；仅在所有可见窗口正常关闭或隐藏至后台托盘后，才收尾经路径与启动时间双重核实的驻留进程。
- **外部插件与启动开关兼容**：重开客户端时自动清理失效的 `%APPDATA%\Antigravity\DevToolsActivePort`，保留 `--remote-debugging-port`、代理、缩放等安全启动开关（剥离临时文件夹路径与协议深链），并按亲和性规则恢复随宿主退出的伴生插件（如 `agy-claude-style-watcher`）。
- **可验证回滚恢复**：每次切换写入恢复日志并执行回读校验；若客户端启动失败则自动回滚至切换前状态，亦可在退出客户端后手动点击「恢复切换前会话」。

## 使用流程

1. 打开官方 Antigravity 桌面客户端，通过官方浏览器流程完成登录，确认客户端内显示目标账号。
2. 打开 `ag-account-switcher-lite.exe`，点击「登记当前账号」并输入便于区分的备注名称（最多登记 4 个账号）。
3. 切换账号前确认当前无正在执行的 Agent 任务并保存工作，点击目标账号完成切换，最后以官方客户端内显示的身份为准。
4. 若授权过期或官方要求重新验证，请在官方客户端内完成重登；如需更新失效槽位，可移除旧槽位后重新登记。

## 本地构建与打包

需要 Windows 11 x64、Rust Stable (`x86_64-pc-windows-msvc`)、MSVC C++ Build Tools、Windows SDK 及 WebView2 Runtime。详见 [构建说明](docs/development/build.md)。

```powershell
cargo test --locked
cargo build --locked --release
./scripts/package.ps1
```

## 边界说明与已知限制

- **共享认证面边界**：本工具仅更新桌面端使用的 `gemini:antigravity` 系统凭据条目，不改写 `~/.gemini/antigravity-cli/antigravity-oauth-token` 或 `.vscdb`。若独立运行的 `agy` CLI 或 Antigravity IDE 正在运行，工具会阻止切换以避免并发冲突；若平时同时使用 `agy` CLI，请注意 CLI 可能仍保留其独立会话文件状态。
- **非官方契约说明**：Google 尚未公开面向 Windows 独立桌面客户端的第三方多账号激活契约。请务必阅读 [合规与风险说明](docs/security/compliance-assessment.md) 并在认可边界的前提下使用。

## 来源署名与开源许可

本项目衍生自 [anglee0323/agy-switch v4.10.18](https://github.com/anglee0323/agy-switch/releases/tag/v4.10.18)（Commit `815714f9ef431233f663a3c9910071a683de7e5d`，该项目致谢 [lbjlaq/Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager)）。

依据上游协议要求，本项目采用 **[CC BY-NC-SA 4.0](LICENSE)（署名—非商业性使用—相同方式共享 4.0 国际）** 授权发布，**仅限非商业用途**。完整来源链与架构改动列表见 [NOTICE.md](NOTICE.md)。早期 `.NET 10 WPF` 离线模拟原型已归档至 `archive/dotnet-simulation-prototype` 分支。

本项目独立于 Google LLC，无隶属、授权或背书关系。
