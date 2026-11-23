# 在 Windows 构建和测试

需要 Git 与 .NET 10 SDK x64。使用 global.json 固定的 SDK 构建候选包。本机已有项目私有 SDK，可在仓库根目录执行 .\.local\dotnet\dotnet.exe --info 核对。

~~~powershell
dotnet restore AGSwitcher.sln --locked-mode --force-evaluate
dotnet build AGSwitcher.sln -c Release --no-restore -p:ContinuousIntegrationBuild=true
dotnet test AGSwitcher.sln -c Release --no-build --no-restore
.\scripts\package.ps1
~~~

NuGet 还原仅使用 nuget.org。锁文件记录解析版本和内容哈希，NuGet Audit 检查直接及传递依赖，警告会令构建失败。构建启用 .NET 静态分析器。xUnit、Visual Studio 测试适配器和 Microsoft.NET.Test.Sdk 只用于测试。

## 运行 WPF

~~~powershell
dotnet run --project .\src\AGSwitcher.App\AGSwitcher.App.csproj -c Release --no-build
~~~

本机只使用仓库内 SDK 时，可用以下命令直接启动已构建版本。

~~~powershell
.\.local\dotnet\dotnet.exe .\src\AGSwitcher.App\bin\Release\net10.0-windows\win-x64\AGSwitcher.App.dll
~~~

## Windows 窗口验收

在可用的交互式 Windows 桌面上运行窗口验收脚本。此脚本通过 Windows UI Automation 操作本项目的临时进程，需要先关闭同一 Windows 会话里的其他模拟窗口。GitHub CI 的单元与模拟测试无需桌面。

~~~powershell
powershell.exe -NoProfile -STA -ExecutionPolicy Bypass -File .\tests\AGSwitcher.IntegrationTests\Smoke-Wpf.ps1 -Executable .\src\AGSwitcher.App\bin\Release\net10.0-windows\win-x64\AGSwitcher.App.exe
~~~

使用私有 SDK 时追加参数 -DotnetRoot .\.local\dotnet。上面命令的执行策略只用于该 PowerShell 进程。真实 Antigravity 测试需另行批准。
