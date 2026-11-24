# 更新记录 (Changelog)

## 0.1.0 (2026-11-24)

- **架构归一（Rust + Tauri 2）**：将 Windows 专用 Tauri 2 四槽账号切换实现提升为主仓唯一工程，原 `.NET 10 WPF` 离线模拟原型归档至 `archive/dotnet-simulation-prototype` 分支；统一采用 `CC BY-NC-SA 4.0` 许可证并补齐 `NOTICE.md` 上游署名链。
- **系统级凭据槽位与回滚事务**：基于当前用户 Windows Credential Manager（`CRED_TYPE_GENERIC`）实现最多 4 个账号登记、原地 `CredWriteW` 原子替换、写入回读校验、启动失败自动回滚及手动「恢复切换前会话」。
- **官方刷新 Token 同步与回退修复**：基于 JWT `sub` 与 `refresh_token` 识别同一账号，在切号与恢复前自动回写官方客户端后台刷新的最新凭据，修复刷新后执行会话恢复误报 `active_changed` 的缺陷，并在前端高亮显示当前活动槽位。
- **1-A 窗口级安全关闭门禁**：客户端运行态切号前增加前端确认提示；先向可见窗口发送 `WM_CLOSE`，若超时仍有可见窗口（如未保存提示框）立即安全中止而不强杀进程，仅在所有可见窗口退入后台后收尾经路径与启动时间核实的驻留进程。
- **外部插件兼容与执行安全收敛**：支持保留 `--remote-debugging-port`、代理等合法启动开关（剥离临时文件夹路径、协议 URI 与注入参数），清理失效 `DevToolsActivePort`，并按显式亲和性与确定性规则恢复 `agy-claude-style-watcher` 等伴生插件；隔离第三方 IDE 同名 `language_server.exe` 避免误拦截。
- **测试、打包与 CI 闭环**：内聚并修复跨进程文件锁子进程真实断言，清理孤立模块与冗余依赖，更新 Windows Rust CI / 候选发布工作流及 SPDX 2.2 SBOM + SHA-256 打包脚本。
