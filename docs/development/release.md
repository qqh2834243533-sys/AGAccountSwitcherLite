# 发布候选与签名规划 (Release Guide)

项目遵循语义化版本（SemVer），`VERSION`、`Cargo.toml` 与 `tauri.conf.json` 三处版本号必须严格一致，由 `scripts/package.ps1` 与 GitHub CI 强制校验。

## 候选包构建流程

1. 确认工作树干净（`git status` 无未提交改动），记录当前 Commit SHA 与 Rust 工具链版本。
2. 执行完整测试与零警告检查：
   ```powershell
   cargo fmt --all -- --check
   cargo clippy --locked --all-targets -- -D warnings
   cargo test --locked
   ```
3. 调用 `scripts/package.ps1` 并传入经 SHA-256 核验的微软 `sbom-tool` x64 生成便携候选包：
   ```powershell
   ./scripts/package.ps1 -SbomTool <path-to-sbom-tool-win-x64.exe>
   ```
4. 核验 `artifacts/` 目录输出：
   - `ag-account-switcher-lite-<version>-win-x64.zip`（内含 `ag-account-switcher-lite.exe`、`LICENSE`、`NOTICE.md`、`README.md`、`README.en.md`）
   - `ag-account-switcher-lite-<version>-win-x64.zip.sha256`
   - `ag-account-switcher-lite-<version>-win-x64.spdx.json` 与 `.sbom-summary.json`，ZIP 同时内置 SPDX 清单与清单校验值
5. 确认候选提交的 Windows CI 全绿。再在隔离 Windows 用户或虚拟机中完成四账号登记、运行态关闭门禁、后台刷新回写、失败恢复及项目/对话连续性人工验收后，再由维护者审批是否创建标签与非商业发布草稿（遵循 `CC BY-NC-SA 4.0`）。
6. 如需 Authenticode，签名并加可信时间戳后，使用 `-NoBuild` 打包已签名的候选二进制并重新生成 SHA-256 与 SPDX 清单。

打包脚本重映射 Rust 构建路径，检查候选 PE、ZIP 内容、旁车文件和发布日志的本机路径、邮箱及已知识别格式的凭据。使用 `-NoBuild` 时执行同样的检查。原始 SBOM 校验报告只在临时目录处理，公开摘要仅保留状态、通过数和失败数。

## 本地额度快照升级

1. 升级至 `v0.3.0` 可沿用已有槽位。按账号保存的旧测试缓存会失效，首次成功采集后会生成按槽位存储的 `quota-cache.json`。
2. 隔离验收时，用两个专用账号完成 A → B → A，核对非当前槽位的 5 小时和周额度，以及当前槽位隐藏。覆盖仅含刷新令牌的凭据、无法确认切出槽位、槽位换绑、接口超时和关窗门禁拒绝。
3. 如需回退，用原便携版可执行文件替换即可继续使用已登记的槽位。若要同时清除额度快照，可删除本地数据目录中的 `quota-cache.json`。
