# 参与开发

在 Windows x64 安装 [global.json](global.json) 指定的 .NET 10 SDK 与 Git。按照 [构建文档](docs/development/build.md) 进行锁定还原、构建和测试。自动测试使用虚拟账号和纯模拟后端。

使用 GitHub Flow，稳定分支为 `main`。开发分支使用 `feat/*`、`fix/*`、`refactor/*`、`test/*`、`docs/*`、`chore/*` 或 `security/*`。提交遵循 Conventional Commits，例如 `test: cover interrupted simulation recovery`。PR 应说明行为变化、验证证据和安全影响。重要架构决策写入 `docs/adr`。

本地 Git 身份可用 `git config --local user.name "姓名"` 和 `git config --local user.email "邮箱"` 设置。提交前排除真实账号、凭据、私人日志、客户端数据及本机生成产物。真实客户端测试适用 [单独批准流程](docs/security/integration-approval.md)。
