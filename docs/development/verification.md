# 验证与准入记录 (Verification Log)

## 2026-11-24 `v0.2.1` 本地候选

- 离线格式、严格 Clippy 和 JavaScript 语法检查通过。Rust 默认测试通过 31 项，4 项标记为忽略。回归覆盖旧启动配置迁移、相同启动计划不重写、安装路径筛选、凭据字节上限与热键恢复失败。
- 用模拟 IPC 验证连续选择槽位时只出现一次确认，取消时不提交。过期的状态刷新不会覆盖最近保存的界面偏好。启用减弱动态效果时 Canvas 完成静态绘制后停止请求下一帧。
- 候选 ZIP 内的 EXE 与本地 Release 二进制哈希一致。SPDX 2.2 校验通过，含项目本身共 503 个包。对 ZIP 内 7 个条目复查本机路径和常见凭据格式，未命中扫描规则。
- 候选 ZIP SHA-256 为 `76b1338ad76aed9f02f4d51211baee53a1373688f7aa4d078bd59f814585cf5d`。二进制未签名。四账号真机往返、跨日刷新和项目／对话连续性待签收。

## 2026-11-24 `v0.2.0` 无边框双模式光谱界面与 DPI 自适应重构验证

- **核心改动范围**：重构 `ui/`（`index.html`、`style.css`、`app.js`）为统一 Antigravity 光谱语言的无边框圆角窗口，支持「矩阵 / 列表」双布局、深浅色切换及自适应放射星环 `<canvas>`（隐藏回托盘自动暂停渲染，后台 0% CPU）；扩展 `src/shell.rs` 与 `src/main.rs` 持久化界面偏好、压制 DWM 四周系统强调色边线并按显示器工作区与 DPI 自适应 `1:1` 偶数物理像素尺寸；核心凭据与安全层（`vault.rs`、`service.rs`、`client.rs`、`switch_lock.rs`）及 `capabilities/main.json`（`permissions: []`）保持不变，SPDX 2.2 SBOM 维持 502 个 Rust 依赖组件零新增。
- **离线质量门禁与夹具测试**：
  - `cargo fmt --all -- --check`、`cargo clippy --locked --offline --all-targets -- -D warnings` 零差异、零告警通过；
  - 前端 DOM ID、双语词条完整性及严格 `style-src 'self'` 零内联样式校验通过；
  - `cargo test --locked --offline` **29 项通过 / 0 失败 / 4 项真机或子进程辅助忽略用例**（新增 `shell.json` 旧版兼容与布局/主题枚举解析测试、多分辨率与多 DPI `1:1` 物理像素自适应不变量测试）。

---

## 2026-11-24 `v0.1.1` 托盘常驻、开机自启与自定义快速唤出发布验证

- **核心改动范围**：新增单一壳层模块 `src/shell.rs` 并最小化扩展 `src/main.rs` 与 `ui/`；核心安全层（`vault.rs`、`service.rs`、`client.rs`、`switch_lock.rs`）及 `capabilities/main.json`（`permissions: []`）保持 0 行改动，SPDX 2.2 SBOM 维持 502 个 Rust 依赖组件零膨胀。
- **离线质量门禁与夹具测试**：
  - `cargo fmt --all -- --check` 与 `cargo clippy --locked --offline --all-targets -- -D warnings` 零差异、零告警通过；
  - `cargo test --locked --offline` **24 项通过 / 0 失败 / 4 项真机或子进程辅助忽略用例**（新增隔离注册表子键自启写入/引号包裹/路径漂移自愈/`StartupApproved` 禁用检测测试、热键解析黑名单测试、`shell.json` 损坏恢复测试及多线程热键冲突 `1409` / 挂起恢复集成测试）。
- **GitHub Release 产物与校验值**：
  - 发布页：`https://github.com/qqh2834243533-sys/AGAccountSwitcherLite/releases/tag/v0.1.1`
  - `ag-account-switcher-lite-0.1.1-win-x64.zip`（SHA-256：`0f2458fbdcb49b6f446e9289808b90ba226139a8aca68343baafde0312d8d0e1`）
  - 随附上传 `.zip.sha256`、`.spdx.json`（`sbom-tool validate` 零错误通过）及 `NOTICE.md`。

---

## 2026-11-24 `v0.1.0` GitHub 正式发布记录

- **远端仓库**：`https://github.com/qqh2834243533-sys/AGAccountSwitcherLite`
- **已推送分支与标签**：`main`、`archive/dotnet-simulation-prototype`（`db7e6b9`），正式发布标签 `v0.1.0`（Commit `24ac0db`）。
- **发布前收尾修复（`24ac0db`）**：
  - 修复 `src/client.rs` 中 `clippy::collapsible_if` 告警，确保本地与云端 `cargo clippy --locked --all-targets -- -D warnings` 零告警通过；
  - 在 `scripts/package.ps1` 的 SBOM 扫描临时目录生成最小 `src/main.rs` 桩文件，消除微软 `sbom-tool` 调用 `cargo metadata` 时的回退告警（完整解析 502 个 Rust 依赖组件）；
  - 更新 `.github/ISSUE_TEMPLATE/bug.yml` 为正式版桌面切换器反馈模板。
- **GitHub Release 产物与校验值**：
  - 发布页：`https://github.com/qqh2834243533-sys/AGAccountSwitcherLite/releases/tag/v0.1.0`
  - `ag-account-switcher-lite-0.1.0-win-x64.zip`（SHA-256：`5bd5eb5c8df5b4708e90424a05b7cf1488f05691d358991e3c5287864a2c2810`，与 GitHub 远端资产摘要一致）
  - 随附上传 `.zip.sha256`、`.spdx.json`（SPDX 2.2 SBOM）及 `NOTICE.md`。
- **GitHub Actions 状态**：已启用仓库 Actions 权限（`actions.enabled = true`），`Windows CI` 工作流（Run `#45364091261`）云端格式检查、Clippy 严格门禁与全部系统凭据夹具单元测试均已通过。

---

## 2026-11-24 主仓架构归一与发布准入整改验收

环境：Windows 11 26H2 x64（Build `26300.9974`），Cargo `1.100.0`（MSVC x64），本地离线锁定依赖与微软 `sbom-tool` v4.1.5。

| 验收域 | 结果与证据 |
| --- | --- |
| **仓库归一与历史归档** | 原 `.NET 10 WPF` 离线模拟原型完整归档至 `archive/dotnet-simulation-prototype`（`db7e6b9`）；主仓切换为单一 Rust + Tauri 2 工程，移除跨 `.local/` 源码引用与孤立 `tray.rs` 模块。 |
| **许可与署名合规** | 根目录 `LICENSE` 更新为 `CC BY-NC-SA 4.0` 正文，新增 `NOTICE.md` 披露 `anglee0323/agy-switch v4.10.18` 与 `lbjlaq/Antigravity-Manager` 完整来源链及改动明细；PE `VersionInfo.LegalCopyright` 已写入版权与非商业许可声明。 |
| **代码格式与零警告构建** | `cargo fmt --all -- --check` 零差异通过；`RUSTFLAGS="-D warnings"` 离线锁定测试与 Release 构建 0 警告、0 错误通过。 |
| **核心单元与系统凭据夹具测试** | **19 项通过、0 失败、4 项辅助/真机忽略用例**。覆盖四槽上限、一键切换与显式恢复、官方后台刷新 Token 回写槽位、刷新后执行 `restore_session` 成功回退、已完成槽位直接移除、启动失败自动回滚、外部登录状态保护、多格式 JWT `sub` / `refresh_token` 识别与清零。 |
| **窗口关闭门禁与进程隔离测试** | 验证 `WM_CLOSE` 仅投递给可见用户窗口（忽略隐藏窗口及 Windows 11 系统注入的 `UAC_InputIndicatorOverlayWnd` / `WS_EX_TOOLWINDOW` 覆盖层）；当可见窗口超时未关时严格返回 `Err("close_timeout")` 不调用 `TerminateProcess`；第三方 IDE 同名 `language_server.exe` 不误触发拦截。 |
| **外部插件亲和与参数过滤测试** | 验证 `agy-claude-style-watcher` 等带显式亲和性的伴生进程可被确定性识别与原子落盘，无关 `--exit-on-host-close` 守护进程与篡改配置被拒绝；`--remote-debugging-port`（等号与双词格式）、代理、缩放参数保留，临时文件夹路径、`antigravity://` 深链与 `--import`/`--inspect` 被剥离。 |
| **跨进程文件锁反假阳性验证** | 子进程锁探针实测执行 `1 passed` 并输出 `AGASL_CHILD_LOCK_PROBE_VERIFIED` 校验标记，根除路径失配导致的 `0 tests` 假阳性。 |
| **候选 ZIP、SHA-256 与 SPDX SBOM** | `scripts/package.ps1` 完成三处版本号一致性检查、Release 打包、ZIP 内 `ag-account-switcher-lite.exe` 哈希回读比对、`.sha256` 生成及微软 `sbom-tool` SPDX 2.2 清单生成与 `validate` 校验通过。 |
| **待人工签收项（发布前）** | 1. 隔离环境四账号完整真机往返及跨日刷新存活率记录；<br>2. 换号前后本地项目与原对话连续性人工核验；<br>3. 公开发行前的 Authenticode 代码签名（当前便携候选包为 `NotSigned`）。 |

---

## 历史记录归档摘要（2026-11-23）

- **`.NET 10 WPF` 离线模拟阶段（已归档至 `archive/dotnet-simulation-prototype`）**：完成 28 项 xUnit 模拟状态机测试与离屏 UI 自动化验证，作为早期防误触与恢复状态机设计参考保留。
- **官方 App 手动换号基准与双账号快切探索**：在 Windows 11 + Antigravity 2.34.0 上确认官方手动 A → B → A 往返可用，并验证通过 `gemini:antigravity` 系统凭据条目可在客户端重启后实现双账号免浏览器回访切换，详见 [上游审计与路线决定](../research/agy-switch-route-audit.md) 与 [目标一致性审计](../research/goal-alignment-audit.md)。
