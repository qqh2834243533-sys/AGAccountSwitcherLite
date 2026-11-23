# 分层与状态机

| 项目 | 职责 |
| --- | --- |
| AGSwitcher.App | WPF 视图、命令、可访问性标识和 ViewModel。App 在启动时组装依赖 |
| AGSwitcher.Application | 确认、安全预检、切换事务、结果验证、恢复 |
| AGSwitcher.Domain | 模拟候选目录、能力状态、切换状态与原因 |
| AGSwitcher.Infrastructure | 模拟客户端、Windows Named Mutex、配置文件和 Windows 本地事件 |
| AGSwitcher.UnitTests | 状态机单测与模拟后端测试 |

典型流程为 `LOCKED → CONFIRMING → PRECHECK → SWITCHING → VERIFYING → COMPLETED → LOCKED`。错误确认留在确认界面。取消时清除数字并回到锁定。预检失败返回 `BLOCKED`。执行结果不明确时返回 `FAILED`，保持恢复锁直到显式重置并取得可信快照。

模拟数据驻留进程内。后端只有一个活动 ID。界面只在空闲时允许修改候选账号或场景，执行和核验期间拒绝重复请求及关闭窗口。Windows 终止进程时释放实例锁。

当前唯一接入的后端为模拟客户端。真实兼容性参见 [官方能力矩阵](../research/official-compatibility.md)。
