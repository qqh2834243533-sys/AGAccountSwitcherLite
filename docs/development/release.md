# 发布候选与签名规划 (Release Guide)

项目遵循语义化版本（SemVer），`VERSION`、`Cargo.toml` 与 `tauri.conf.json` 三处版本号必须严格一致，由 `scripts/package.ps1` 与 GitHub CI 强制校验。

## 候选包构建流程

1. 确认工作树干净（`git status` 无未提交改动），记录当前 Commit SHA 与 Rust 工具链版本。
2. 执行完整测试与零警告检查：
   ```powershell
   cargo test --locked
   ```
3. 调用 `scripts/package.ps1` 并传入经 SHA-256 核验的微软 `sbom-tool` x64 生成便携候选包：
   ```powershell
   ./scripts/package.ps1 -SbomTool <path-to-sbom-tool-win-x64.exe>
   ```
4. 核验 `artifacts/` 目录输出：
   - `ag-account-switcher-lite-<version>-win-x64.zip`（内含 `ag-account-switcher-lite.exe`、`LICENSE`、`NOTICE.md`、`README.md`、`README.en.md`）
   - `ag-account-switcher-lite-<version>-win-x64.zip.sha256`
   - `ag-account-switcher-lite-<version>-win-x64.spdx.json` 与 `.sbom-validation.json`
5. 在隔离 Windows 用户或虚拟机中完成四账号登记、运行态关闭门禁、后台刷新回写、失败恢复及项目/对话连续性人工验收后，再由维护者审批是否创建标签与非商业发布草稿（遵循 `CC BY-NC-SA 4.0`）。
6. 正式公开分发前建议配置 Authenticode 代码签名与可信时间戳，签名后重新生成 SHA-256 与 SPDX 清单。
