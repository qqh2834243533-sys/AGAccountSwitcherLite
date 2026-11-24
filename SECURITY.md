# 安全策略 (Security Policy)

## 支持版本

| 版本 | 支持状态 |
| --- | --- |
| `0.2.x` | 当前维护（Windows 11 x64，Rust + Tauri 2） |
| `0.1.x` | 历史版本 |
| `archive/dotnet-simulation-prototype` | 历史研究归档，不再维护 |

## 安全边界摘要

- **零网络凭据传输**：本工具不包含 HTTP 客户端、OAuth 回调服务器、自动更新拉取、额度查询或遥测上报逻辑。
- **秘密隔离**：OAuth 凭据字节仅驻留于当前 Windows 用户的 Credential Manager 与 Rust 后端 `Zeroizing` 内存缓冲中（身份比对中间字段在析构时显式清零），不通过 Tauri IPC 进入 WebView2 前端，也不写入日志文件或 `launch-context.json`。
- **进程与启动安全**：仅操作同会话内经可执行映像路径与创建时间戳双重核实的客户端进程；未关闭的可见窗口绝不强杀；重开参数按显式列表和取值校验。只有本次切换中观察到的伴生进程可以重启，冷启动直接打开官方客户端。

详见 [安全设计说明](docs/security/design.md)。

## 报告安全问题

- 请勿在公开 Issue 中粘贴任何真实的 `access_token`、`refresh_token`、`id_token`、Cookie、凭据导出内容或个人账号隐私数据。
- 提交漏洞报告时，请提供操作系统版本、Antigravity 客户端版本、复现步骤及使用合成测试数据构造的最小复现证据。
