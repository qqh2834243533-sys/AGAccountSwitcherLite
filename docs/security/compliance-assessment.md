# 合规调查

查询日期为 2026-11-23。依据公开资料及本地安装元数据。通用条款引用公开英文页面，具体账户适用的国家或地区版本须在真实测试前核对。

| 证据级别 | 结论 |
| --- | --- |
| 官方明确支持 | Antigravity App 从客户端发起浏览器 OAuth。Google Workspace 文档明确写出退出后重新登录，再选择目标 Google 账号的路径。[认证](https://antigravity.google/docs/install) · [Workspace](https://antigravity.google/docs/workspace) |
| 官方明确禁止 | Antigravity 附加条款第 6 条禁止滥用和干扰，并将第三方工具接入服务列为违约。Google 通用条款禁止绕过其系统或保护措施。[Antigravity 条款](https://antigravity.google/terms) · [Google 条款](https://policies.google.com/terms?hl=en) |
| 技术可行，适用性未知 | Windows UI Automation 可访问部分桌面控件。控件状态无法确立官方认证能力或全局任务状态。当前仅用于自有 WPF 验证。[Microsoft UI Automation](https://learn.microsoft.com/dotnet/framework/ui-automation/ui-automation-overview) |
| 尚无可靠依据 | 未找到可用于个人账号免浏览器换号、跨账号继续会话或独立工具查询全部活动任务的公开契约。[FAQ](https://antigravity.google/docs/faq) · [功能概览](https://antigravity.google/docs/features) |

FAQ 确认个人 Google 账号适用，计划文档说明额度随登录计划变化。未发现专门规定同一用户持有四个账号并手动切换的条款。自动轮换、配额池与规避保护措施不在项目范围。[FAQ](https://antigravity.google/docs/faq) · [计划](https://antigravity.google/docs/plans)

浏览器 OAuth 由用户在官方客户端发起。官方文档说明同机 App、CLI、IDE 等入口共享认证，真实验证前须将潜在影响写入批准范围。[认证](https://antigravity.google/docs/install) · [集成批准](integration-approval.md)
