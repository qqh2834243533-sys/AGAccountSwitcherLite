# Antigravity 官方能力与兼容性

查询日期为 2026-11-23。Supported 表示存在明确的官方路径。Unsupported 表示本工具不提供该能力，或条款明确禁止该用法。Unknown 表示缺少公开契约或待实测。

| 能力 | 官方证据 | 本机 2.34.0 |
| --- | --- | --- |
| Windows x64 App | Supported。[官方渠道](https://antigravity.google/download) | 确认版本与开始菜单项，具体安装器待核实 |
| 人工退出并从 App 走浏览器 OAuth | Supported。[认证](https://antigravity.google/docs/install) · [切换说明](https://antigravity.google/docs/workspace) | 用户报告 A → B → A 双向登录成功，使用 Windows 数字密码验证 |
| 已授权个人账号免浏览器快速换号 | Unknown，未找到公开接口。[认证](https://antigravity.google/docs/install) | Unsupported，未实现 |
| 全局 Agent 状态、未完成操作和当前身份查询 | Unknown。官方描述了对话内的子 Agent、后台任务与 Goals，以及可独立运行的远程任务。[任务面板](https://antigravity.google/docs/side-panel/#sections) · [远程 Agent](https://antigravity.google/docs/remote-agents) | Unsupported，只有模拟快照 |
| A · 跨账号可见相同本地历史 | Unknown。有历史搜索，无跨账号保证。[搜索与命令面板](https://antigravity.google/docs/search) | 真实环境未测 |
| B · 另一账号打开同一本地项目 | Unknown。有本地项目支持，无切换后的保证。[功能概览](https://antigravity.google/docs/features) | 真实环境未测 |
| C · 跨账号继续原对话 | Unknown。官方描述同账号的多入口访问。[远程 Agent](https://antigravity.google/docs/remote-agents) | 真实环境未测 |
| D · 不重启桌面客户端换号 | Unknown。人工流程未承诺进程连续。[切换说明](https://antigravity.google/docs/workspace) | 用户观察应无需重启，主进程连续性未确认 |
| 第三方接入服务、替换 OAuth 凭据 | Unsupported，条款限制第三方访问，项目禁用凭据替换。[附加条款](https://antigravity.google/terms) | 未实现 |

本轮双账号试验聚焦轻量换号。用户已回报 A → B → A 的结果和验证方式。方案考虑了 App、CLI 和 IDE 扩展统一认证带来的潜在影响。 [验收记录](../development/verification.md) · [试验范围与停止条件](../security/integration-approval.md) · [Google 官方认证文档](https://antigravity.google/docs/install#unified-sign-in-across-ide-extensions)
