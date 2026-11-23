using AGSwitcher.Application;
using AGSwitcher.Domain;

namespace AGSwitcher.Infrastructure.Simulation;

public enum SimulationScenario
{
    Normal,
    AgentBusy,
    AgentStateUnknown,
    ClientMissing,
    ClientStopped,
    IncompatibleVersion,
    CompatibilityUnknown,
    UnknownIdentity,
    OtherManagedInstance,
    UnfinishedOperation,
    SwitchFailure,
    RestartFailure,
    AuthUnavailable,
    VerificationFailure
}

public sealed class SimulatedClient(Guid initialId, TimeSpan latency) : ISwitchClient
{
    private Guid? _currentId = initialId;
    private bool _interrupted;

    public SimulationScenario Scenario { get; set; }

    public ClientSnapshot Inspect()
    {
        var scenario = Scenario;
        return new(
            Installed: scenario != SimulationScenario.ClientMissing,
            Compatibility: scenario switch
            {
                SimulationScenario.IncompatibleVersion => Capability.Unsupported,
                SimulationScenario.CompatibilityUnknown => Capability.Unknown,
                _ => Capability.Supported
            },
            Running: !_interrupted && scenario != SimulationScenario.ClientStopped
                && scenario != SimulationScenario.ClientMissing,
            ManagedInstanceCount: scenario == SimulationScenario.OtherManagedInstance ? 2 : 1,
            UnfinishedOperation: _interrupted || scenario == SimulationScenario.UnfinishedOperation,
            Agent: scenario switch
            {
                SimulationScenario.AgentBusy => AgentActivity.Busy,
                SimulationScenario.AgentStateUnknown => AgentActivity.Unknown,
                _ => AgentActivity.Idle
            },
            ActiveAccountId: scenario == SimulationScenario.UnknownIdentity ? null : _currentId);
    }

    public async Task<SwitchAttempt> SwitchAsync(Guid targetId)
    {
        var scenario = Scenario;
        await Task.Delay(latency);

        switch (scenario)
        {
            case SimulationScenario.SwitchFailure:
                return SwitchAttempt.Failed;
            case SimulationScenario.RestartFailure:
                _interrupted = true;
                _currentId = null;
                return SwitchAttempt.RestartFailed;
            case SimulationScenario.AuthUnavailable:
                return SwitchAttempt.AuthUnavailable;
            case SimulationScenario.VerificationFailure:
                _currentId = null;
                return SwitchAttempt.Applied;
            default:
                _currentId = targetId;
                return SwitchAttempt.Applied;
        }
    }

    public void Reset(Guid accountId)
    {
        Scenario = SimulationScenario.Normal;
        _currentId = accountId;
        _interrupted = false;
    }
}
