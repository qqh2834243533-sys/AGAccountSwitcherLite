using System.IO;
using System.Windows;
using AGSwitcher.Application;
using AGSwitcher.Domain;
using AGSwitcher.Infrastructure.Desktop;
using AGSwitcher.Infrastructure.Simulation;

namespace AGSwitcher.App;

public partial class App : System.Windows.Application
{
    private InstanceLease? _instance;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);
        try
        {
            _instance = InstanceLease.TryAcquire(InstanceLease.ApplicationName);
        }
        catch (Exception exception) when (
            exception is UnauthorizedAccessException or IOException or WaitHandleCannotBeOpenedException)
        {
            MessageBox.Show("无法确认模拟工具的实例锁。请关闭工具后重试。", "AG Account Switcher Lite",
                MessageBoxButton.OK, MessageBoxImage.Warning);
            Shutdown(1);
            return;
        }

        if (_instance is null)
        {
            MessageBox.Show("当前 Windows 会话已打开一个模拟切换器。", "AG Account Switcher Lite",
                MessageBoxButton.OK, MessageBoxImage.Information);
            Shutdown(2);
            return;
        }

        var accounts = new AccountCatalog();
        var client = new SimulatedClient(accounts.Accounts[0].Id, TimeSpan.FromMilliseconds(450));
        var (settings, fallback) = LocalSettingsStore.Load(LocalSettingsStore.DefaultPath);
        var coordinator = new SwitchCoordinator(accounts, client, new LocalSwitchAudit(settings.EnableDiagnostics));
        MainWindow = new MainWindow
        {
            DataContext = new SwitchViewModel(accounts, client, coordinator, fallback)
        };
        MainWindow.Show();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        _instance?.Dispose();
        base.OnExit(e);
    }
}
