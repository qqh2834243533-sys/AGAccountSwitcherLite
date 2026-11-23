# Phase 0 可行性调查

查询日期为 2026-11-23。依据官方公开资料、只读安装元数据和第三方仓库公开设计说明。

## 本机环境

Windows 11 x64 26H2，系统构建 26300.9974，Git 2.55.0.windows.3。全局 dotnet 原有 .NET 8.0.15 x64 宿主和桌面运行时，未发现 SDK。已将微软官方 Windows x64 ZIP 部署至忽略目录 `.local/dotnet`，SHA-512 核对通过。使用 SDK 10.0.404 和 Windows Desktop Runtime 10.0.15。[.NET 支持策略](https://dotnet.microsoft.com/en-us/platform/support/policy/dotnet-core) · [发布元数据](https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/10.0/releases.json)

开始菜单快捷方式指向 `%LOCALAPPDATA%\Programs\antigravity\Antigravity.exe`。文件版本 2.34.0，产品版本 2.34.0.0，版本资源公司字段为 Google。路径符合按用户安装布局。常见卸载注册表项未命中，具体安装器和分发渠道未知。本轮未启动或修改 Antigravity，也未访问登录、配置或会话数据。

## 可行性

- **可行。** 离线演练、随机确认、模拟恢复和手动登录指引。Windows Named Mutex 支持窗口会话范围的跨进程互斥。[微软文档](https://learn.microsoft.com/en-us/windows/win32/termserv/kernel-object-namespaces)
- **受限。** 个人账号通过客户端发起浏览器 OAuth。官方换号路径是先退出再登录。同机各认证入口可能受到影响。没有确认可供此工具使用的快速换号和全局任务安全接口。[认证](https://antigravity.google/docs/install) · [换号说明](https://antigravity.google/docs/workspace)
- **待验。** 历史可见性、打开同一项目、继续原对话及不重启换号分别记录于 [官方能力矩阵](official-compatibility.md)。

## 下一阶段

1. 核实 Google 是否提供适用于独立桌面 App 的换号、活动身份或任务检测契约，以及同机共享认证的影响。
2. 获得单次批准后，用测试账号和空白项目逐项检查真实人工登录及 A–D，记录恢复路径。
3. 在不同 Windows 缩放和 Antigravity 版本验收指引，再复核 Windows CI、候选包与签名流程。

[合规调查](../security/compliance-assessment.md) · [参考项目](reference-projects.md)
