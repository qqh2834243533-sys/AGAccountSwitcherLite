# 0.1.0 验证与准入记录 (Verification Log)

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
