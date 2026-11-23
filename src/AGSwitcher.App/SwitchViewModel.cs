using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Input;
using AGSwitcher.Application;
using AGSwitcher.Domain;
using AGSwitcher.Infrastructure.Simulation;

namespace AGSwitcher.App;

public sealed record ScenarioOption(SimulationScenario Value, string Label);

internal sealed class SwitchViewModel : INotifyPropertyChanged
{
    private readonly AccountCatalog _accounts;
    private readonly SimulatedClient _client;
    private readonly SwitchCoordinator _coordinator;
    private readonly RelayCommand[] _commands;
    private Account? _selectedAccount;
    private ScenarioOption _selectedScenario;
    private string _confirmationInput = "";
    private string _confirmationError = "";
    private string _message;

    public SwitchViewModel(AccountCatalog accounts, SimulatedClient client, SwitchCoordinator coordinator,
        bool settingsFallback)
    {
        _accounts = accounts;
        _client = client;
        _coordinator = coordinator;
        Accounts = new ObservableCollection<Account>(accounts.Accounts);
        _selectedScenario = Scenarios[0];
        _message = settingsFallback ? "设置文件无效，已加载默认设置。" : "选择一个候选账号开始模拟。";

        StartCommand = new RelayCommand(Start, () => IsLocked && _selectedAccount is not null
            && _client.Inspect().ActiveAccountId != _selectedAccount.Id);
        CancelCommand = new RelayCommand(Cancel, () => IsConfirming);
        ConfirmCommand = new RelayCommand(Confirm, () => IsConfirming);
        AddCommand = new RelayCommand(() => Accounts.Add(_accounts.Add()), () => IsLocked);
        RemoveCommand = new RelayCommand(Remove, () => CanRemove);
        ResetCommand = new RelayCommand(Reset, () => IsLocked);
        _commands = [StartCommand, CancelCommand, ConfirmCommand, AddCommand, RemoveCommand, ResetCommand];
    }

    public event PropertyChangedEventHandler? PropertyChanged;

    public ObservableCollection<Account> Accounts { get; }
    public IReadOnlyList<ScenarioOption> Scenarios { get; } =
    [
        new(SimulationScenario.Normal, "正常"),
        new(SimulationScenario.AgentBusy, "Agent 忙碌"),
        new(SimulationScenario.AgentStateUnknown, "任务状态未知"),
        new(SimulationScenario.ClientMissing, "客户端不存在"),
        new(SimulationScenario.ClientStopped, "客户端未运行"),
        new(SimulationScenario.IncompatibleVersion, "版本不兼容"),
        new(SimulationScenario.CompatibilityUnknown, "版本兼容性未知"),
        new(SimulationScenario.UnknownIdentity, "活动身份未知"),
        new(SimulationScenario.OtherManagedInstance, "存在其他受管实例"),
        new(SimulationScenario.UnfinishedOperation, "存在未完成操作"),
        new(SimulationScenario.SwitchFailure, "切换失败"),
        new(SimulationScenario.RestartFailure, "模拟重启失败"),
        new(SimulationScenario.AuthUnavailable, "官方认证不可用"),
        new(SimulationScenario.VerificationFailure, "身份验证失败")
    ];

    public Account? SelectedAccount
    {
        get => _selectedAccount;
        set
        {
            _selectedAccount = value;
            Refresh();
        }
    }

    public ScenarioOption SelectedScenario
    {
        get => _selectedScenario;
        set
        {
            if (!IsLocked || value is null)
            {
                return;
            }

            _selectedScenario = value;
            _client.Scenario = value.Value;
            Refresh();
        }
    }

    public string ConfirmationInput
    {
        get => _confirmationInput;
        set
        {
            _confirmationInput = value;
            Notify();
        }
    }

    public bool IsLocked => _coordinator.Status.Phase == SwitchPhase.Locked;
    public bool IsConfirming => _coordinator.Status.Phase == SwitchPhase.Confirming;
    public bool CanClose => IsLocked || IsConfirming;
    private bool CanRemove => IsLocked && _selectedAccount is not null
        && _client.Inspect().ActiveAccountId is Guid current && current != _selectedAccount.Id;

    public string ActiveAccount => Accounts.FirstOrDefault(
        account => account.Id == _client.Inspect().ActiveAccountId)?.Label ?? "身份未知";
    public string Challenge => _coordinator.Status.Challenge ?? "";
    public string ConfirmationError => _confirmationError;
    public string Message => _message;
    public string TaskState => _client.Inspect().Agent switch
    {
        AgentActivity.Idle => "空闲",
        AgentActivity.Busy => "忙碌",
        _ => "未知"
    };
    public string StateLabel => _coordinator.Status.Phase switch
    {
        SwitchPhase.Confirming => "等待确认",
        SwitchPhase.Precheck => "安全检查",
        SwitchPhase.Switching => "模拟切换中",
        SwitchPhase.Verifying => "验证身份",
        _ => "已锁定"
    };

    public RelayCommand StartCommand { get; }
    public RelayCommand CancelCommand { get; }
    public RelayCommand ConfirmCommand { get; }
    public RelayCommand AddCommand { get; }
    public RelayCommand RemoveCommand { get; }
    public RelayCommand ResetCommand { get; }

    private void Start()
    {
        if (_selectedAccount is null)
        {
            return;
        }

        ConfirmationInput = "";
        _confirmationError = "";
        Show(_coordinator.Begin(_selectedAccount.Id));
    }

    private void Cancel()
    {
        ConfirmationInput = "";
        Show(_coordinator.Cancel());
    }

    private async void Confirm()
    {
        var operation = _coordinator.ConfirmAsync(ConfirmationInput);
        if (!IsConfirming)
        {
            ConfirmationInput = "";
            _message = "正在检查并执行模拟切换。";
            Refresh();
        }

        var result = await operation;
        if (result.Issue == SwitchIssue.InvalidConfirmation)
        {
            _confirmationError = "数字不匹配，请重新输入。";
        }

        Show(result);
    }

    private void Remove()
    {
        var selected = _selectedAccount;
        if (selected is not null && _accounts.Remove(selected.Id, _client.Inspect().ActiveAccountId))
        {
            Accounts.Remove(selected);
        }
    }

    private void Reset()
    {
        _client.Reset(Accounts[0].Id);
        _selectedScenario = Scenarios[0];
        Show(_coordinator.Reconcile());
    }

    private void Show(SwitchResult result)
    {
        _message = result.Issue switch
        {
            SwitchIssue.Cancelled => "已取消模拟切换。",
            SwitchIssue.ConcurrentOperation => "已有模拟操作在执行。",
            SwitchIssue.UnknownTarget => "候选账号已移除。",
            SwitchIssue.InvalidConfirmation => "等待正确的模拟确认数字。",
            SwitchIssue.ClientMissing => "模拟客户端不存在。",
            SwitchIssue.IncompatibleVersion => "模拟客户端版本不兼容。",
            SwitchIssue.CompatibilityUnknown => "模拟客户端兼容性未知。",
            SwitchIssue.ClientStateUnknown => "无法确认模拟客户端状态。",
            SwitchIssue.ClientNotRunning => "模拟客户端未运行。",
            SwitchIssue.ManagedInstanceConflict => "检测到其他受管模拟实例。",
            SwitchIssue.UnfinishedOperation => "检测到未完成的模拟操作。",
            SwitchIssue.AgentBusy => "模拟 Agent 正在执行任务，切换已阻止。",
            SwitchIssue.AgentStateUnknown => "模拟 Agent 状态未知，切换已阻止。",
            SwitchIssue.ActiveIdentityUnknown => "无法验证当前模拟账号。",
            SwitchIssue.AlreadyActive => "此账号已处于模拟活动状态。",
            SwitchIssue.RecoveryRequired => "上次模拟操作失败，请先重置模拟。",
            SwitchIssue.RestartFailed => "模拟重启失败，请重置模拟。",
            SwitchIssue.AuthUnavailable => "当前场景无法完成官方认证，请重置模拟。",
            SwitchIssue.SwitchFailed => "模拟切换失败，请重置模拟。",
            SwitchIssue.VerificationFailed => "无法验证模拟切换结果，请重置模拟。",
            SwitchIssue.UnexpectedFailure => "模拟操作异常，请重置模拟。",
            _ => result.Phase switch
            {
                SwitchPhase.Completed => $"模拟切换完成，当前活动账号为 {ActiveAccount}。",
                SwitchPhase.Confirming => "请输入本次模拟确认数字。",
                _ => "模拟状态已重置。"
            }
        };
        Refresh();
    }

    private void Refresh()
    {
        Notify(string.Empty);
        foreach (var command in _commands)
        {
            command.Refresh();
        }
    }

    private void Notify([CallerMemberName] string? property = null) =>
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(property));
}

internal sealed class RelayCommand(Action execute, Func<bool> canExecute) : ICommand
{
    public event EventHandler? CanExecuteChanged;
    public bool CanExecute(object? parameter) => canExecute();

    public void Execute(object? parameter)
    {
        if (canExecute())
        {
            execute();
        }
    }

    public void Refresh() => CanExecuteChanged?.Invoke(this, EventArgs.Empty);
}
