# AG Account Switcher Lite 目标一致性审计

审计日期为 2026-11-23，环境目标为 Windows 11、官方 Antigravity App 2.0 系列。本报告基于当前工作树、全部本地 Git 历史、仓库内的设计和验收计划、已有测试以及 Google 公开资料。

**结论。** 0.1.0 是可运行的离线切号原型，真实账号切换的交付进度为零。已有记录支持用户在官方 App 手动完成双账号往返。四个账号在同一官方 App 中预授权后快速激活、减少回访时的浏览器验证，以及同一本地项目和对话的连续性，均未达成验收。官方手动路径可以作为基准。产品的四账号目标应继续作为可行性调查的验收条件。

## Git 基线和审计边界

| 项目 | 审计开始时的状态 |
| --- | --- |
| 分支与提交 | `main`，`db7e6b9bb3994fb9058aa05310244947a1115d32`，2026-11-23 17:48:15 UTC+08:00，提交说明为 `chore: initialize Antigravity switch simulation` |
| 历史 | 本地 Git 引用可见这一条提交，无标签，无配置的远程 |
| 索引和未跟踪状态 | 无暂存修改，无未忽略的未跟踪文件 |
| 六处未暂存修改 | `docs/development/verification.md`、`docs/research/official-compatibility.md`、`docs/security/integration-approval.md`、`src/AGSwitcher.App/App.xaml`、`src/AGSwitcher.App/MainWindow.xaml`、`src/AGSwitcher.App/MainWindow.xaml.cs` |
| 差异内容 | 三份文档补充第二轮离线验收、官方手动换号试验和结果。三处 WPF 文件涉及按钮文字对比度、Esc 取消和焦点。合计新增 96 行、删除 13 行。原有忽略项包括 `.local/`、`artifacts/` 及各项目的 `bin/`、`obj/` |

本轮只新增本报告。复跑使用现有代码的隔离副本和本地测试依赖。真实客户端的登录、配置、凭据和会话均未访问或变更。

## 1. 原始目标与当前实现

用户需要在同一 Windows 11 官方 App 中预先授权四个 Google 账号，之后主动选择其中一个活动账号并用四位数字防误触，尽量减少重复浏览器环节，继续使用既有本地项目和对话。Google 和官方 App 负责认证。自动轮换、额度池、API 反代、并发使用和按账号另建工作区均排除在产品范围外。

| 原始要求 | 当前完成度与证据 |
| --- | --- |
| 1. 保留官方原生 App | 工具是独立 WPF 应用，当前没有注入、改造、启动或终止官方 App 的运行路径。前期 [本地安装元数据记录](feasibility-report.md#本机环境) 记载 2.34.0，Google 有该版本的 Windows 安装包。工具本身未做真实版本识别、兼容握手或切号。[官方下载](https://antigravity.google/download) · [2.34.0 更新记录](https://antigravity.google/docs/changelog#v2-34-0) |
| 2. 四个账号可预先官方授权 | 只有四条默认模拟记录。`AccountCatalog()` 每次启动调用 `Add()` 生成 GUID 和模拟别名，只存在于内存，没有真实账号登记、授权状态或持久化。添加和移除只改模拟目录。[AccountCatalog.cs L3–31](../../src/AGSwitcher.Domain/AccountCatalog.cs#L3-L31) |
| 3. 用户主动快速切号 | `SwitchViewModel.Start()/Confirm()` 走协调器，实际切换动作只改内存。登录指引是 XAML 静态文字，点击本工具的确认按钮既不会打开 App，也不会发起官方浏览器登录。[SwitchViewModel.cs L135–170](../../src/AGSwitcher.App/SwitchViewModel.cs#L135-L170) · [MainWindow.xaml L88–94](../../src/AGSwitcher.App/MainWindow.xaml#L88-L94) |
| 4. 只用一个活动账号 | 模拟后端只有一个 `_currentId`。协调器限制单事务，`InstanceLease.TryAcquire()` 限制同一 Windows 会话内的本工具实例。两者均无法约束真实 App、CLI、扩展、其他 Windows 会话或已运行的远程任务。[SimulatedClient.cs L24–52](../../src/AGSwitcher.Infrastructure/Simulation/SimulatedClient.cs#L24-L52) · [InstanceLease.cs L4–38](../../src/AGSwitcher.Infrastructure/Desktop/InstanceLease.cs#L4-L38) |
| 5. 无自动轮换、额度池、反代和并发使用 | 审阅应用入口、生产实现和项目依赖，未见上述运行路径。工具事务互斥有测试覆盖，真实使用的单活动身份仍待验收。[App.xaml.cs L14–46](../../src/AGSwitcher.App/App.xaml.cs#L14-L46) · [并发测试](../../tests/AGSwitcher.UnitTests/SwitchCoordinatorTests.cs#L50-L76) |
| 6. 共用原项目和对话 | 未按账号创建工作区、切换配置目录、复制项目或迁移对话。已确认的是工具自身未实施隔离。官方 App 换号后能否打开同一目录、看见并继续原对话，属于独立的待验证事项。[LocalSettings.cs L6–28](../../src/AGSwitcher.Infrastructure/Desktop/LocalSettings.cs#L6-L28) · [官方能力矩阵](official-compatibility.md) |
| 7. 四位数字防误触 | `SwitchCoordinator.Begin()` 生成 1000–9999 的随机数字，`ConfirmAsync()` 精确匹配后才进入预检，`Cancel()` 清理确认。当前只保护模拟事务。数字显示在本工具里，其用途为防误触。[SwitchCoordinator.cs L29–86](../../src/AGSwitcher.Application/SwitchCoordinator.cs#L29-L86) |
| 8、9. 安全、数据完整性、合规 | 生产应用没有 Google 凭据、网络登录、认证回调、官方客户端配置或对话读写入口。`LocalSettingsStore.Load()` 只在启动时读取工具自己的诊断配置，`LocalSwitchAudit.Write()` 默认关闭，开启后仅记录状态及原因枚举。真实切换的数据完整性和合规边界仍需随具体方案验证。[LocalSettings.cs L12–42](../../src/AGSwitcher.Infrastructure/Desktop/LocalSettings.cs#L12-L42) · [SwitchEventSource.cs L12–24](../../src/AGSwitcher.Infrastructure/Desktop/SwitchEventSource.cs#L12-L24) |
| 10. Git 与未来 GitHub 发布准备 | 已有版本和锁文件、Windows CI、手动发布候选工作流、打包脚本、许可证和安全政策。历史记录有本地 ZIP、SHA-256 和 SPDX 验收。尚无远程、托管 CI 验证、签名或公开发行。[CI](../../.github/workflows/ci.yml) · [候选工作流](../../.github/workflows/release.yml) · [发布规划](../development/release.md) |

## 2. 实现链路和测试证据

- **生产入口与 Mock 边界。** `App.OnStartup()` 显式创建 `new SimulatedClient(...)` 并注入 `SwitchCoordinator`。`ISwitchClient` 只定义 `Inspect()` 和 `SwitchAsync(Guid)`。生产实现只有 `SimulatedClient`，测试中的 `ControlledClient` 是另一个测试替身。`SimulatedClient.Inspect()` 依据场景枚举生成已安装、兼容、实例数、Agent 和身份快照。正常 `SwitchAsync()` 等待模拟延时后赋值 `_currentId = targetId`。重启失败、认证不可用等也由分支返回。WPF 的活动账号和任务状态直接读取这份模拟快照。这个接口是接入点，现有调用链到此结束。[App.xaml.cs L38–46](../../src/AGSwitcher.App/App.xaml.cs#L38-L46) · [SwitchPorts.cs L5–18](../../src/AGSwitcher.Application/SwitchPorts.cs#L5-L18) · [SimulatedClient.cs L31–84](../../src/AGSwitcher.Infrastructure/Simulation/SimulatedClient.cs#L31-L84) · [SwitchViewModel.cs L108–118](../../src/AGSwitcher.App/SwitchViewModel.cs#L108-L118) · [测试替身 L132–159](../../tests/AGSwitcher.UnitTests/SwitchCoordinatorTests.cs#L132-L159)
- **状态机是什么。** `ConfirmAsync()` 在执行前后调用 `Inspect()`，`CheckSnapshot()` 要求已安装、兼容、正在运行、单实例、无未完成操作、Agent 空闲及身份已知。错误或不明结果可触发恢复锁，`Reconcile()` 重新检查快照。它是模拟切换事务的状态机，源码里没有 OAuth 认证状态机。对演练来说，并发防护和误触确认有用。对纯人工指引来说，要求真实客户端提供整套快照会带来没有证据支撑的复杂度。预检能力应随经确认的真实动作确定。[SwitchCoordinator.cs L64–187](../../src/AGSwitcher.Application/SwitchCoordinator.cs#L64-L187) · [架构说明](../architecture/overview.md)
- **本轮离线复跑。** 使用仓库内 SDK 10.0.404 和现有本地包，在工作树的隔离副本完成锁定还原及 Release 构建，项目构建为 0 警告、0 错误。离线还原未运行 NuGet Audit。VSTest 在 90 秒内未连接 testhost，此次标准宿主运行中止。随后用临时进程内入口调用相同 xUnit 框架，对同一个新构建的测试程序集执行原有用例，**28 项通过、0 失败、0 跳过**。临时入口的独立编译有两条 xUnit 程序集引用兼容性警告，测试框架没有报错。现有测试覆盖默认四项与删除保护、四位输入和取消、错码不调用后端、并发阻止、未知或错误身份、失败及恢复、诊断配置降级。切换断言的两端均为模拟对象，测试与官方 App 无连接。[AccountCatalogTests.cs](../../tests/AGSwitcher.UnitTests/AccountCatalogTests.cs) · [SwitchCoordinatorTests.cs](../../tests/AGSwitcher.UnitTests/SwitchCoordinatorTests.cs) · [SimulationTests.cs](../../tests/AGSwitcher.UnitTests/SimulationTests.cs) · [LocalSettingsTests.cs](../../tests/AGSwitcher.UnitTests/LocalSettingsTests.cs)
- **历次桌面和打包。** 初始化记录显示 WPF 启动、四项、第二进程拒绝、错码、取消、模拟切换、重置和重启通过，基线冒烟后来复跑通过。UI Automation 目标是测试脚本启动的本项目进程。第二轮记录显示修复版离屏绑定和故障场景通过，候选 ZIP 解压可启动，SBOM 校验通过。修复版前台的 Esc、焦点、正确数字加 Enter、150% 修复效果，以及 125% 和干净 Windows 仍待验。本轮未运行交互式冒烟或重打包。[Smoke-Wpf.ps1 L63–115](../../tests/AGSwitcher.IntegrationTests/Smoke-Wpf.ps1#L63-L115) · [历次验收](../development/verification.md)
- **真实 App 的既有证据。** 用户报告亲自通过官方界面完成 A → B → A，两次目标身份登录成功，并报告使用 Windows 数字密码验证。每趟的验证类型、点击数和耗时未分别记录。试验按 2.34.0 单实例准备，执行时未独立采集版本、实例数、主进程 PID、启动时间或窗口变化。用户观察可能无需重启。该记录支持双账号手动往返，四账号、免浏览器、确认一次后零验证、项目对话连续性和进程连续都未验收。[真实试验结果](../development/verification.md#2026-11-23-官方-app-轻量换号) · [试验范围](../security/integration-approval.md#antigravity-app-2340-轻量换号)

## 3. 目标漂移和根本原因

1. **原型验收代替了核心验收。** 仓库的唯一提交和 ADR 明确选择先做模拟，当时缺少公开的快速切换、任务状态和活动身份接口，这个决定保住了安全边界。随后主要可交付项集中在模拟故障、可视交互和发布候选，仓库仍缺一份以四账号实际切换耗时、验证次数、项目与对话连续性为通过条件的产品验收。已有 Phase 0 提到真实验证，发布规划也有证据门槛，但目前都没有能支撑原需求的验收结果。[ADR 0001](../adr/0001-simulation-only.md) · [Phase 0 后续计划](feasibility-report.md#下一阶段) · [发布规划](../development/release.md)
2. **约束先被写成了完整执行模型，能力仍由模拟提供。** `ClientSnapshot` 假定能可靠观察官方 App 全局任务、实例、身份和未完成操作。真实端缺少已确认的对应契约，`RestartFailure` 等场景也容易让路线提前围绕重启设计。Google 的 Overview 描述对话或入口内可见的任务，远程任务也能独立继续运行。单个面板、窗口或 PID 无法证明所有工作均为空闲。[SwitchPorts.cs L5–12](../../src/AGSwitcher.Application/SwitchPorts.cs#L5-L12) · [官方任务面板](https://antigravity.google/docs/side-panel/#sections) · [Remote Agents](https://antigravity.google/docs/remote-agents)
3. **已开发的低优先级工作。** 模拟账号增删、故障下拉、重置、诊断事件、候选包磨合和签名规划都有原型或发布用途。继续扩展这些内容不会减少真实切号步骤。现有代码未出现账号独立工作区、API 代理或真实凭据处理等偏离目标的实现。可见历史只支持判断当前交付与优先级，无法从一条提交重建此前需求或决策的完整时间线。[SwitchViewModel.cs L25–64](../../src/AGSwitcher.App/SwitchViewModel.cs#L25-L64) · [安全设计](../security/design.md)

## 4. A、B、C 的可行性与合规边界

这里的要求 A 是留存并复用多个官方授权状态，B 是第三方工具不得读取、复制、注入或自行刷新官方 OAuth Token，C 是采用官方明确支持的客户端切换机制。

**A、B、C 可以在一种条件下同时成立。** Google 或官方 App 自己管理四个已授权身份，提供明确适用于 Windows App 2.x 的选择机制，由官方完成恢复、刷新和必要的风险验证。工具只选择目标、提示确认，或使用官方明确允许第三方调用的入口。现有公开证据只说明个人账号经浏览器 OAuth、换号指引为退出后重新登录，以及 App、CLI、IDE 扩展、SDK 在本机共享认证。统一认证描述了凭据共享，未提供四个可切换身份的接口或出入站契约。切换云项目、开发 SDK 或处理插件账号的功能也没有给出这项 App 能力。[认证](https://antigravity.google/docs/install) · [官方换号](https://antigravity.google/docs/workspace#option-1-use-a-personal-google-account) · [App 设置](https://antigravity.google/docs/settings) · [SDK](https://antigravity.google/docs/sdk/overview) · [版本记录](https://antigravity.google/docs/changelog#v2-34-0)

| 判断 | 证据与适用范围 |
| --- | --- |
| 已经证实不可行 | 以当前代码实现真实切号不可行，因为唯一后端只替换内存 GUID。以已公开的 App 退出后重新登录路径实现「回访完全不打开浏览器」也不可行，因为其个人账号登录步骤明确发起浏览器 OAuth。此结论仅针对这两条路径。 |
| 目前没有找到官方支持的实现 | 核对认证、Workspace 换号、设置、FAQ、2.34.0 更新记录和 SDK 后，未找到面向 Windows 独立 App 的四账号预授权并免浏览器激活机制，也未找到可供第三方调用的对应公开契约。公开资料的搜索结果不足以断言所有官方版本的能力。 [FAQ](https://antigravity.google/docs/faq) |
| 技术可行，合规性未明确 | 仓库冒烟已证明 Windows UI Automation 能操作自有 WPF 控件。把自动化用于第三方工具驱动官方 App 的账户按钮是另一场景，其可访问性、稳定性和官方支持均缺证据，也无法从原理上保证免浏览器或免验证。条款第 6 条明确约束第三方工具访问服务。对本地界面辅助是否适用及允许的动作范围，需官方明确说明。在 C 下只作只读调查。[微软 UI Automation](https://learn.microsoft.com/dotnet/framework/ui-automation/ui-automation-overview) · [Antigravity 附加条款第 6 条](https://antigravity.google/terms) |
| 尚未测试 | 四账号在官方浏览器已有会话时的逐次点击数、验证频率、凭据输入、回到 App 的身份与窗口连续性，及同目录、原对话可见、原对话续接和恢复情况。现有用户反馈不足以回答。「不输入口令」「没有二次验证」「不打开浏览器」分别记录。 |

Google 支持在网页同时保留多个 Google 账号登录，也支持 passkey 等官方验证方式。这可能缩短实际登录流程，身份验证要求仍由 Google 决定。App 内置浏览器按账号保留的网站 Cookie 属于另一项功能。网页账号会话、网站 Cookie 和官方 App 授权会话的证据分别适用各自场景。用户报告的 Windows 数字密码不足以确认具体认证因子。[Google 网页多账号](https://support.google.com/accounts/answer/1721977?co=GENIE.Platform%3DDesktop) · [Google passkey](https://support.google.com/accounts/answer/13548313) · [Antigravity 内置浏览器](https://antigravity.google/docs/browser)

读取、复制、注入或刷新官方凭据，使用包含认证数据的配置快照换号，抓取或回放认证回调，以及修改客户端或通过非官方通道接入服务，均与本项目明确边界冲突。对 Google 保护措施的规避也受 Google 通用条款约束。仓库的社区参考项目只能作为市场和界面观察，它们的声明没有验证 Windows App 2.34.0 可用或得到 Google 许可。[参考项目调查](reference-projects.md) · [Antigravity 附加条款](https://antigravity.google/terms) · [Google 通用条款](https://policies.google.com/terms?hl=en)

## 5. 两条纠偏路线

**方案 A · 官方手动切号基准。** 用户在原生 App 退出，由 App 发起官方浏览器登录，用户选择目标并回到 App 核对身份。现有工具可作为进程外操作指引。若日后保留四位数，它只保护工具内的目标确认或交接动作，独立在官方界面操作时仍由官方控制流程。

**方案 B · 调查官方原生多身份切换。** 只读核对适用版本的公开发布说明、官方支持答复、官方 UI 文档和集成契约。优先寻找原生个人账号切换界面，其次核对 Google 是否明确允许第三方用某个公开入口发起本地切换、读取最少的活动身份元数据，并说明对工作区和其他认证面的影响。先以官方原生路径验证体验。若只有人工入口，即使它很快，独立工具也只能提示与交接。若明确允许外部调用，再判断是否有理由做最薄的接入。

| 维度 | 方案 A · 官方手动 | 方案 B · 调查原生切换 |
| --- | --- | --- |
| 重复浏览器登录 | 已公开流程每次进入浏览器 OAuth。已有浏览器会话可能减少输入，重复挑战无法保证省略。 | 只有官方明确支持留存并激活已授权身份，才有机会省去常态回访的浏览器流程。过期和风险挑战仍由官方决定。 |
| 四账号快速选择 | 工具可提示四个本地别名，实际选择在官方浏览器。四账号选择器出现频率、操作数和耗时待量化。 | 以同一安装四身份可选、逐次核对活动账号为验收；目前无支持证据。 |
| 原有项目和对话 | 工具不移动数据。B 能否打开原目录、查看并继续 A 的对话，以及回 A 后是否完整，均待测。 | 需要同目录及对话逐项验收。官方选择器存在也不自动授予跨账号对话访问。 |
| 是否重启 | 有用户观察可能无须重启。缺主进程 PID、启动时间和窗口记录。 | 取决于官方实现，需分别记录主进程和窗口；无实测。 |
| 认证凭据 | 仅用户和官方软件处理，工具不接触。 | 调查期只读公开资料和脱敏的非秘密证据。可接受实现仍由官方负责凭据。 |
| 官方支持 | 官方支持 App 退出并通过浏览器重登。独立工具仅提供本地指引。 | 未找到适用的多身份激活契约。第三方调用和自动化需要该动作的明确支持。 |
| 账号安全风险 | 较低。仍需留意误选、Google 正常再验证、同机共享登录面和未完成任务。 | 只读阶段低。真实接入风险取决于官方边界、身份核验和其他入口的影响。 |
| 实现难度 | 低。工作重点是测量现有官方体验和核实数据连续性。 | 调查可立即开始，交付难度由官方能力决定。公开入口缺失会阻断集成交付。 |
| 代码复用 | 复用登录指引、四位确认交互及相关单测、窗口可访问性、打包脚本。模拟完整协调器只保留为演练。 | 可复用目标选择、确认、单事务和结果验证的行为及测试。快照、恢复和端口要由真实契约决定。 |

## 6. 保留、暂停与建议顺序

- **优先保留**四位确认和取消交互、错码与并发测试、身份核对失败时停止的行为、明确标识的离线模拟、WPF 可访问性与 Git/CI/打包基础。它们分别支持防误触、可验证交接和将来发行。[SwitchCoordinatorTests.cs](../../tests/AGSwitcher.UnitTests/SwitchCoordinatorTests.cs) · [打包脚本](../../scripts/package.ps1)
- **暂停或推迟**新增模拟故障、认证或恢复状态扩展、诊断面板、真实进程控制、配置改造、签名安装器和正式发布。架构能否直接复用，由已验证的官方动作决定。现有原型和提交保留。
- **建议顺序**为先走 B 的只读能力核查，以 A 的官方操作作为可测基准。若找到明确的原生能力，再验证它能否缩短切号并保留数据。若无证据，保留 A 作为已知可用的人工路径，由用户决定它是否值得成为一个缩减范围的辅助产品。免重复验证仍是核心目标的开放项。

## 7. 下一阶段最小可行性实验

1. **先过只读门槛。** 针对当前 Windows App 2.x、个人 Google 账号和同一设备，确认 Google 是否支持保存四个授权后的 App 会话、如何切到指定身份、是否允许外部发起、何时会重新验证、换号如何影响项目、会话和共享认证面。记载公开链接或用户提供的官方答复及适用版本。没有证据时记为未找到，真实集成留待证据。
2. **单次批准后测两账号。** 由用户在官方 App 与官方浏览器亲自完成 A → B → A。使用非关键账号、一个共同的空白本地目录和 A 创建的无私密测试对话。开测前确认版本、活动账号、其他共享入口无进行中工作，以及 A 的恢复方式。当前任务无真实实验授权，本步仅为下一阶段设计。[既有批准流程](../security/integration-approval.md)
3. **分别验证操作成本和数据。** 每趟记录退出或选择到确认目标身份的秒数、App 与浏览器的人工步骤、是否打开浏览器、是否选账号、是否输入口令或需要额外验证。只读记录 App 主进程 PID 和启动时间，另记窗口是否重建。B 下分别检验能否打开同一目录、看到 A 的测试对话、在该原对话提交一条无害消息并得到正常响应。若原对话不可见，记为无法验证续写。回 A 后复核文件与对话仍可用。记录脱敏结果和时间，不记录凭据或私密内容。身份、任务或数据状态出现疑问时停止，恢复通过官方界面完成。
4. **再扩展到四账号。** 两账号的身份核对及数据结果符合用户接受条件后，另行批准并测试四个身份逐一预授权、逐个激活并回访，在保持单个活动身份的前提下记录每次浏览器和验证结果。若目标是严格免浏览器，验收限定为已授权且会话仍有效时的普通回访，要求均无浏览器打开。若用户接受官方浏览器交接，则使用用户认可的秒数、点击次数及追加验证频率作为阈值。复核 App 重启后和隔日回访，分别记录结果。

## 8. 需要用户决定的取舍

1. 官方只有退出并重登时，是否接受「仍进浏览器，但尽量只选账号或用已有官方验证方式」，以及可接受的换号时间、步骤和重复验证频率。
2. 原对话的最低标准是切回原账号后完整可见并可继续，还是要求目标账号在同一原对话内继续。后者须满足官方账号隔离和访问规则。
3. 无重启是硬要求还是偏好。是否接受窗口重建、主进程持续或短暂官方重启，各自的体验影响需要按实测评估。
4. 如果只找到原生人工入口，是否保留独立小工具用于四位确认和交接，或只用官方 App。若没有满足原始体验的受支持入口，是否停在可行性调查阶段。
5. 是否另行批准两账号空白项目实验，以及明确接受哪些共享认证面的影响和恢复步骤。凭据边界仍按 B、C 执行。
