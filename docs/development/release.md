# 发布候选与签名规划

发布候选使用 VERSION、CHANGELOG.md、锁定文件和 global.json 中的固定 SDK。项目版本遵循 SemVer。0.1.0 覆盖模拟 MVP，0.3.0 等真实集成版本须以官方能力和验收证据为准。

1. 在干净检出目录按 [构建说明](build.md) 执行还原、分析、测试和 Windows x64 发布。记录 Git 提交、SDK、哈希及 Windows runner 镜像。
2. 用 scripts/package.ps1 生成框架依赖 ZIP 和 SHA-256。使用脚本的 -SbomTool 参数传入已核对 SHA-256 的微软 sbom-tool x64，生成并验证 SPDX 2.2 物料清单。[微软 SBOM 工具](https://github.com/microsoft/sbom-tool)
3. 检查 ZIP 内容和 SHA-256，验证运行、正常退出、删除目录后的卸载。包需安装 .NET 10 Desktop Runtime x64。复跑相同步骤可核对工件内容。ZIP 时间戳及 SBOM 标识可能随执行改变，当前未承诺逐字节相同。
4. 审查依赖及许可证后，由维护者批准是否建立 v<版本> 标签与 GitHub Release 草稿。附上 ZIP、校验、SBOM、Release Notes 和已知限制。创建远程仓库、推送、公开及正式发布分别由用户授权。
5. 稳定发行前选定 Authenticode 签发与受信时间戳方案，签名后重跑校验和冒烟测试。后续再决定签名安装器及更新方案。

Windows release candidate 工作流仅手动触发，生成候选构建工件，使用固定版本和 SHA-256 校验微软 SBOM 工具。工作流没有创建标签或 GitHub Release 的写权限。

远程仓库建立后可启用 PR Review、main 分支保护、Dependabot、CodeQL C# 与 Secret Scanning，具体可用性以仓库设置为准。
