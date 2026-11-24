# 构建与测试说明 (Build Guide)

## 环境要求

- Windows 11 x64
- Rust Stable 工具链（`x86_64-pc-windows-msvc`）
- Visual Studio 2022 C++ Build Tools 与 Windows 10/11 SDK
- Microsoft Edge WebView2 Runtime（Windows 11 默认内置）

## 标准命令

在仓库根目录执行：

```powershell
# 代码格式检查与静态分析（CI 标准）
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings

# 运行全部隔离合成单元测试（不触碰真实客户端与真实凭据）
cargo test --locked

# 构建 Release 可执行文件并生成候选 ZIP 包
cargo build --locked --release
./scripts/package.ps1
```

## 测试安全说明

默认 `cargo test --locked` 仅运行合成凭据（`ag-switch-lite-test:*` 独立命名空间，析构自动清理）、离屏测试窗口与子进程锁探针，不会读取或修改 `gemini:antigravity`，也不会关闭正在运行的 Antigravity。标记为 `#[ignore]` 的真机生命周期测试仅限在隔离测试账户且保存全部工作后由人工显式授权执行。
