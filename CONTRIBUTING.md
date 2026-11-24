# 贡献指南 (Contributing)

## 许可与合规前提

本项目衍生自 `anglee0323/agy-switch`（`CC BY-NC-SA 4.0`），所有贡献内容均须兼容 **[CC BY-NC-SA 4.0](LICENSE)**（署名—非商业性使用—相同方式共享 4.0 国际），且保留 [NOTICE.md](NOTICE.md) 中的完整上游署名与修改说明。

## 架构与安全约束

提交代码变更前请确认满足以下边界：

1. **不触碰认证协议层**：禁止引入自建 OAuth 登录/刷新客户端、Token 明文文件导入导出、API 反代、自动轮换或额度轮询功能。
2. **最小权限与零泄漏**：前端 `capabilities/main.json` 保持最小权限；IPC 返回结构与错误码不得包含任何原始凭据或服务器响应正文。
3. **测试隔离红线**：自动化单元测试必须使用合成凭据（`Vault::fixture`）与隔离临时目录，严禁在默认 `cargo test` 中读写生产 `gemini:antigravity` 凭据或终止宿主正在运行的 Antigravity 实例。
4. **质量门禁**：提交前确保 `VERSION`、`Cargo.toml` 与 `tauri.conf.json` 版本号一致，并通过 `cargo fmt --all -- --check`、`cargo clippy --locked --all-targets -- -D warnings`（或 `RUSTFLAGS="-D warnings" cargo test --locked`）及 `./scripts/package.ps1` 验证。
