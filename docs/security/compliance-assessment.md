# 合规与风险评估 (Compliance Assessment)

更新日期为 2026-11-24。依据公开资料、上游许可条款及本地架构实现整理。具体账户适用的国家或地区条款须在实际使用前自行核对。

## 1. 官方服务条款与技术边界

| 证据级别 | 结论与当前实现边界 |
| --- | --- |
| **官方明确支持** | Antigravity 桌面客户端发起官方浏览器 OAuth 登录；官方文档说明个人账号标准换号流程为退出后重新登录，且同机 App、CLI、IDE 等入口可共享本地认证状态。[认证文档](https://antigravity.google/docs/install) · [换号说明](https://antigravity.google/docs/workspace) |
| **官方明确禁止** | Antigravity 附加条款第 6 条禁止滥用、干扰及使用未获授权的第三方代理/工具复用 OAuth 直接访问后端服务；Google 通用条款禁止绕过安全保护措施。本项目**不包含**任何第三方 OAuth 客户端、Token 网络刷新、额度轮询、并发账号池、自动轮换或模型 API 反代。[Antigravity 附加条款](https://antigravity.google/terms) · [Google 通用条款](https://policies.google.com/terms?hl=en) |
| **技术可行，官方契约未公开** | 在当前 Windows 用户 Credential Manager 内备份与原地切换官方已生成的 `gemini:antigravity` 凭据条目并重启官方客户端。Google 尚未公开面向 Windows 独立桌面端的第三方多账号切换契约，是否允许此类本地状态切换以官方解释为准。 |
| **已移除的高耦合路径** | 早期实验曾评估 Windows UI Automation 驱动系统托盘菜单，现已在正式架构中完全移除，改为无焦点抢占的 `WM_CLOSE` 窗口级安全关闭门禁。 |

## 2. 开源与衍生许可合规

- 本项目衍生自 `anglee0323/agy-switch v4.10.18`（其致谢 `lbjlaq/Antigravity-Manager`），严格遵循 **[CC BY-NC-SA 4.0](../../LICENSE)** 许可证：
  1. **署名（Attribution）**：随仓库与发行包附带 [NOTICE.md](../../NOTICE.md)，完整保留上游作者、来源链接、许可链接及逐项修改说明；
  2. **非商业性使用（NonCommercial）**：禁止将本软件或其构建产物用于商业目的或有偿分发；
  3. **相同方式共享（ShareAlike）**：任何修改后的衍生分发版本须采用相同或 Creative Commons 认可的兼容许可。
